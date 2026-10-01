# CHANGELOG
- [CHANGELOG](#changelog)
  - [Unreleased: macros, a fluent API, and an API reference](#unreleased-macros-a-fluent-api-and-an-api-reference)
  - [Unreleased: a first-byte detection index](#unreleased-a-first-byte-detection-index)
  - [Unreleased: benchmarks against infer and libmagic](#unreleased-benchmarks-against-infer-and-libmagic)
  - [Unreleased: one format dataset](#unreleased-one-format-dataset)
  - [magical-py: Version 0.4.0](#magical-py-version-040)
  - [magical-py: Version 0.3.0](#magical-py-version-030)
  - [magical-py: Version 0.2.0](#magical-py-version-020)
  - [Version: 0.1.3](#version-013)
  - [Version: 0.2.0](#version-020)
  - [Version: 0.2.1:](#version-021)
  - [Version: 0.3.0:](#version-030)
  - [Version: 0.3.1, `Minor edits`](#version-031-minor-edits)
  - [Version: 0.4.0 `Major API Update`](#version-040-major-api-update)
  - [Version: 0.4.5 `Major API Update`](#version-045-major-api-update)
  - [Version: 0.6.0 `Signature Tightening`](#version-060-signature-tightening)
  - [Version: 0.6.1 `Documentation and Test Coverage`](#version-061-documentation-and-test-coverage)
  - [Version: 0.6.2 `Overflow Fix and Release Gates`](#version-062-overflow-fix-and-release-gates)
  - [Version: 0.6.3 `Header Padding Fix`](#version-063-header-padding-fix)
  - [Version: 0.6.4 `Offset Arithmetic and a Wrong Constant`](#version-064-offset-arithmetic-and-a-wrong-constant)


## Unreleased: macros, a fluent API, and an API reference

**Level 2 stops asking for five fields, level 1 has a shorter spelling, and the
readme lists what all three languages export.**

* **Added `magic_rules!`.** Level 2 rule sets as a table:

  ```rust
  static RULES: &[MagicCustom<Kind>] = magic_rules![
      (Kind::CadFile, b"ACAD", read 2048),
      (Kind::ShortFile, [b"<<", b">>"], read 4),
      (Kind::Fallback, via all [is_cad, is_short]),
  ];
  ```

  Sugar for the five-field `MagicCustom` literal and nothing else. A byte rule
  stops spelling out `offsets: &[0]` every time and a predicate rule stops
  spelling out `signatures: &[]` and `offsets: &[]` — two fields that are noise,
  and where leaving a signature in place reads as "and also" and means nothing.
  `at` and `read` are optional and default to the crate's own `DEFAULT_OFFSET` and
  `DEFAULT_MAX_BYTES_READ`. Not behind a feature flag: it adds no dependency,
  allocates nothing, and emits no code unless invoked.

* **Added the `magical_fluent` feature: `bytes.detect()`.** Plus `detect_within`,
  `is`, `is_any`, and `detect_in` for level 2. Gated, and the only flag here that
  is not about a level — it puts a method on `[u8]`, so writing it is a dependency
  on this crate for a slice of bytes you could have handed to anything, and
  nothing in the signature says so afterwards. With the flag off there is no
  trait, no type, and not an empty module in the documentation either: the `cfg`
  is on the `pub mod` line rather than only inside the module, and a test reads the
  crate root to check it.

  There is deliberately no `detect_or`. `FileKind` has no "unknown" variant — all
  114 of them are real formats — so there is no honest value to substitute.

* **The readme has an API reference for all three languages**, one table each, and
  a fourth that lists where the three are deliberately *not* the same. A reader
  who knew the crate had `is_shebang` had no way to find that out from the
  documentation before.

  Writing it turned up two claims that were about to be written down wrong:
  `releaseRules` drops a handle rather than freeing the wasm-side memory, and
  `neededBytes()` is 36,870 while `DEFAULT_MAX_BYTES_READ` is 2,048 — not the same
  number, and the readme was about to describe them as if they were.


## Unreleased: a first-byte detection index

**Detection no longer walks all 114 rules to answer.**

* **Detection is indexed by the first byte of a signature.** `match_types` was
  `SIGNATURE_KIND.iter().find(..)`, so its cost was the position of the match in the
  table: 5 ns for PNG at position 0, 123 ns for GIF at 36, and 525 ns for a file the
  table does not recognise, which had to be tried against all 114. It is now 5, 21 and
  **31 ns**. The last figure is the one that matters, because scanning a directory is
  mostly files this table does not recognise.

* **Nothing about the answer changed.** Every input produces the same answer, and
  that is a property of the construction rather than of the tests: an entry is
  reachable through a bucket only if every offset it declares is `0`, so a file
  starting with any other byte cannot match it; everything else is tried for every
  input as before; and the two lists are merged by smallest table index, not by
  trying one and then the other. Three differential tests then check that against a
  separately written linear scan — the table's fixtures, non-matching inputs at every
  declared offset, and 4,096 pseudo-random buffers.

* **Both bindings now use it**, including the paths they used to reimplement.
  `which_kind_max_at` in `bindings/asm` and `match_within` in `bindings/python` each
  carried their own copy of the filtered scan, because the crate only compiles
  `match_with_max_read_rule` under `no_std`. They call `dispatch::first_match` now.

* **It costs bytes, and the two modules say very different numbers.** The npm
  binding's module is 43,212 -> 44,616 bytes, +1,404 or 3.2%. The library-only
  `examples/wasm_rust` module is 17,764 -> 32,847, +15,083 or 84.9%. The cost is
  code, not data, so a module that is nearly all code pays the most for it. Both
  numbers are in the readme and in `bindings/asm/Cargo.toml` rather than left for
  someone to discover.


## Unreleased: benchmarks against infer and libmagic

**A benchmark crate that measures this crate against the libraries a reader
would compare it to, and that is checked for lying.**

* **Added `benchmarks/`.** Its own workspace, on purpose: the readme's first table
  claims the root lockfile holds exactly one package, and criterion is a few hundred.
  `crates/magical_rs/tests/workspace.rs` now fails if `benchmarks` is ever dropped
  from `workspace.exclude`.

* **Three libraries on one corpus.** 286 buffers of 36,870 bytes — half with a
  format planted at a declared signature and offset, half matching nothing — plus the
  same 286 written to disk and opened by each library. The two entry points are
  separate tables, because the numbers are not close: from a file, `infer` is faster
  than this crate, since `magical_rs` reads 36,870 bytes to reach ISO 9660 and `infer`
  reads only what its own table needs. That row is in the report on purpose.

* **The answer table is not a score and says so.** The format cases are generated from
  this crate's own `SIGNATURE_KIND`, so a correctness figure over them would be
  measuring the generator. The report prints what each library said about the four
  real files the repository already commits. That split was found rather than
  designed: libmagic reports `application/octet-stream` for a buffer holding a valid
  eight-byte PNG signature and 36,862 zero bytes, and `image/png` for a real PNG
  whose first eight bytes are the same eight, because its rules look past the header.

* **libmagic is behind the crate's `libmagic` feature.** `magic-sys`'s build script
  tries `pkg-config` then `vcpkg` and fails rather than degrading, so a missing
  libmagic has to be a benchmark with one fewer row rather than a red build. The
  report prints which libraries ran, so a two-row table is never mistaken for a
  three-row one. `benchmarks/README.md` has the per-platform instructions.

* **Added `make bench`, `make bench-report` and `make bench-mutations`,** and
  `.github/workflows/benchmarks.yml`, which publishes the numbers to the job's step
  summary. Nothing gates on a number: a nanosecond figure on a shared runner is a
  property of the runner, and a gate built on one fails at random and gets muted.

* **`benchmarks/mutations.ps1` is the gate.** 22 changes to the harness, each one a
  way the report could flatter this crate or hide a library, each of which must turn a
  test red. All 22 are caught. Two of them are bugs this had while being written, and
  both are written up in the crate's readme: libmagic's pass over the 10.5 MB corpus
  evicted it from cache and charged the next library for it (this crate measured
  177 ns where criterion measured 57 ns for identical work), and a `NaN` median made
  every ratio in the table print as `inf`.

* **`tests/ci_coverage.rs` reads every workflow rather than `crate_dev.yml`.** The
  subject of that test is a `Makefile` target nothing runs, and `bench` is run by a
  workflow that did not exist when the test was written.
## Unreleased: one format dataset

**`@reim-developer/magical-js` can now answer what a format is called and served as.**

* **Added `displayName(kind)`, `mime(kind)` and `extension(kind)`.** The two
  questions detection cannot answer, because `detectBytes` reads bytes. This was
  the one parity gap left in the NodeJS binding: Python had
  `FileKind.description`, `.mime` and `.extension` since 0.4.0 and JavaScript had
  none of them, and its README said so under a heading called "Two things this
  binding does not do". It does not any more.
* **All three answer `null` rather than a guess,** for the same reason and with the
  same 16 formats the other bindings report: a MIME type is served to a browser,
  and an invented one is indistinguishable from a real one at the point it does
  harm. The `application/octet-stream` fallback is the caller's to write, where it
  is visible.
* **An unknown format name throws** — a `RangeError` for a string that is not one
  of the 114, a `TypeError` for a value that is not a string — rather than
  answering `undefined`, which would be ambiguous between "not a format we know"
  and "a format with no registered type". Only the first is a caller error.

**`formats.json` is now the only place a format's metadata is written down.**

The values were in three places and a value that appeared in two of them was
checked by a person:

* `mime`, `extension` and a short token were a PowerShell hashtable at the top of
  `scripts/gen_kinds.ps1`.
* Display names were the first column of the markdown tables in `readme.md`, which
  the generator parsed back out. That is a table with its own parser, and the
  parser is where the bugs would have been.
* The names and their order were parsed out of `pub enum FileKind`.

`formats.json` at the repository root holds all of it now, and two scripts generate
from it: `scripts/gen_kinds.ps1` for the Rust and Python metadata,
`scripts/gen_formats.mjs` for the JavaScript and TypeScript tables. JSON because
`JSON.parse` is in Node, `json` is in Python's standard library, and the Rust side
never parses it at all — it is generated *from* it, so `no_std` and the zero
dependencies are untouched.

Every field in the file is read by a generator and checked by a test. The magic
bytes are deliberately *not* in it: they are facts about the formats rather than
metadata about this project's table, they stay in the readme, and
`tests/readme_coverage.rs` checks them against the `SIGNATURE_KIND` code that
matches on them.

`abi_order` is the exception that proves the rule. It is the position of a variant
in the enum, which is a fact about code rather than about a format, so it was read
out of the declaration and written into the dataset rather than typed into it. It
is also the JavaScript side's ABI, so the generator refuses to emit anything if it
is not a permutation of `0..113`.

**What changed and what did not:**

* `_kinds.js` and `_kinds.d.ts` gained three tables and a fourth export. Every
  existing line of both files is byte-identical except the header comment.
* `kinds_meta.rs` and `magical_py/_kinds.py` are unchanged except their header
  comments. The dataset reproduces the old output exactly, which is the check that
  the migration moved a value rather than rewriting it.
* `scripts/gen_kinds.ps1` lost its hashtable and its readme parser. Its emitted
  text is the same.
* `bindings/nodejs/scripts/gen_kinds.mjs` is gone, replaced by
  `scripts/gen_formats.mjs`, which reads the dataset instead of the enum.
* New: `crates/magical_rs/tests/dataset.rs` (6 tests) checks the dataset against
  `SIGNATURE_KIND`, against the metadata generated from it, and against the
  readme. `bindings/nodejs/test/meta.test.js` (11 tests) checks the JavaScript
  tables and the three new functions. `bindings/nodejs/test/gen.test.js` (7 tests)
  checks the line-ending helpers, which exist because this repository is CRLF on a
  Windows checkout and LF in CI.
* Both binding workflows now watch `formats.json`, since editing it regenerates
  both bindings and would otherwise not run either gate.

* The NodeJS binding's Rust moved to **`bindings/asm`,** a crate of its own beside
  the npm package rather than inside it. `Cargo.toml`, `Cargo.lock` and `src/lib.rs`
  moved; the crate is now `magical-asm` — `publish = false` either way — and is
  still built as a `cdylib` plus an `rlib`, so its 19 ABI tests run with no wasm in
  the loop. Two directories and one npm package, because the two halves change for
  different reasons and are versioned apart: the crate says `0.1.0` and is published
  nowhere, while the package has its own version and its own release workflow. One
  crate for both would tie two version numbers together for no benefit. What they
  cannot be apart is the module and the loader that instantiates it, so
  `bindings/nodejs/scripts/build.mjs` builds the crate, copies `magical_js.wasm`
  in, and verifies it — that copy is the only thing crossing between the two
  directories.

  Two npm packages would be the opposite mistake, and it is worth saying why that
  was not done: the memory ABI is a contract between the loader and the module, and
  version skew there fails at `import()` inside a caller's project rather than at
  install time. One package means the two cannot be installed apart.

  `bindings/asm` joins `workspace.exclude`, which `tests/workspace.rs` checks in
  both directions — a new crate that is not listed fails to build on its own, and a
  listed path that no longer exists is the stale line nobody re-reads.
  `tests/lint_level.rs` follows the crate to its new path. The built module is
  byte-identical apart from the crate name that appears in its panic strings:
  43,212 bytes raw, 17 exports, 0 imports.

## `magical-py`: Version 0.4.0
**What has been changed:**

Level 1 could tell you what a file was and nothing else. Three of the crate's own
facts about the table were unreachable from Python, and one question could not be
asked at all. All of it is additive: nothing existing changed shape.

* **Added `describe(kind)`, `signature_table()`, `Signature` and
  `FileKind.rule`.** What a format is matched on, entry by entry: the bytes, the
  offsets they are compared at, and the read size the entry declares.
  `signature_table()` returns all 114 entries **in the order detection tries
  them**, so walking it and stopping at the first match reproduces `detect`
  exactly. That order is the answer to "why did my file come back as this", and
  it is why the list is not sorted.
* **Added `FileKind.matches(data)`,** the per-format predicate. `detect` stops at
  the first entry that matched, so a format whose magic is also another format's
  is unreachable through it. `Ktx`'s magic is a prefix of `Ktx2`'s, so a KTX2
  file is reported as KTX2 and is never reported as KTX.
  `FileKind.Ktx.matches(header)` answers `True`, which is a different and also
  true answer. `Qcow` is shadowed by `Qcow2` the same way; those two pairs are
  the only ones in the table, and a test walks all 114 entries to keep it so.
* **Added `read_limits()`, `ReadLimits` and `DEFAULT_MAX_BYTES_READ`.** The
  crate's `pub const` read sizes, which nothing in Python could read. The one
  that matters is the 2,048 floor, and the reason it exists: 113 of the 114
  entries declare 2,048 or more, so a smaller window drops nearly everything.
  `MP3` is the single exception, at 262, because a frame header ends well before
  the second sector.
* **Fixed a wrong constant in the crate's documentation.** `ISO_MAX_BYTES_READ`
  is 36,870, and `magic.rs` claimed 32,774. `max_bytes` takes the *largest*
  offset, and `ISO_OFFSETS` ends at 36,865 rather than the 32,769 it starts with
  — the comment had the first entry's arithmetic and was out by exactly one
  sector. Readme and CHANGELOG never carried the figure.

**A predicate entry reports no signatures, and that is the honest answer:**

`ScriptExecute` and `WEBP` are decided by Rust functions over the buffer, not by
a fixed pattern. `FileKind.ScriptExecute.rule.uses_predicate` is `True` and its
`signatures` is empty. Listing the bytes those files happen to start with would
be a claim about how the rule works, and it would be wrong: the shebang rule
requires a `/` later on the line, which is the whole reason it exists. Without
that check it claimed `#!AMR`, which is the literal magic of AMR audio.

**`matches` does not honour `max_bytes_read`, and that is deliberate:**

A per-format question has no window to declare. `detect_bytes(PNG,
max_bytes_read=1)` returns `None` because no rule's declared read size fits in
one byte, and `FileKind.Png.matches(PNG)` is `True`, because it asks only about
the comparison.

**`Signature` lives in `_kinds.py` next to the enum, not in `_signatures.py`:**

A `Signature` names a `FileKind`, and a `FileKind` has a `rule` that is a
`Signature`. Split across two modules they are mutually recursive, and the
deferred import that breaks the tie is flagged by pyright strict as an import
cycle. One module is the honest structure. For the same reason neither module
does `from . import _magical_rs`: that names the package, and the package is in
the middle of importing them.

## Version: 0.6.4 `Offset Arithmetic and a Wrong Constant`
**What has been changed:**

* **`Magic::matches` saturates its offset arithmetic.** `offset +
  signature.len()` was unchecked, and that sum is not a total function on
  `usize`. An offset near the maximum overflowed: a debug build panicked with
  `attempt to add with overflow` at `signatures.rs:126`, and a release build
  wrapped the end to a value below the offset's own start and then sliced a
  range that was not one. Both are aborts, and both happen for an input that
  should simply not match. This is the same fix `magic_custom.rs` took in 0.6.2,
  applied to the function that reaches it first. A saturated end is the largest
  number it can be, so the length check is false and the rule declines; no
  signature that could have fitted after the offset was in the buffer, so no
  match is lost.
* **Fixed a wrong constant in the documentation.** `magic.rs` documented
  `ISO_MAX_BYTES_READ` as `~32774` and told readers that ISO "requires
  `allowed_max_read >= 32774`". It is 36,870. `max_bytes` takes the largest
  offset, and `ISO_OFFSETS` ends at 36,865, not at the 32,769 it starts with.

**The overflow was not reachable through `SIGNATURE_KIND`, and the test does not
pretend otherwise:**

The table is a crate-authored `static` with small literal offsets, so no input
could reach it. The test builds a `Magic` directly with
`offsets: &[usize::MAX]`, because the arithmetic is the thing under test and the
table is not. Without the fix that panics in debug and indexes a reversed range
in release. A second assertion in the same test pins that a saturating sum
landing exactly on the end of a real buffer still matches, so the fix is not
"give up on large offsets".

This is the second time this crate has had the same unchecked addition in two
places. `magic_custom.rs` got the `saturating_add` in 0.6.2; `signatures.rs` had
the identical expression and did not. Miri is still not in CI, which is why this
took a manual audit to find rather than a failing build.

## `magical-py`: Version 0.3.0
**What has been changed:**

Level 1 could not say how much of a buffer it was entitled to look at, and it
could only be pointed at a path. Both were gaps in the crate's level 1 that the
binding had been carrying without noticing, because level 1 was treated as
finished.

* **Added `max_bytes_read` to `detect` and `detect_bytes`,** keyword-only and
  defaulting to `bytes_read()`. It is the crate's
  `FileKind::match_with_max_read_rule`: the table is narrowed to the rules whose
  own declared read size fits inside the window. Every rule declares at least
  2,048, so a window below that excludes `Png` as well. That is the trade being
  made and it is worth stating plainly — a smaller window is a cheaper read and a
  smaller set of formats reachable inside it.
* **Added `read_header(source, max_bytes=None)`,** which reads the leading bytes
  of a path or a stream and detects nothing. `bytes_read()` has always reported a
  figure that is only useful to a caller reading headers themselves; until now
  there was no supported way to be that caller, and the examples had to teach the
  `open`/`read`/`close` themselves.
* **`detect` now accepts anything open for binary reading,** not only a path. A
  file already open, a socket's `makefile`, a pipe and a `zipfile.ZipExtFile` all
  work, and nothing about the path form changed.

**Why the window is not a convenience:**

A 4 KB buffer cut from an ISO 9660 disc came back `None` from `detect_bytes`. A
bare `None` reads as "this is not an ISO", when the truth is that the magic sits
at offset 36,865 and was never inside the buffer. Two different situations, one
answer, and no way for the caller to tell them apart. Naming the window makes
the `None` a claim about the read rather than about the file, and a claim a
caller can act on — retry wider, or record the file as undetermined rather than
unknown. The crate has had the two functions for this since `0.6.0`; the binding
had neither, and the crate's copies are `no_std`-only, so the binding reproduces
the filter over the public `SIGNATURE_KIND` the same way it already reproduces
level 2's comparison in `signatures_match`.

**A bug in the crate that this found:**

`read_file_header` allocated `max_bytes` zeroed and returned the whole buffer
without truncating to the bytes it had actually read, so a file shorter than the
limit came back padded with invented zeros. 15 of the 142 signature entries end
in a zero, so the padding could complete a magic the file did not contain: a
one-byte file holding a newline was reported as `Pcx`, whose magic is `0A 00`,
while `detect_bytes` on that same byte said no match. Detection that disagrees
with itself depending on whether the caller had a path or a buffer is worse than
the missing read it looked like, so the crate is fixed in `0.6.3`. The binding
would have shipped those invented bytes to anyone who called `read_header`, which
is the other reason it could not be left alone.

**Not changed:**

* `FileKind` and every one of its 114 members. No member was added, removed or
  reordered, and no signature was touched.
* Levels 2, 3 and 4 are untouched, and `max_bytes_read` on a `MagicCustom`,
  `DynMagicCustom` or `AsyncDynMagic` rule is unchanged and still advisory. The
  new parameter is level 1 only, and it means the same thing the crate's
  `allowed_max_read` does rather than what the rule-level parameter means.
* Every new parameter has a default, so no existing call site changes behaviour.
* Level 5 is still not exposed, and `read_header` deliberately is not a
  translation of the crate's `read_file_header`: it also takes a file object,
  which has no path to hand over.

**Tests:** 209, up from 185. 21 of the new ones are in `tests/test_detect.py`,
covering the window, `read_header`, the non-path sources and the agreement
between a path and its own bytes; 3 are in `tests/test_readme.py` and execute
this release's documentation claims the way the existing ones do. The crate
gained 3, in `src/magical/bytes_read.rs`, all of which fail against `0.6.2` and
name `Some(Pcx)` for the one-byte file.


## `magical-py`: Version 0.2.0
**What has been changed:**

The Python bindings now carry the crate's custom detection levels. Level 1,
`detect` and `detect_bytes`, was already there; this adds levels 2, 3 and 4.

| Level | Rule | A match costs | Kinds | Nothing matches |
| ----- | ---- | ------------- | ----- | --------------- |
| 2 | `MagicCustom` | one Rust call | one type across the set | your `fallback` |
| 3 | `DynMagicCustom` | a trip into Python | any, mixed freely | `None` |
| 4 | `AsyncDynMagic` | an awaited call on your loop | any, mixed freely | `None` |

* Added `MagicCustom` and `MatchRules`, with `all`, `any` and `with_fn` to
  mirror the crate's `all_matches!`, `any_matches!` and `with_fn_matches!`, plus
  `match_types_custom` and `match_types_custom_all`.
* Added `DynMagicCustom` with `match_dyn_types` and `match_dyn_types_all`, for
  rules that are not known when the code is written.
* Added `AsyncDynMagic` with `match_async_dyn_types` and
  `match_async_dyn_types_all`. The two level 3 and level 4 pairs have different
  names because a single Python namespace cannot hold both, the way two Rust
  modules can.
* Added `Predicate` and `AsyncPredicate`, for annotating your own matchers.
* Added `signatures_match` to the compiled module, the one new Rust function.

**Three places this deliberately differs from the crate:**

* **Level 2 does not call `MagicCustom`.** That struct holds `&'static` slices,
  because a level 2 rule is meant to be a `static`. A rule assembled from
  Python data at run time would have to be `Box::leak`ed, and a process that
  builds rules in a loop would leak without bound. The comparison from the
  `CustomMatchRules::Default` arm is reproduced in Rust instead, with no
  allocation per rule, so level 2 still costs no Python call. `tests/magic_custom.rs`
  in the crate pins the original and `tests/test_levels.py` pins the copy, so a
  change to either side is a test failure.
* **Level 4 runs on your event loop.** The crate takes a closure returning a
  future and depends on no async runtime. A Python awaitable nearly always needs
  the caller's loop, so the matcher is awaited where you await it. Polling it
  from a worker thread would add a thread, a channel and a loop handle to do
  what `await rule.matches(data)` does in three characters, and would deadlock
  on any matcher that touches the loop. Level 4 therefore needs no feature flag
  and no extra dependency, unlike `cargo add magical_rs --features
  magical_async_dyn`.
* **Construction is validated more strictly.** The crate ignores a rule's
  signatures and offsets when `rules` is set, because its fields are `&'static`
  and it cannot report the mistake usefully. Here that raises `ValueError`, as
  does signatures without offsets or the reverse, and a negative offset or
  `max_bytes_read`. An empty rule with neither is still legal and still never
  matches, which is the crate's behaviour rather than an oversight: its arm is
  `signatures.any(offsets.any(...))`, so either side being empty means no.

**A bug in the crate that this found:**

`match_types_custom` computed `offset + signature.len()`, and a signature near
`usize::MAX` overflowed that. In a debug build it panicked; in a release build
it wrapped to a small number, so the length check passed and the slice index
went out of range. Both paths aborted the caller's process over an offset the
caller cannot have meant. The binding already saturated, because an offset
reaching it is a Python integer; the crate is fixed in `0.6.2` and both sides
now agree.

**Also:**

* A pre-made coroutine is not accepted as a level 4 matcher. A rule is matched
  against many buffers and a coroutine can only be awaited once, so accepting
  one would turn a reused rule into a rule that fails on its second call, with a
  message from the event loop that says nothing about the cause. The matcher is
  a callable returning an awaitable, which is what the annotation already said.
* A level 2 rule with signatures and no offsets is rejected rather than
  accepted-and-never-matching, because a caller who passed them meant to use
  them. The comparison underneath is unchanged, and its behaviour with an empty
  side is pinned through the primitive in both crates' test suites.

**Not changed:**

* `FileKind` and every one of its 114 members. No member was added, removed or
  reordered, and no signature was touched.
* `detect`, `detect_bytes`, `bytes_read` and `version`, including their
  signatures and their return values.

**Tests:** 185, up from 91. 86 of the new ones cover the levels case by case
and 8 check the claims this release's documentation makes, all built from
literal byte strings with no fixture file checked in. The package is checked
with `pyright` in strict mode at `pythonVersion` 3.8, so every annotation in
the new module is 3.8-compatible.


## Version: 0.1.3
**What has been changed:**
* Added examples of how to use `magical_rs`. 
* Specifically as follows:

| Use case           | Can be found at                        |
| ------------------ | -------------------------------------- |
| Basic usage        | [normal_usage](examples/normal_usage)  |
| Magic Custom usage | [magic_custom](examples/magic_custom/) |

* Added category `"no-std"` to [`[Cargo.toml]`](Cargo.toml)
* Changed repository URL of `magical_rs` in [`Cargo.toml`](Cargo.toml)
* Added examples to [`Cargo metadata`](Cargo.toml)

## Version: 0.2.0
**What has been changed:**
* Added methods to normalize and extends file matching in `CustomRulesMatches`
* Added documentation and test cases.
* Still retains backward compatibility for `no_std`.
* Added some macro to standardize the syntax sugar to make the API more friendly.
* Some examples of using the macros have also been added. Can be found at: [examples](examples).
* Some examples of using `DynMagic` have also been added. Can be found at: [examples](examples).
 
	**Bellow is a list of macros that have been added:**

---
  | Macro name         | Support `no_std`, backward compatibility? |
  | ------------------ | ----------------------------------------- |
  | `match_custom!`    | Yes                                       |
  | `magic_custom!`    | Yes                                       |
  | `with_fn_matches!` | Yes                                       |
  | `any_matches!`     | Yes                                       |
  | `all_matches!`     | Yes                                       |

---
* The list of supported file in `readme.md` will also synchronized.

  **Bellow is a list of the signature files have been added:**

  | Name                    | Signature                | Offset |
  | ----------------------- | ------------------------ | ------ |
  | VMDK File               | `0x4B, 0x44, 0x4D`       | `0`    |
  | Google Chrome Extension | `0x43, 0x72, 0x32, 0x34` | `0`    |

* Bellow is the development roadmap for version `0.2.0`:
  
| Name              | Description                                                             | Status |
| ----------------- | ----------------------------------------------------------------------- | ------ |
| `Macro Supported` | Allows the use of macros to sugar-syntaxize the API                     | [x]    |
| `MultipeFn`       | Support for multiple `OR`, `AND` type pointer function in `CustomMagic` | [x]    |

## Version: 0.2.1:
**What has been changed:**

* Fixed the documentation and added use for each module in [readme.md](readme.md)
* Fixed blank signatures & offsets blank in [magic_custom example](examples/magic_custom/src/v_2_0_0/magic_custom_macro.rs)

## Version: 0.3.0:
**What has been changed:**

* Added feature only avalable in version `0.3.0` of `magical_rs`: `AsyncDynMagic`
* Added documentation and usage warnings to [`lib.rs`](src/lib.rs) and [`readme.md`](readme.md)
* From this version onwards, `AsyncDynMagic` becomes an optional module. Cargo and flags are required to enable it:

```bash
cargo add magical_rs --features magical_async_dyn
```
* Of course, flag `magical_async_dyn` has also been added to [`Cargo.toml`](Cargo.toml)
* Instructions on how to use have also added at [`AsyncDynMagic Examples`](examples/async_dyn_magic)
* Current flags in version `0.3.0` can be used:

| Name                | Description                                                    | Cargo flag              |
| ------------------- | -------------------------------------------------------------- | ----------------------- |
| `magical_dyn`       | Unlock lvl 3 with file dection with infinite rules at run time | `magical_dyn`           |
| `magical_async_dyn` | Has all the features of level 3 but supports asynchronous      | `magical_async_dyn`     |
| `no_std`            | Used in non-std environments like kernel, emebedded            | `--no-default-features` |

## Version: 0.3.1, `Minor edits`
**What has been changed:**
* Minor edit in [`Cargo.toml`](Cargo.toml), added category slug `asynchronous`
* Edited some keywords related to the framework in [`Cargo.toml`](Cargo.toml)
* Changed the description of the framework to better identify it's purpose


## Version: 0.4.0 `Major API Update`
**What has been changed:**
* Added feature flag `unsafe_context` to [`Cargo.toml`](Cargo.toml)
* Release new features included in the module `magic_custom` is `WithUnsafeFn`
  - Test can be found at: [`here`](tests/unsafe.rs).
  - Documentation and instructions, security warnings have also added for `magic_custom` module.
  - This unsafe feature is only compiled and used when the `unsafe_context` flag is explicitly enabled via `Cargo`:
    ```bash
    cargo add magical_rs --features unsafe_context
    ```
  - This version also adds more documentation and warnings for features like `Default`, `WithFn`.
  - However, `no_std` support is still absolutely guaranteed.
  - Edited [`Makefile`](Makefile) rules, allowing testing with `unsafe_context` feature
  - Added example for using `unsafe_context` [`here`](examples/unsafe_context) and [`readme.md`](readme.md)
  - We do a plan to add bindings to Python. However, we can't show them yet. So, the `bindings` folder will be ignored by Git for now. [`.gitignore`](.gitignore)

## Version: 0.4.5 `Major API Update`
**What has been changed:**
* Added support for multiple unsafe function pointers. It will be disabled by default.
  - Only usable if feature flag is explicitly used by Cargo:
  - ```bash
    cargo add magical_rs --features unsafe_context
    ```
  - These new features will not affect `no_std`, and will still be supported.
  - Added testing for the above features. Can be found at [`test`](tests/unsafe.rs)
  - Samples for the above features:
  - `AllMatchesUnsafe`:
  - ```rust
    use core::slice;
    use magical_rs::magical::magic_custom::match_types_custom;
    use magical_rs::magical::magic_custom::{CustomMatchRules, MagicCustom};

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum MagicKind {
        MoeMoe,
        UnknownFallback,
    }

    fn is_shoujo_girl(data: *const ()) -> bool {
        unsafe {
            let slice_ptr = data.cast::<u8>();
            let slice = slice::from_raw_parts(slice_ptr, 100);

            slice.starts_with(b"MagicalGirl")
        }
    }

    fn is_not_shoujo_girl(data: *const ()) -> bool {
        unsafe {
            let slice_ptr = data.cast::<u8>();
            let slice = slice::from_raw_parts(slice_ptr, 100);

            !slice.starts_with(b"MagicalGirl")
        }
    }

    let rules: &[MagicCustom<MagicKind>] = &[MagicCustom {
        signatures: &[],
        offsets: &[],
        max_bytes_read: 200,
        kind: MagicKind::MoeMoe,
        rules: CustomMatchRules::AllMatchesUnsafe(&[is_shoujo_girl, is_not_shoujo_girl]),
    }];

    let result = match_types_custom(b"MagicalGirl", rules, MagicKind::UnknownFallback);

    assert_ne!(result, MagicKind::MoeMoe);
    assert_eq!(result, MagicKind::UnknownFallback);
    ```
  - `AnyMatchesUnsafe`:
  - ```rust
    use core::slice;
    use magical_rs::magical::magic_custom::match_types_custom;
    use magical_rs::magical::magic_custom::{CustomMatchRules, MagicCustom};

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum MagicKind {
        MoeMoe,
        UnknownFallback,
    }

    fn is_shoujo_girl(data: *const ()) -> bool {
        unsafe {
            let slice_ptr = data.cast::<u8>();
            let slice = slice::from_raw_parts(slice_ptr, 100);

            slice.starts_with(b"MagicalGirl")
        }
    }

    fn is_not_shoujo_girl(data: *const ()) -> bool {
        unsafe {
            let slice_ptr = data.cast::<u8>();
            let slice = slice::from_raw_parts(slice_ptr, 100);

            !slice.starts_with(b"MagicalGirl")
        }
    }

    let rules: &[MagicCustom<MagicKind>] = &[MagicCustom {
        signatures: &[],
        offsets: &[],
        max_bytes_read: 200,
        kind: MagicKind::MoeMoe,
        rules: CustomMatchRules::AnyMatchesUnsafe(&[is_shoujo_girl, is_not_shoujo_girl]),
    }];

    let result = match_types_custom(b"MagicalGirl", rules, MagicKind::UnknownFallback);

    assert_eq!(result, MagicKind::MoeMoe);
    assert_ne!(result, MagicKind::UnknownFallback);
    ```


## Version: 0.6.1 `Documentation and Test Coverage`

**No API change, and no format added or removed.** The signature table is
byte-for-byte the one in `0.6.0`, and the twelve public items are the same
twelve. This release exists because `0.6.0` shipped a crate that nothing in
the repository was checking, and a readme whose claims no test agreed with.

**What this fixes:**

* The readme is now the single source of the crate's documentation. `src/lib.rs`
  carries `#![doc = include_str!("../readme.md")]` instead of 242 lines of
  duplicated doc comment, so the two can no longer disagree. Nothing but
  documentation was removed from the crate.
* The format table is checked rather than asserted. `tests/table_size.rs`,
  `tests/signature_coverage.rs` and `tests/readme_coverage.rs` fail if the
  built-in table and the readme stop matching, and the nine two-byte-only
  signatures are listed and checked individually.
* The `no_std` claim is verified instead of asserted. `make test-nostd` runs
  the suite without `std` and cross compiles to `thumbv7em-none-eabi`. It
  excludes doctests deliberately, since the readme's quick start calls a
  `std`-gated function; the cross compile is the real gate.
* `crate_dev.yml` now runs on pull requests into `master`, not only into
  `dev`. It had not run at all on the code that shipped as `0.6.0`, because
  `dev` had not moved since the commit before it.

**Not part of this release:** the Python bindings are a separate package,
`magical-py`, and are not reachable through this crate.

## Version: 0.6.2 `Overflow Fix and Release Gates`

**No API change, and no format added or removed.** The signature table is
byte-for-byte the one in `0.6.1`, the twelve public items are the same twelve,
and detection results for any real file are unchanged. What follows is one bug
fix and three gates, none of which a caller can observe except the first.

**What this fixes:**

* `match_types_custom` no longer takes the caller's process down over an offset
  the caller cannot have meant. It computed `offset + signature.len()`, which is
  not a total function on `usize`: an offset near `usize::MAX` with any
  signature at all overflowed the sum. A debug build then panicked with
  `attempt to add with overflow`, and a release build wrapped the sum to a small
  number, so the length check passed and `&bytes[offset..offset_end]` indexed a
  range starting at the maximum and ending below its own start. Both are aborts,
  and both happen for an input that should simply not match. The addition
  saturates now, so an impossible offset compares as the large number it is,
  `bytes.len() >= offset_end` is false, and the rule declines. No signature that
  could not fit after the offset was ever in the buffer, so no match is lost.

  This was found while writing the Python binding, whose copy of the same
  comparison had always saturated because an offset reaching it is a Python
  integer and is not the crate's to trust. The two had quietly disagreed; they
  do not now, and `tests/magic_custom.rs` pins the same table of offsets from
  both sides.

* Formatting is gated. `make fmt` runs `cargo fmt --check` and `crate_dev.yml`
  runs it as its own step before the linter. Nothing ran rustfmt before this,
  which is why 17 diffs sat in `src/` and `tests/` through two releases; they
  are applied in this release. One of the 17 was not whitespace: `src/lib.rs`
  began with a UTF-8 BOM, and rustfmt removed it. `readme.md` never had one, so
  nothing was absorbed into the crate documentation and no doctest changed.

* The helper scripts moved to `scripts/`, so a reader looking for them finds them
  rather than finding them next to code they do not maintain. `gates.sh` and
  `gen_kinds.ps1` were both in `bindings/python/`. `gen_kinds.ps1` also had a bug
  the move exposed: `Set-Location` moves PowerShell's location but not the
  process working directory that .NET's `WriteAllText` resolves a relative path
  against, so the generator only ever worked when run from the repository root
  and otherwise failed after parsing the whole table. Both paths are absolute
  now.

* crates.io publishing is a workflow, `.github/workflows/publish_crate.yml`,
  rather than a command typed on one machine. It uses crates.io's trusted
  publishing, so no registry token is stored in this repository. It needs a
  one-time registration on the crate's crates.io settings page and the four
  fields for it are written at the top of the file.

**Tests:** 20 in `tests/magic_custom.rs`, up from 16, and the crate's own gates
are now the ones CI runs. `magical-py` 0.2.0 ships in the same tree and is
published separately, off a `py-v*` tag.


## Version: 0.6.3 `Header Padding Fix`
**What has been changed:**

`read_file_header` allocated a zeroed buffer of `max_bytes` and returned all of
it, so a file shorter than the limit came back padded with bytes that were never
in it.

* **Fixed.** The buffer is now truncated to the number of bytes actually read, so
  the returned vector is the file's own leading bytes and its length says how
  many there were. This is what the function's documentation already claimed.
* The short files above are now reported as no format at all, and a path and the
  same bytes agree. Before, one of the two ways of asking was wrong and which one
  depended on the argument.

**The bug that padding could cause:**

A signature is allowed to end in a zero, and 15 of the 142 entries in
`SIGNATURE_KIND` do. The invented zeros therefore completed magics the file did
not contain. A one-byte file holding a newline was reported as
`FileKind::Pcx` — whose magic is `0A 00` — and a three-byte file holding `II*` as
`FileKind::Tiff`, whose magic is `II*\0`. `FileKind::match_types` on those same
bytes said no match, so `read_file_header` followed by `match_types` and
`match_types` alone disagreed about the same file, and which one you got depended
on whether the caller had a path or a buffer.

The formats that could be reached this way are those whose magic ends in a zero:
`ICO`, `SQLite`, `RAR`, `TrueTypeFont`, `WindowImagingFormat`,
`CreativeVoiceFile`, `OpenGLIrisPerformer`, `Xz`, `Tiff`, `Pcx`, `Cursor` and
`WindowsShortcut`. A file cut short inside one of those was completed by the
padding rather than rejected, so for a file smaller than its magic a negative
result was not reliably negative. A magic that merely *begins* with a zero, of
which `WASM`, `JPEG2000`, `JpegXl` and `MpegProgramStream` are examples, cannot
be completed this way and was never at risk. The binding surfaced this as
`magical_py.detect` reporting a one-byte file as a `Pcx` while
`magical_py.detect_bytes` on the same byte returned `None`.

**Not changed:**

* No signature, no `FileKind` member, no feature flag and no public signature
  changed. `read_file_header` takes the same two arguments and returns the same
  type.
* A file at least `max_bytes` long is unaffected; it was never padded.
* `with_bytes_read()` and `bytes_read()` are untouched. What a caller should read
  is still 36,870 bytes.
* `FileKind::match_types` and the whole detection table are untouched. The
  padding only ever supplied zeros, so it could complete a magic but never
  prevent a real one from matching.

**Tests:** 3 added, in `src/magical/bytes_read.rs`. One asserts the header is
byte-for-byte the file's own, one that a short file and those same bytes agree on
the kind, and one that the three truncated magics match no format at all. All
three fail against `0.6.2`, and the failure names `Pcx` directly.

`magical-py` 0.3.0 ships in the same tree and is published separately, off a
`py-v*` tag. It is the release that carries the binding half of this work, and
the first that goes out through the Python workflow's trusted publisher rather
than a stored token.


## Version: 0.6.0 `Signature Tightening` and `Format Table Expansion`

**Breaking: three signatures changed.**
Detection results for the same bytes can differ from `0.5.x`. Review before upgrading.

| Format | `0.5.x` | `0.6.0` | Why |
| --- | --- | --- | --- |
| `FileKind::Bzip` | `BZ` | `BZh` | `BZ` is only a prefix. The bzip2 block header is `BZh`, so the old rule reported any file starting with those two letters as bzip2. |
| `FileKind::ScriptExecute` | `#!` | `#!` plus `/` on the same line | `#!` claimed every file starting with those bytes. A real shebang must name an interpreter by path. |
| `FileKind::Ply` | `ply` | `ply` plus a line break | `ply` claimed any text file starting with that word. The PLY spec puts a line ending straight after the keyword. |

**What this fixes:**
* An AMR audio file is now reported as [`FileKind::Amr`] instead of
  `FileKind::ScriptExecute`. The AMR header `#!AMR` has no path separator, so
  it no longer matches the narrowed shebang rule.
* Bzip2 detection no longer produces false positives on files that merely
  begin with the letters `BZ`.
* A `#` comment or `#include` line in a source file is no longer reported as a
  script.

**New public module:** `magical::ext_fn::shebang`, exposing `is_shebang`.
Entries that rely on a predicate rather than a byte signature are now
`ScriptExecute` and `WEBP`.

**Known limitation of the new shebang rule:** a relative interpreter name such
as `#!python` is not detected, because it contains no path separator. Such a
script is non-portable in practice. Previously it was also undetected for a
different reason, so no realistic script is lost.

**What was changed:**
* `magical_rs` is now licensed under the MIT License instead of the GNU General
  Public License v3.0.
* The built-in format table grew from 48 to 114 formats (142 distinct magic
  signatures), defined in the new `src/magical/signatures_ext.rs`.
* The 65 new rules are **appended** to `SIGNATURE_KIND`, never interleaved.
  Because `match_types` returns the first match, appending guarantees no
  pre-existing rule can be shadowed. Behaviour of the original 48 formats is
  bit-for-bit unchanged.
* `readme.md` was rewritten and is now the single source of crate
  documentation, pulled in by `#![doc = include_str!("../readme.md")]`.
  Previously the README and the crate docs were two separate copies that had
  drifted apart; the crate docs contained a doctest referencing `async_std`,
  which is not a dependency, so `cargo test --doc` was already failing on
  `master`.
* The 65 new rules are **appended** to `SIGNATURE_KIND`, never interleaved.
  Because `match_types` returns the first match, appending guarantees no
  pre-existing rule can be shadowed, apart from the three deliberate changes
  listed above.

**Bugs found and fixed in the documentation:**
* The old format table misdescribed 9 signatures, including XML (the docs
  claimed `<!DOCTYPE` was accepted; the code only matches `<?xml ` with a
  trailing space) and the environment module format (docs said
  `MODULE\0\0\0`; the code checks `#%Module`).
* The level 5 example in the old README did not compile. It referenced
  `CustomMatchRules::WithFnUnsafe`, which does not exist; the available
  variants are `AllMatchesUnsafe` and `AnyMatchesUnsafe`.
* `with_bytes_read()` returns 36,870 bytes, not 2,048, because ISO 9660 stores
  its magic at offset 36,865. This is now documented and covered by a test.

**New tests:**
* `tests/signature_coverage.rs` walks `SIGNATURE_KIND` and asserts every entry
  detects itself, that no signature matches at an undeclared offset, that
  padding is never misdetected, and that truncated input never panics. Any
  format added in the future is covered automatically.
* `tests/signature_tightening.rs` pins the `0.6.0` behaviour change. Each of the
  three tightened formats is tested both ways: real files are still detected,
  and the specific over-match the change was made to fix no longer happens.
* `tests/readme_coverage.rs` checks the README against the code in both
  directions. Every format the table can return must be named in the README,
  every `FileKind` the README advertises must be one the table can actually
  return, and the stated table size must match `SIGNATURE_KIND.len()`. A guard
  assertion fails the test if the README table layout changes and the parser
  silently stops matching anything.
* `tests/readme_examples.rs` compiles and runs every code example in the
  README.
* `tests/table_size.rs` reports the live table size.

**Formats evaluated and deliberately excluded:**
* 3D Studio Max, whose magic is byte-for-byte identical to BigTIFF.
* Text-based formats, which have no magic bytes.

**Why:**
* The previous GPL-3.0 license capped adoption. Many organizations block GPL-licensed
  dependencies in their build and security policy, which ruled out both corporate
  adoption and the use of `magical_rs` inside permissively licensed tooling.
* MIT removes that ceiling without changing a single line of library code.

**What this does not change:**
* `magical_rs` is a single-author project. No other contributor holds copyright,
  so no third-party consent was required for this relicense.
* Versions `0.4.5` and earlier were already distributed under the GPL. Those grants
  are permanent and cannot be revoked, for anyone who already received those
  versions. Anyone depending on `0.4.5` may continue to use it under GPL terms.
  Only versions from `0.5.0` onward are MIT-licensed.
* No API, behavior, feature flag, or `no_std` support was modified by this change.
  
