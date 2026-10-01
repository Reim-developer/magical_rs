# Benchmarks

`magical_rs` against the libraries a reader would compare it to, on one shared
corpus. Its own workspace, not published, and not part of the root one.

## Why it is outside `crates/`

The root `Cargo.lock` is quoted in the readme as holding exactly one package.
Criterion is a few hundred packages, so a benchmark target inside the root
workspace would make that claim false for the sake of a directory that has
nothing to do with the library. It is in `workspace.exclude`, and
`tests/workspace.rs` fails if it is ever dropped from that list.

## Running it

```sh
make bench           # criterion, then the report
make bench-report    # the report alone: no criterion, about ten seconds
make bench-mutations # the harness's own mutation check
```

`make bench` turns the `libmagic` feature on when it can find libmagic and leaves
it off when it cannot. The report prints which libraries actually ran, so a
two-row table is never mistaken for a three-row one.

## Getting libmagic

libmagic is a C library with a 10 MB rules database. `magic-sys`'s build script
tries `pkg-config` and then `vcpkg` and **fails** rather than degrading, which is
why it is an optional dependency behind a feature and not a dependency behind a
`cfg`: a missing libmagic has to be a benchmark with one fewer row, not a build
that is red.

| Platform | What to do | What the crate needs |
|---|---|---|
| Debian / Ubuntu | `sudo apt-get install libmagic-dev libmagic-mgc` | two packages: the headers, and the compiled rules database |
| macOS | it is preinstalled | nothing |
| Windows, vcpkg | `vcpkg install libmagic:x64-windows` | `$VCPKG_ROOT`, `$VCPKGRS_TRIPLET`, `$VCPKGRS_DYNAMIC=1` |

The Windows case is worth spelling out, because it is the one that does not
work by accident:

- `$VCPKGRS_TRIPLET` and `$VCPKGRS_DYNAMIC` are the variables the **`vcpkg` Rust
  crate** reads, not the ones the `vcpkg` executable reads. `VCPKG_TARGET_TRIPLET`
  is ignored and the default triplet `x64-windows-static-md` is used, which is
  not the one anything installed.
- `$VCPKG_ROOT` must contain `.vcpkg-root` and an `installed/<triplet>` tree.
  A manifest-mode vcpkg installs into `<manifest>/vcpkg_installed/<triplet>`;
  the crate only looks under `$VCPKG_ROOT/installed/`.
- The database is **not** found by `magic_load(cookie, NULL)` on Windows, and not
  reliably on Linux either. libmagic's `MAGIC` macro resolves to whatever its
  build passed -- the man page documents the upstream value as
  `/usr/local/share/misc/magic`, while Debian and Ubuntu ship the file in
  `/usr/share/misc` -- so the compiled-in default is right on some machines and
  wrong on others for reasons nothing on this side can see. The crate searches
  `$MAGIC`, then the compiled-in default, then the paths distributions put it in,
  then the vcpkg layouts, validating each with a real `magic_load`. It panics with
  the list it tried if none of them work, rather than running with no rules and
  reporting `application/octet-stream` for all four files.

  `benchmarks.yml` also sets `$MAGIC` from a `find`, because the first run of that
  workflow relied on the compiled-in default, did not find the database, and
  panicked -- leaving a **green** job that had measured nothing.

`filemagic`'s `vendored` feature looks like the obvious answer on Windows and is
not: it builds libmagic from source with the `cc` crate, which does find MSVC's
`cl.exe`, and then fails on `file.h:82` because `softmagic.c` needs a POSIX
`regex.h` that MSVC's CRT does not have. The vendored source does not bundle one.
vcpkg's port does, which is why the port is the route.

## What is measured, and what is not

**The timing tables are a comparison.** Every library gets the same `Vec<u8>` of
the same length, in the same order, the same number of calls. None of them chose
its own input.

**The answer table is not.** The format cases in the timing corpus are generated
from `SIGNATURE_KIND` -- this repository's own library's own table -- so of course
that library recognises all of them. A correctness score over that corpus would
be measuring the generator. So the answers are printed as a disagreement list
about the four real files the repository already commits, which is a question
that can be answered.

That split was not designed in. It was found: libmagic reports
`application/octet-stream` for a buffer holding a valid eight-byte PNG signature
and 36,862 zero bytes, and `image/png` for a real PNG whose first eight bytes are
the same eight, because its rules look at the IHDR chunk. With no real files in
the corpus, the answer table had 125 rows and all of them were that one fact.

**Two entry points, two tables.** A caller either has the bytes or has a path,
and the numbers are not close. `magical_rs` and `infer` both have a path-based
entry point, so all three are measured both ways, and the from-path table
includes the I/O. On a developer laptop `infer` is *faster* from a path than
`magical_rs` is, because `magical_rs` reads 36,870 bytes to be able to detect
ISO 9660 and `infer` reads only what its own table needs. That row is in the
report on purpose.

**No threshold.** There is no number in this crate that a test compares against a
limit, and adding one would defeat the purpose. A shared runner varies by a
factor of two between jobs.

## Three things the harness got wrong, and what they cost

Both were found by comparing this crate's output against Criterion's, and both
are recorded here because the fix is not obvious from the code.

**A green job that measured nothing.** The first run of `benchmarks.yml` had
`continue-on-error: true` on the job, on the reasoning that a busy runner should
not turn a pull request red over a measurement. It panicked in the database
search, which skipped every step that produces a number, and the workflow
reported success. The job no longer has it: there is no threshold here to fail,
so the only steps that can fail are lint and test, and those should be red when
they are red.

**Cache pollution between libraries.** The first version timed library A over
the whole corpus, then B, then C, rotating the order between passes. libmagic
takes about 400 microseconds per buffer and `magical_rs` about 57 nanoseconds,
and the corpus is 10.5 MB, so one libmagic pass evicts every buffer the next
library is about to read. `magical_rs` came out at 177 ns where Criterion
measures 57 ns for identical work, and the difference was entirely libmagic's
memory traffic.

The fix is not to interleave more finely. Interleaving per call was tried and is
worse: `Instant::now()` twice per call is tens of nanoseconds on Windows, against
a 57-nanosecond measurement. The number came out as 397 ns, which is neither. The
fix is to equalise the starting state -- the corpus is read into cache before
every timed pass, so every library begins warm.

**A `NaN` median made the whole ratio column `inf`.** The first version folded
the medians with `f64::max`, and `f64::max(inf, NaN)` is `inf`, so one bad row
made every ratio print as `infx`. Every ratio in the column was unreadable
rather than merely wrong. The fold now filters non-finite values first, and a
library that produces a NaN sees its own NaN in its own row.

`make bench-mutations` applies 21 changes that are each one of these, or a
neighbour of one, and requires the tests to go red for each. All 21 are caught.

Two of the twenty-one are housekeeping that turned out to matter more than it
looks, and both are in the script because of how they failed:

- A mutation's `Find` string had to match the file byte for byte, so two of them
  went stale on a line-ending difference -- the code was fine and the newline was
  not. The script now reads the line ending from the file and applies it to both
  the search and the replacement. A mutation that reports itself stale is worse
  than one that fails, because a stale mutation looks like a test that does not
  exist for anything.
- `cargo fmt` does not change a file's line ending, and `[System.IO.File]::WriteAllLines`
  writes the platform's. So the line ending of a source file here depends on which
  tool last touched it, and there is no way to write a `Find` down once and have
  it stay correct.

## Layout

| Path | What it is |
|---|---|
| `src/corpus.rs` | The buffers. Generated from `SIGNATURE_KIND`, deterministic, no committed binary. |
| `src/adapter.rs` | One wrapper per library. The only thing an adapter does is be the library. |
| `src/report.rs` | The medians, and the markdown a pull request gets. |
| `benches/detect.rs` | Criterion, per case, with its noise estimates. |
| `src/bin/report.rs` | Writes that markdown, to stdout or to a file. |
| `tests/harness.rs` | The harness's own tests. Every one of them is about a way the report could lie. |
| `mutations.ps1` | The 21 changes above, applied and reverted. |
