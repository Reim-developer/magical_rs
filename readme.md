# `magical_rs`

[![crates.io](https://img.shields.io/crates/v/magical_rs)](https://crates.io/crates/magical_rs)
[![crates.io downloads](https://img.shields.io/crates/d/magical_rs)](https://crates.io/crates/magical_rs)
[![MSRV](https://img.shields.io/crates/msrv/magical_rs)](https://github.com/Reim-developer/magical_rs/blob/master/crates/magical_rs/Cargo.toml)
[![docs.rs](https://img.shields.io/badge/docs.rs-magical__rs-4d2e63?logo=docs.rs)](https://docs.rs/magical_rs)
[![CI](https://github.com/Reim-developer/magical_rs/actions/workflows/crate_dev.yml/badge.svg?branch=dev)](https://github.com/Reim-developer/magical_rs/actions/workflows/crate_dev.yml)
[![license](https://img.shields.io/github/license/license/reim-developer/magical_rs)](https://github.com/Reim-developer/magical_rs/blob/master/LICENSE)

Zero-dependency file type detection for Rust, with a customization layer that
stays out of your way until you need it.

`magical_rs` identifies files by their magic bytes. It has **no dependencies at
all** — not one — and it compiles for `no_std` targets, so it works in
embedded, kernel-adjacent, and WebAssembly builds where most detection crates
cannot go.

## What is claimed here, and what checks it

Every row below is a number or a property that a check in this repository holds. A
project that asserts its own claims in its test suite is unusual enough to be worth
showing rather than asserting, so the check is named. Nothing in this table is a
benchmark or a comparison: those would move with the machine and could not be
verified by anything, and there is a whole crate of them that is deliberately kept
out of this table — see [Benchmarks](#benchmarks).

| | | Checked by |
| --- | --- | --- |
| 0 dependencies | not "few" — `cargo tree --edges normal` prints this crate and nothing else, and the lockfile holds exactly one package | `Cargo.lock`; there is nothing to opt out of |
| 114 formats | pinned in every binding and every generated file | `tests/table_size.rs`, `tests/dataset.rs`, `bindings/nodejs/test/kinds.test.js` |
| 0 WebAssembly imports | measured on the built artifact, not assumed | `bindings/nodejs/scripts/build.mjs` fails the build otherwise |
| 17 exports, 44,616 bytes | measured at build time and printed | `bindings/nodejs/scripts/build.mjs` |
| Works on `wasm32` | compiled **and run**, from Rust, with no binding | `examples/wasm_rust/`, run by `make examples` |
| Works without `std` | built for `thumbv7em-none-eabi` | `make test-nostd` |
| Detection answers what the linear scan answered | every fixture, every non-matching input, 4,096 pseudo-random buffers | `src/magical/dispatch.rs` |
| Every format's own rule matches itself | all 114 entries, from the table | `tests/signature_coverage.rs` |
| No signature matches at an undeclared offset | all 114 entries | `tests/signature_coverage.rs` |
| Padding is never misdetected | every entry, at every declared length | `tests/signature_coverage.rs` |
| Truncated input never panics | every entry, at every truncation | `tests/signature_coverage.rs` |
| The readme's format table matches the code | both directions | `tests/readme_coverage.rs` |
| Every documented example compiles | the rustdoc tests run on this file | `cargo test --doc` |
| A gated feature leaves nothing behind | the crate root, read as text | `tests/fluent.rs` |
| The benchmark harness is not quietly flattering | 22 changes to it, each of which must turn a test red | `benchmarks/mutations.ps1` |

What is **not** claimed: that it is faster than anything else, that it is
battle-tested, or that the format list is exhaustive. It is one author's crate with
one author's tests. The formats it deliberately does not detect are listed under
[Supported formats](#supported-formats), and the reasons are there too.

**On speed specifically.** There is a [benchmark crate](benchmarks/) that measures
this crate against `infer` and libmagic on a shared corpus, and it is deliberately
not in the table above. A number of nanoseconds is a fact about the machine, so it
cannot be verified by anything in this repository and it does not belong next to
claims that can. What the benchmark crate *can* be held to is that it is not lying
about what it measured, and that is the last row: 22 mutations of the harness, each
one a way the report could flatter this crate, each caught by a test. The numbers
are published in the pull request that a benchmark workflow runs on, and they are
read there rather than copied here, where they would be a claim about a machine
nobody else has.

## The three bindings

All three detect the same 114 formats through the same detection table. The
metadata they answer from — a display name, a short token, a MIME type, an
extension — is generated from one file, [`formats.json`](formats.json), so a format
cannot be `image/png` in Rust and something else in Python.

| | Crate | `PyPI` | `npm` |
| --- | --- | --- | --- |
| Rust | `magical_rs` 0.6.5 | — | — |
| Python | — | `magical-py` 0.4.0 | — |
| JavaScript / TypeScript | — | — | `@reim-developer/magical-js` 0.1.0 |

[![PyPI](https://img.shields.io/pypi/v/magical-py)](https://pypi.org/project/magical-py/)
[![PyPI downloads/month](https://img.shields.io/pypi/dm/magical-py)](https://pypi.org/project/magical-py/)
[![npm](https://img.shields.io/npm/v/@reim-developer/magical-js)](https://www.npmjs.com/package/@reim-developer/magical-js)
[![npm downloads/month](https://img.shields.io/npm/dm/@reim-developer/magical-js)](https://www.npmjs.com/package/@reim-developer/magical-js)

```toml
[dependencies]
magical_rs = "0.6"
```

Not writing Rust? There are Python bindings, published as `magical-py`:

```bash
pip install magical-py
```

```python
from magical_py import FileKind, detect

kind = detect("photo.jpg")

kind is FileKind.Jpg      # a real enum member, not a string to parse
kind.mime                 # 'image/jpeg'
kind.extension            # 'jpg'
```

They detect exactly the 114 formats listed below, through the same detection
table, with no separate list to keep in step. See
[`bindings/python`](bindings/python).

Not writing Rust or Python? There are JavaScript and TypeScript bindings, shipped
as a 44.6 KB WebAssembly module with hand-written generics:

```bash
npm install @reim-developer/magical-js
```

```ts
import { detectPath, matchTypes } from "@reim-developer/magical-js";

detectPath("photo.png");                   // "Png", not a Promise

// `const` type parameter: the answer is the union of the kinds *you* declared.
const rules = [
  { kind: "Png", signatures: [PNG_BYTES], offsets: [0] },
  { kind: "GIF", signatures: [GIF_BYTES], offsets: [0] },
] as const;
const found = matchTypes(rules, bytes);    // "Png" | "GIF" | null
if (found === "Jpg") { }                   // error: "Jpg" is a real format, just not one of these two
```

The module declares no imports, so there is no glue file and no toolchain, and
`detectBytes` is synchronous end to end. See [`bindings/nodejs`](bindings/nodejs).

## Quick start

```rust,no_run
use magical_rs::magical::bytes_read::{read_file_header, with_bytes_read};
use magical_rs::magical::magic::FileKind;

fn main() -> Result<(), std::io::Error> {
    let header = read_file_header("photo.png", with_bytes_read())?;

    match FileKind::match_types(&header) {
        Some(kind) => println!("{kind:?}"),
        None => println!("unrecognized file type"),
    }

    Ok(())
}
```

The same call with the `magical_fluent` feature puts the data first:

```rust,no_run
#[cfg(feature = "magical_fluent")]
use magical_rs::magical::fluent::Detect;

#[cfg(feature = "magical_fluent")]
fn fluent() -> Result<(), std::io::Error> {
    use magical_rs::magical::bytes_read::{read_file_header, with_bytes_read};

    let header = read_file_header("photo.png", with_bytes_read())?;
    println!("{:?}", header.detect());
    Ok(())
}
```

## Why it might fit your project

**Zero dependencies.** The `Cargo.toml` dependency list is empty. Nothing to
audit, nothing to keep up to date, no version conflicts in your tree.

**`no_std` support.** Levels 1 and 2 build for bare-metal targets. Verified
against `thumbv7em-none-eabi`.

**WebAssembly.** The 114-entry table is data, so it is close to incompressible
already and there is little left to optimise: a module exporting this crate at
`opt-level = "s"` measures **32,847 bytes with 4 exports** -
[`examples/wasm_rust`](examples/wasm_rust) builds exactly that and asserts the
answer for nine headers - and the npm binding's, which adds the encoded table, level
2 rules and a released ABI, measures **44,616 bytes with 17 exports**. Both declare
**zero imports**, which is what removes the glue file; `make examples` runs the
first and `bindings/nodejs/scripts/build.mjs` fails the build if either claim goes
false.

The profile's `opt-level` is `"s"`, and that is a middle default rather than a
measured optimum. [`scripts/wasm_sizes.mjs`](scripts/wasm_sizes.mjs) builds the
binding's module at every level and prints what it costs, because the figure this
paragraph used to quote - "243 bytes between `z` and `s`" - was wrong in both the
number and its direction: `"z"` is 196 bytes **larger**. The reason is the table. The
spread on the shipped module is 2.3%.

Both the `std` build and the bare build compile for `wasm32-unknown-unknown`. The
dynamic levels are not part of that — 3 and 4 call back into a host language, which
in a browser would mean JavaScript and a different API. A `wasm-bindgen` wrapper
depends on `wasm-bindgen`, so the zero-dependency promise above is untouched.

**Detection is just bytes in, enum out.** `FileKind::match_types` takes a
`&[u8]` and returns `Option<FileKind>`. There is no handle, no session, no async
runtime requirement. First match wins, so it is allocation-free and
predictable.

**Custom rules compile-time or runtime, your choice.** Fixed signatures at
compile time, or closures that decide at runtime. See the levels below.

## Two things to know before you rely on it

**`with_bytes_read()` returns 36,870 bytes.** ISO 9660 stores its magic at
offset 36,865, so detecting it requires reading that far into the file. If you
only care about formats with magic at offset 0, pass a smaller value yourself
and skip ISO. Do not hardcode `2048` and assume you are done.

**Some signatures are still only two bytes.** 9 formats match on nothing but a
two-byte prefix: `Arj` on `60 EA`, `Bitmap` on `42 4D`, `Gzip` on `1F 8B`,
`MP3` on one of `FF FB` / `FF F3` / `FF F2`, `MSDOS` on `4D 5A`, `Pcx` on one of
`0A 00` / `0A 02` / `0A 03` / `0A 05`, `Pickle` on one of `80 02` through
`80 05`, `SerializedJavaData` on `AC ED`, and `Zlib` on one of `78 9C` /
`78 01` / `78 DA` / `78 5E`. Those are the values the formats themselves
specify, so there is nothing longer to match against. It does mean a file
beginning with those bytes is reported as that type, so treat detection as a
strong hint rather than proof, and validate the result if a wrong answer would
be harmful.

Two more are decided by a function rather than a fixed byte pattern, because
the byte pattern alone would be too broad: `ScriptExecute` requires a `/` later
on the first line, and `WEBP` requires the full `RIFF....WEBP` layout. Both are
listed with their `FileKind` names in the tables below.

Three signatures that were **wrong or far too broad** were tightened in
`0.6.0`, which is a breaking change:

| Format | Before | Now |
| --- | --- | --- |
| `Bzip` | `BZ` | `BZh`, the real bzip2 block header |
| `ScriptExecute` | `#!` | `#!` plus a path separator on the same line |
| `Ply` | `ply` | `ply` followed by a line break |

The old `Bzip` rule matched any file starting with the letters `BZ`, and the old
shebang rule matched any file starting `#!`, which included the `#!AMR` audio
header. Narrowing the shebang rule is what allowed `Amr` to be added.

## Detection levels

Five levels, ordered by how much control you give up. Start at level 1 and move
down only when you need to.

### Level 1 — built-in signatures

Uses the built-in table of 114 formats. No configuration.

```rust
use magical_rs::magical::magic::FileKind;

let kind = FileKind::match_types(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
assert_eq!(kind, Some(FileKind::Png));
```

Returns `None` when nothing matches. This level and level 2 both support
`no_std`.

**First match wins, and the table's order is the reason.** `SIGNATURE_KIND` is not
alphabetical and not negotiable: `ScriptExecute` sits at 17 and `RAR` at 18,
swapped, and both bindings' tests pin that order because the discriminants *are*
the ABI. Two entries overlap on purpose — a `#!` line is a shebang or the AMR
audio header, and the shebang rule wins because it is asked first.

Detection is a first-byte index over that table, built during const evaluation.
It costs 525 ns → 31 ns for a file the table does not recognise, and it answers
exactly what a walk of the table answered, for every input. Both halves of that
last sentence are worth reading: the speed is in
[`src/magical/dispatch.rs`](https://github.com/Reim-developer/magical_rs/blob/master/crates/magical_rs/src/magical/dispatch.rs),
and the proof is three differential tests against a separately written scan, not
the tests agreeing with the old code on the cases they happened to try.

### Level 2 — custom rules at compile time

Define your own signatures, offsets, and predicates. Predicates are plain
`fn(&[u8]) -> bool`, so the whole thing is resolved at compile time and works
under `no_std`.

```rust
use magical_rs::{all_matches, magic_custom, match_custom};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    CadFile,
    Fallback,
}

fn is_cad(bytes: &[u8]) -> bool {
    bytes.starts_with(b"ACAD")
}

fn is_short(bytes: &[u8]) -> bool {
    bytes.len() <= 4
}

let rule = magic_custom!(
    signatures: [b"ACAD"],
    offsets: [0],
    max_bytes_read: 2048,
    kind: Kind::CadFile,
    rules: all_matches!(is_cad, is_short)
);

let result = match_custom!(bytes: b"ACAD", rules: [rule], fallback: Kind::Fallback);

assert_eq!(result, Kind::CadFile);
```

Combine predicates with `all_matches!`, `any_matches!`, or `with_fn_matches!`.

#### `magic_rules!` — a rule set as a table

The five-field `MagicCustom` literal is the one place this crate asks for more than
it needs, so `magic_rules!` is sugar for exactly that struct. It expands to
`MagicCustom { .. }` literals and nothing else, and it is not behind a feature
flag: it adds no dependency, allocates nothing, and emits no code unless invoked.

```rust
use magical_rs::magic_rules;
use magical_rs::magical::magic_custom::{MagicCustom, match_types_custom};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    CadFile,
    ShortFile,
    Fallback,
}

fn is_cad(bytes: &[u8]) -> bool {
    bytes.starts_with(b"ACAD")
}

fn is_short(bytes: &[u8]) -> bool {
    bytes.len() <= 4
}

static RULES: &[MagicCustom<Kind>] = magic_rules![
    // A byte signature. `at` and `read` are both optional, and default to
    // `DEFAULT_OFFSET` (0) and `DEFAULT_MAX_BYTES_READ` (2,048).
    (Kind::CadFile, b"ACAD", read 2048),

    // Several signatures: the rule matches on any of them.
    (Kind::ShortFile, [b"<<", b">>"], read 4),

    // A predicate instead of bytes. The macro leaves `signatures` and `offsets`
    // empty, which is a step the literal makes easy to get wrong.
    (Kind::Fallback, via all [is_cad, is_short]),
];

let found = match_types_custom(b"ACAD", RULES, Kind::Fallback);
assert_eq!(found, Kind::CadFile);
```

The full syntax, and what each form expands to:

| Form | Means |
| --- | --- |
| `(Kind, b"SIG")` | one signature at offset 0 |
| `(Kind, [b"A", b"B"])` | any of these signatures, at offset 0 |
| `(Kind, b"SIG", at N)` | one signature at one offset |
| `(Kind, b"SIG", at [A, B])` | one signature at any of these offsets |
| `(Kind, [..], at [..], read N)` | the two nest |
| `(Kind, via PRED)` | decided by `fn(&[u8]) -> bool` |
| `(Kind, via any [P, ..])` / `via all [P, ..]` | one of / every one of |
| `(Kind, unsafe via [P, ..])` | the `unsafe_context` predicates |

Order is the answer, and the macro does not sort it. `match_types_custom` returns
the first rule that matches, so a predicate that matches everything placed first
means nothing after it can ever run.

`magic_custom!` and the bare struct literal both still work, and the tests in
[`tests/magic_rules.rs`](https://github.com/Reim-developer/magical_rs/blob/master/crates/magical_rs/tests/magic_rules.rs)
build every rule set twice — once through the macro, once as a literal — and
assert the two agree field by field. A `#[cfg]` on the module body rather than on
its `pub mod` line would leave a module that exists and holds nothing when the
feature is off, so the tests read the source to check the flag is where it should
be.

### Level 3 — rules decided at runtime

When the rule itself is not known until run time, pass a closure instead.

```bash
cargo add magical_rs --features magical_dyn
```

```rust
#[cfg(feature = "magical_dyn")]
use magical_rs::magical::dyn_magic::DynMagicCustom;

#[cfg(feature = "magical_dyn")]
fn example() {
    let rule = DynMagicCustom::new(
        |bytes: &[u8]| bytes.starts_with(b"MAGICAL"),
        String::from("custom format"),
        32,
    );

    assert!(rule.matches(b"MAGICAL"));
    assert_eq!(rule.kind_downcast_ref::<String>().unwrap(), "custom format");
}
```

Unlike the previous levels, this allocates and requires `std`. The matcher must
be `Send + Sync + 'static`.

### Level 4 — asynchronous rules

For detection that has to await something — a network lookup, a database, a
service call.

```bash
cargo add magical_rs --features magical_async_dyn
```

```rust
#[cfg(feature = "magical_async_dyn")]
use magical_rs::magical::async_dyn_magic::{match_dyn_types_as, AsyncDynMagic};

#[cfg(feature = "magical_async_dyn")]
async fn example() {
    let rule = AsyncDynMagic::new(
        |bytes: &[u8]| {
            // The future must be 'static, so it cannot borrow `bytes`.
            let owned = bytes.to_vec();
            async move { owned.starts_with(b"MAGICAL") }
        },
        String::from("custom format"),
        128,
    );

    let rules = [rule];
    let result = match_dyn_types_as::<String>(b"MAGICAL", &rules).await;
    assert_eq!(result.map(String::as_str), Some("custom format"));
}
```

`magical_rs` does not depend on any async runtime. The future you return is
yours to poll on whichever executor you already use.

Two constraints are easy to trip over here. The returned future must be
`'static`, so it cannot borrow the input slice — copy the bytes you need into
the async block first. And `match_dyn_types_as::<T>` returns `Option<&T>`, so a
`&'static str` kind must be downcast as `::<&str>`, which yields
`Option<&&str>`. Storing owned data in a `String` avoids that double reference.

### Level 5 — raw pointers

The escape hatch for kernel, embedded, or severely memory-constrained builds
where a `&[u8]` cannot be constructed. This level is `unsafe` by definition, and
enabling it makes the safety contract yours to uphold.

```bash
cargo add magical_rs --features unsafe_context
```

```rust
#[cfg(feature = "unsafe_context")]
fn example() {
    use core::slice;
    use magical_rs::magical::magic_custom::{match_types_custom, CustomMatchRules, MagicCustom};

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum Kind {
        Matched,
        Fallback,
    }

    const MAGIC_LEN: usize = 8;

    // The pointer is only valid for the number of bytes the caller
    // guarantees at the call site.
    unsafe fn starts_with_marker(ptr_data: *const ()) -> bool {
        // SAFETY: caller guarantees `ptr_data` is readable for `MAGIC_LEN` bytes.
        let ptr = ptr_data.cast::<u8>();
        unsafe { slice::from_raw_parts(ptr, MAGIC_LEN).starts_with(b"MAGICALG") }
    }

    let rules: &[MagicCustom<Kind>] = &[MagicCustom {
        signatures: &[],
        offsets: &[],
        max_bytes_read: 200,
        kind: Kind::Matched,
        rules: CustomMatchRules::AllMatchesUnsafe(&[starts_with_marker]),
    }];

    let result = match_types_custom(b"MAGICALG", rules, Kind::Fallback);
    assert_eq!(result, Kind::Matched);
}
```

The available predicates are `AllMatchesUnsafe` and `AnyMatchesUnsafe`, each
taking a slice of `unsafe fn(*const ()) -> bool`. There is no
`WithFnUnsafe` variant — the single-predicate case uses `AllMatchesUnsafe` with
one element. Multiple unsafe predicates have been supported since `0.4.5`.

### The same levels in `magical_py`

The Python bindings carry levels 1, 2, 3 and 4. Three things about them are not
just a transliteration of the above.

Level 5 is not there. It is `unsafe` by definition and there is no Python
counterpart worth shipping.

Levels 3 and 4 need no feature flag and no extra dependency, because they are
Python and there is nothing to switch on. Level 4 additionally runs on the
caller's event loop rather than being driven from Rust, since a Python
awaitable usually needs the loop it was called from.

Level 2 is not this level 2. `MagicCustom` holds `&'static` slices, so a rule
assembled from Python data at run time could only be `Box::leak`ed, and a
process that builds rules in a loop would leak without bound. The binding
reproduces the comparison in Rust instead, and pins it to the behaviour above
from both sides.

The [bindings README](https://github.com/Reim-developer/magical_rs/tree/master/bindings/python#detection-levels)
sets out what each level costs in Python and when to reach for it.

### The same levels in `@reim-developer/magical-js`

The `NodeJS` bindings carry levels 1 and 2, and the reason the other three are
missing is structural rather than a matter of effort: a level 3 or 4 predicate is
a *host* function, and a WebAssembly module can only call the host by declaring an
**import**. This module declares none, which is what lets it load with
`new WebAssembly.Instance(module, {})` and no generated glue file beside it. One
import would mean every caller supplies an import object, so the binding stops at
two levels on purpose rather than by omission.

Level 5 has no meaning at all: it hands out `magic_rs` pointers, and there is no
`magic_rs` address space on the other side of a `WebAssembly.Instance`. The Rust
in [`bindings/asm`](bindings/asm) does offer it, to Rust callers, because that
crate is built as an `rlib` as well as a `cdylib`.

It lives beside the package rather than inside it. The module's Rust and the
package's JavaScript change for different reasons and are versioned apart — the
crate says `0.1.0` and is published nowhere, while the npm package has its own
version and its own release workflow — and one crate for both would tie two version
numbers together for no benefit. What they cannot be apart is the module and the
loader that instantiates it, so `bindings/nodejs/scripts/build.mjs` builds the
crate, copies the artifact in, and verifies it; that copy is the only thing that
crosses between the two directories.

The two predicate formats the crate has — `ScriptExecute` and `WEBP` — are
reported and matched correctly, because their predicates are compiled *into* the
module and called from inside it. What cannot cross the boundary is a predicate
written in JavaScript.

### Metadata across all three

`FileKind::display_name`, `FileKind::mime` and `FileKind::extension` on the Rust
side, `FileKind.description`, `.mime` and `.extension` on the Python side, and
`displayName`, `mime` and `extension` on the JavaScript side, all answer from one
table: [`formats.json`](https://github.com/Reim-developer/magical_rs/blob/master/formats.json)
at the repository root. It is generated into each language, so a format cannot be
`image/png` in Rust and something else in JavaScript.

A format with no verified MIME type or extension answers `None`, `None` and `null`
respectively. Nothing is invented: a MIME type is served to a browser, and an
invented one is indistinguishable from a real one at the point it does harm. The
fallback — `application/octet-stream`, or no extension at all — is a decision the
caller makes where the decision is visible.

## API reference

Everything the three of them export, in one place, so you can find it without
reading the source. Each entry says what it does and which level it belongs to.

Nothing here is generated, and the readme does not claim to be complete against
the compiler. What *is* checked is the part that can be: `tests/readme_examples.rs`
compiles every Rust example on this page, and
`crates/magical_rs/tests/dataset.rs` holds the format names in the table below
against `SIGNATURE_KIND` and against `formats.json` in both directions. A function
added to a binding without a row here is a documentation gap, not a broken build —
which is the honest thing to say about it rather than implying a check that does
not exist.

### `magical_rs`

```toml
[dependencies]
magical_rs = "0.6"
```

| Call | Level | What it does |
| --- | --- | --- |
| `FileKind::match_types(&[u8])` | 1 | `Option<FileKind>` from the built-in table. First match in `SIGNATURE_KIND` order |
| `FileKind::match_with_max_read_rule(&[u8], usize)` | 1 | The same, restricted to rules whose own `max_bytes_read` fits the window. **`no_std` only** |
| `FileKind::match_with_custom_max_read(&[u8], usize)` | 1 | The same, with no per-rule filter but a minimum buffer length. **`no_std` only** |
| `read_file_header(path, limit)` | 1 | The `std` path: read the first `limit` bytes of a file. `read_file_header` is `#[cfg(feature = "std")]` |
| `with_bytes_read()` | 1 | The crate's own default window: 36,870 bytes, which is what ISO 9660 needs |
| `FileKind::display_name()` | — | `'static str`, e.g. `"JPEG image"` |
| `FileKind::mime()` | — | `Option<&'static str>`, e.g. `Some("image/jpeg")`. `None` where there is no verified type |
| `FileKind::extension()` | — | `Option<&'static str>`, e.g. `Some("jpg")` |
| `FileKind::variant_name()` | — | The token the other two languages use, e.g. `"Jpg"` |
| `FileKind::from_name(&str)` | — | The reverse of `variant_name`, and what the bindings' name lookup is |
| `kinds_meta::ALL_KINDS` | — | All 114, in table order. The order is the ABI every binding indexes by |
| `magic_rules![ .. ]` | 2 | A `&'static [MagicCustom<K>]` from a rule table. Not gated |
| `magic_custom!( .. )` | 2 | One `MagicCustom` from named fields. The older spelling; still public |
| `match_types_custom(&[u8], rules, fallback)` | 2 | The kind of the first rule that matches, or the fallback |
| `any_matches!` / `all_matches!` / `with_fn_matches!` | 2 | Sugar for the `CustomMatchRules` variants |
| `Detect::detect` and friends | 1, 2 | `bytes.detect()`. Behind `magical_fluent` |
| `dispatch::first_match(&[u8], usize)` | 1 | The index the three functions above share, for a caller who wants it directly |
| `dispatch::probes_for(&[u8])` | 1 | How many table entries an input causes to be tried. For tests and benchmarks |
| `DynMagicCustom::new` / `match_dyn_types*` | 3 | Runtime rules. Behind `magical_dyn` |
| `AsyncDynMagic::new` | 4 | Async rules. Behind `magical_async_dyn` |
| `CustomMatchRules::AllMatchesUnsafe` and friends | 5 | Raw-pointer rules. Behind `unsafe_context` |

Two of the level 1 entry points exist only without `std`, and that is not an
oversight — the `std` build has a faster path for the same question, and the
bindings call `dispatch::first_match` rather than depending on either. The
`no_std` restriction on `match_with_max_read_rule` and
`match_with_custom_max_read` is the reason
[`bindings/asm`](https://github.com/Reim-developer/magical_rs/tree/master/bindings/asm)
used to carry its own copy of that scan; it does not any more.

### `magical-py`

```bash
pip install magical-py
```

| Call | Level | What it does |
| --- | --- | --- |
| `detect(path)` | 1 | `FileKind \| None` from a path, with the header read for you |
| `detect_bytes(data, *, max_bytes_read=None)` | 1 | The same from bytes you already have |
| `bytes_read()` | 1 | 36,870 — the header size every format needs. Rust's `with_bytes_read()` |
| `DEFAULT_MAX_BYTES_READ` | 1 | 2,048 — the crate's per-rule default, which is *not* the same number |
| `read_header(source, limit=None)` | 1 | The first `limit` bytes, from a path or anything file-like |
| `version()` | — | The extension module's version, so a caller can branch on it |
| `FileKind.description` / `.mime` / `.extension` | — | The same three metadata calls, as properties. `None` means "none registered", never "unknown" |
| `FileKind` | — | A real `enum` of 114 members. Not a string to parse |
| `describe(kind)` | — | A `Signature`: the offsets, the bytes, and the read size for one format |
| `signature_table()` | — | All 114, in table order. A tuple, so it cannot be mutated |
| `read_limits()` | — | A `ReadLimits`: the smallest window that finds each format, and the crate's default |
| `MagicCustom` / `MatchRules` / `Predicate` | 2 | Rule sets, the same order-is-the-answer rule as Rust |
| `match_types_custom` | 2 | The kind of the first rule that matches, or your fallback |
| `match_types_custom_all` | 2 | Every rule that matches, not just the first |
| `DynMagicCustom` / `match_dyn_types` / `match_dyn_types_all` | 3 | Runtime rules |
| `AsyncDynMagic` / `AsyncPredicate` / `match_async_dyn_types*` | 4 | Async rules |

`detect` is deliberately blocking. It reads a header and compares bytes, and a
file read is a file read — making it `async` would mean either an executor this
crate does not have or a thread this crate should not start. For many files, read
them yourself and use `detect_bytes`. Level 4 is the async one, and it takes
*predicates*, not paths: the coroutine is yours, so a caller reads the file
wherever it likes.

### `@reim-developer/magical-js`

```bash
npm install @reim-developer/magical-js
```

| Call | Level | What it does |
| --- | --- | --- |
| `detectPath(path, options?)` | 1 | `FileKind \| null` from a path. Synchronous, not a `Promise` |
| `detectBytes(bytes, options?)` | 1 | The same from bytes you already have |
| `readHeader(path, options?)` | 1 | The header itself, if you want to keep it |
| `neededBytes()` | 1 | 36,870 — the header size every format needs to be distinguishable. Rust's `with_bytes_read()` |
| `DEFAULT_MAX_BYTES_READ` | 1 | 2,048 — the crate's per-rule default, which is *not* the same number. A caller who wants a narrow window wants this one |
| `readLimits()` | 1 | The smallest window that finds each format, and the default |
| `displayName(kind)` | — | e.g. `"JPEG image"` |
| `mime(kind)` | — | `string \| null` |
| `extension(kind)` | — | `string \| null` |
| `allKinds()` | — | All 114 tokens, in table order |
| `isFileKind(value)` | — | A type guard, for a name from a config file or a query string |
| `describe(kind)` | — | A `Signature` for one format |
| `signatureTable()` | — | All 114, in table order |
| `matches(kind, bytes)` | 1 | Is it *this* format, ignoring table order — the same question as Rust's `Detect::is` |
| `matchTypes(rules, bytes)` | 2 | The first rule that matches, with the answer typed as the union of the kinds *you* declared |
| `matchAllTypes(rules, bytes)` | 2 | Every rule that matches |
| `releaseRules(rules)` | 2 | Drops the compiled rule set's handle. `false` if there was nothing to release, so "already released" and "never compiled" are distinguishable | |

`matchTypes` is the piece worth understanding before using:

```ts
import { matchTypes } from "@reim-developer/magical-js";

// `const` type parameter: the answer is the union of the kinds you declared.
const rules = [
  { kind: "Png", signatures: [PNG_BYTES], offsets: [0] },
  { kind: "GIF", signatures: [GIF_BYTES], offsets: [0] },
] as const;

const found = matchTypes(rules, bytes);   // "Png" | "GIF" | null
if (found === "Jpg") { }                  // error: "Jpg" is a real format, just not one of these two
```

A rule set is a `readonly` array of plain objects, so it can be written in a JSON
file, sent over a message, or built at run time. It is compiled into the module's
linear memory on first use.

`releaseRules` drops the *handle*, not the memory: the compiled copy is leaked
once, deliberately, so the `MagicCustom` values inside the module can hold a
reference to it. That is a documented trade in `bindings/asm/src/lib.rs` and it is
repeated here because the function's name implies something stronger than it does.
A caller who builds rule sets once at startup never needs it. A caller building
them in a loop should call it when done with one, and should understand that the
wasm-side allocation is not returned to the allocator — the honest advice is to
build fewer, larger rule sets rather than many small ones.

### What is deliberately not the same across the three

Parity is a goal, not an accident, and the places where it is not are listed here
rather than left to be discovered.

| | Rust | Python | JavaScript |
| --- | --- | --- | --- |
| Unknown answer | `Option<FileKind>` | `FileKind \| None` | `FileKind \| null` |
| The kind itself | an enum | an enum | a `string` union — the discriminants *are* the ABI |
| Read size has a hard minimum | yes: ISO 9660 needs 36,870 bytes | same | same, and `readLimits()` reports it |
| A rule set lives in | the caller's binary | the caller's objects | compiled into the module, leaked once |
| Rule sets are mutable | no, `&'static` | the list is a `Sequence`, the caller owns it | no, `readonly`, and compiled on use |
| "Is it this format?" | `Detect::is` | none — add it in a `Predicate` | `matches(kind, bytes)` |
| `no_std` | yes, levels 1 and 2 | not applicable | not applicable — the module is `wasm32` |

The two rows that will surprise somebody:

**JavaScript has no `is_any`** and **Python has no `is` at all.** Both are one
line on top of what exists — Python writes a `Predicate` that looks the kind up,
JavaScript calls `matches` in a loop — and neither is in the binding because each
is a wrapper over something the language already has. A binding that ships a
one-line wrapper for every call site in Rust does not have parity, it has a bigger
API to keep in step.

**The rule-set lifetime differs**, and the reason is the boundary. In Rust and
Python the rules are yours and cost nothing to keep. In JavaScript they cross into
a module whose address space you do not own, so a loop that compiles a fresh rule
set per file grows that module's heap per file. `releaseRules` bounds the handles;
it does not return the memory.

## Feature flags

| Flag | Default | Effect |
| --- | --- | --- |
| `std` | yes | Enables `read_file_header` and filesystem I/O |
| `magical_dyn` | no | Level 3, runtime rules |
| `magical_async_dyn` | no | Level 4, async rules |
| `unsafe_context` | no | Level 5, raw pointer rules |
| `magical_fluent` | no | `bytes.detect()` and friends |

Building with `--no-default-features` gives a `no_std` library with only levels
1 and 2 available.

### Why `magical_fluent` is the one flag that is not about a level

Every other flag adds a capability. This one adds a spelling, and it is gated
because a spelling on a type you do not own is not free to take: `bytes.detect()`
is a method on `[u8]`, so writing it is a dependency on this crate for a slice of
bytes you could have handed to anything, and nothing in the signature lets you
see that afterwards. A `macro_rules!` costs nothing until invoked, so
`magic_rules!` is not gated; a trait method is a name in a namespace, so it is.

```rust
#[cfg(feature = "magical_fluent")]
use magical_rs::magical::fluent::Detect;

#[cfg(feature = "magical_fluent")]
fn example() {
    use magical_rs::magical::magic::FileKind;

    let gif = b"GIF89a";

    gif.detect();                             // Option<FileKind>
    gif.detect_within(2_048);                 // only rules that fit in 2,048 bytes
    gif.is(FileKind::GIF);                    // is it *this* format, ignoring table order
    gif.is_any([FileKind::Png, FileKind::GIF]);
}
```

Turning the flag off leaves nothing behind: no trait, no type, and not an empty
module in the documentation either. The `#[cfg]` is on the `pub mod` line rather
than only inside the module, and `tests/fluent.rs` reads the crate root to check
it — a `#[cfg]` inside the body compiles perfectly and leaves exactly the hole the
flag exists to close.

`is` is not the same question as `detect() == Some(kind)`, and the difference is
worth naming: `is` asks whether *that format's own rule* matches, ignoring every
rule that would have been tried first. A file that is both CBOR and, by its first
bytes, an earlier entry in the table is the earlier one as far as `detect` is
concerned and is still "yes, it is CBOR" as far as `is` is concerned.

There is deliberately no `detect_or`. `FileKind` has no "unknown" variant — all
114 of them are real formats — so there is no honest value to substitute, and an
`Option` is the better answer. `unwrap_or` on `detect()` is four characters if you
have your own sentinel.

## Supported formats

Magic bytes and offsets below are read directly from `SIGNATURE_KIND` in
[`crates/magical_rs/src/magical/signatures.rs`](https://github.com/Reim-developer/magical_rs/blob/master/crates/magical_rs/src/magical/signatures.rs).
"Offset" is the byte position the magic is compared at.

The format names in this section come from
[`formats.json`](https://github.com/Reim-developer/magical_rs/blob/master/formats.json),
which is the one place a format's display name, MIME type and extension is
written down — the same table that generates the Rust `FileKind::mime`, the
Python `FileKind.mime` and the JavaScript `mime()`.
[`crates/magical_rs/tests/dataset.rs`](https://github.com/Reim-developer/magical_rs/blob/master/crates/magical_rs/tests/dataset.rs)
checks the names below against it, so a format cannot be documented with one name
and detected with another.

The magic bytes are hand-written here rather than generated, because they are
facts about the formats rather than metadata about this project's table, and
[`crates/magical_rs/tests/readme_coverage.rs`](https://github.com/Reim-developer/magical_rs/blob/master/crates/magical_rs/tests/readme_coverage.rs)
checks them against the `SIGNATURE_KIND` code that matches on them.

| Format | `FileKind` | Magic | Offset |
| --- | --- | --- | --- |
| PNG | `Png` | `89 50 4E 47 0D 0A 1A 0A` | 0 |
| Java class | `Class` | `CA FE BA BE` | 0 |
| JPEG | `Jpg` | `FF D8 FF E0` | 0 |
| Gzip | `Gzip` | `1F 8B` | 0 |
| Bzip2 | `Bzip` | `42 5A 68` (`BZh`) | 0 |
| Zip / JAR / APK | `PkgZip` | `50 4B 03 04` | 0 |
| Bitmap | `Bitmap` | `42 4D` (`BM`) | 0 |
| DOS/PE executable | `MSDOS` | `4D 5A` (`MZ`) | 0 |
| Tar | `Tar` | `75 73 74 61 72` (`ustar`) | 257 |
| MP3 | `MP3` | `FF FB`, `FF F3`, or `FF F2` | 0 |
| ISO 9660 | `ISO` | `43 44 30 30 31` (`CD001`) | 32769, 34817, 36865 |
| RPM | `RPM` | `ED AB EE DB` | 0 |
| SQLite | `SQLite` | `SQLite format 3\0` | 0 |
| XML | `XML` | `3C 3F 78 6D 6C 20` (`<?xml `) | 0 |
| Windows icon | `ICO` | `00 00 01 00` | 0 |
| WebAssembly | `WASM` | `00 61 73 6D` (`\0asm`) | 0 |
| Debian package | `Deb` | `21 3C 61 72 63 68 3E 0A` (`!<arch>\n`) | 0 |
| Script / shebang | `ScriptExecute` | `23 21` plus a `/` on the same line | 0 |
| RAR | `RAR` | `52 61 72 21 1A 07 00` or `... 01 00` | 0 |
| ELF | `ELF` | `7F 45 4C 46` | 0 |
| Ogg | `OGG` | `4F 67 67 53` (`OggS`) | 0 |
| Photoshop PSD | `_8BPS` | `38 42 50 53` | 0 |
| Blender | `BLENDER` | `42 4C 45 4E 44 45 52` (`BLENDER`) | 0 |
| TrueType font | `TrueTypeFont` | `00 01 00 00 00` | 0 |
| OpenType font | `OpenTypeFont` | `4F 54 54 4F` (`OTTO`) | 0 |
| Environment module | `ModuleForEvenvironmentModules` | `23 25 4D 6F 64 75 6C 65` (`#%Module`) | 0 |
| Windows Imaging Format | `WindowImagingFormat` | `4D 53 57 49 4D 00 00 00 D0 ...` | 0 |
| `StarDict` binary | `Slob` | `21 2D 31 53 4C 4F 42 1F` | 0 |
| Java serialization | `SerializedJavaData` | `AC ED` | 0 |
| Creative Voice File | `CreativeVoiceFile` | `Creative Voice File\x1A\x1A\x00` | 0 |
| AU audio | `AuAudioFileFormat` | `2E 73 6E 64` (`.snd`) | 0 |
| OpenGL Iris Performer | `OpenGLIrisPerformer` | `DB 0A CE 00` | 0 |
| Noodlesoft Hazel | `NoodlesoftHazel` | `48 5A 4C 52 00 00 00 18` (`HZLR`) | 0 |
| Encoded `VBScript` | `VBScriptEncoded` | `23 40 7E 5E` (`#@~^`) | 0 |
| Apple icon | `AppleIconImage` | `69 63 6E 73` (`icns`) | 0 |
| GIF | `GIF` | `47 49 46 38 37 61` / `... 39 61` | 0 |
| JPEG 2000 | `JPEG2000` | `00 00 00 0C 0A 6A 50 20 20 0D 0A 87 0A` or `FF 4F FF 51` | 0 |
| PDF | `PDF` | `25 50 44 46 2D` (`%PDF-`) | 0 |
| Apple disk image | `AppleDiskImage` | `6B 6F 6C 79` (`koly`) | 0 |
| Cabinet | `Cabinet` | `4D 53 43 46` (`MSCF`) | 0 |
| Matroska | `MatroskaMediaContainer` | `1A 45 DF A3` | 0 |
| Rich Text Format | `RichTextFormat` | `7B 5C 72 74 66 31` (`{\rtf1`) | 0 |
| `PhotoCap` template | `PhotoCapTemplate` | `78 56 34` (`xV4`) | 0 |
| ACE archive | `AceCompressed` | `2A 2A 41 43 45 2A 2A` (`**ACE**`) | 0 |
| Flash video | `FlashVideo` | `46 4C 56` (`FLV`) | 0 |
| `VMware` disk image | `Vmdk` | `4B 44 4D` (`KDM`) | 0 |
| Chrome extension | `GoogleChromeExtension` | `43 72 32 34` (`Cr24`) | 0 |
| WebP | `WEBP` | RIFF container, validated by `is_webp` | 0 |

Two entries are not matched by a byte signature alone. WebP uses a dedicated
function in
[`crates/magical_rs/src/magical/ext_fn/webp.rs`](https://github.com/Reim-developer/magical_rs/blob/master/crates/magical_rs/src/magical/ext_fn/webp.rs)
because the format requires checking the `RIFF` header and the file size field
together. Shebangs use
[`crates/magical_rs/src/magical/ext_fn/shebang.rs`](https://github.com/Reim-developer/magical_rs/blob/master/crates/magical_rs/src/magical/ext_fn/shebang.rs)
to require a path separator, which is what separates `#!/bin/sh` from the
`#!AMR` audio header.

**Matching order matters.** `match_types` returns on the first match, so the
order of `SIGNATURE_KIND` decides which type wins when two signatures could both
apply. A JAR file is reported as `PkgZip`, not `Class`, because `PK` is tested
first. The 66 formats added after the original 48 are **appended** to the table
rather than interleaved, which guarantees they can never shadow an original
rule.

### Extended formats

Added in `0.5.0` and `0.6.0`, defined in
[`crates/magical_rs/src/magical/signatures_ext.rs`](https://github.com/Reim-developer/magical_rs/blob/master/crates/magical_rs/src/magical/signatures_ext.rs).

**Archives and compression**

| Format | `FileKind` | Magic | Offset |
| --- | --- | --- | --- |
| 7-Zip | `SevenZip` | `37 7A BC AF 27 1C` | 0 |
| XZ | `Xz` | `FD 37 7A 58 5A 00` | 0 |
| LZ4 frame | `Lz4` | `04 22 4D 18` | 0 |
| Zstandard | `Zstd` | `28 B5 2F FD` | 0 |
| LHA/LZH | `Lzh` | `2D 6C 68` | 2 |
| cpio | `Cpio` | `30 37 30 37 30` | 0 |
| ARJ | `Arj` | `60 EA` | 0 |
| `StuffIt` | `Stuffit` | `53 74 75 66 66 49 74` | 0 |
| `StuffIt` `.sit` | `StuffitSit` | `53 49 54 21` | 0 |
| PAR2 | `Par2` | `50 41 52 32 0A 50 4B 54` | 0 |
| zlib stream | `Zlib` | `78 01`, `78 5E`, `78 9C`, `78 DA` | 0 |

**Images**

| Format | `FileKind` | Magic | Offset |
| --- | --- | --- | --- |
| `TIFF` / `BigTIFF` | `Tiff` | `49 49 2A 00`, `4D 4D 00 2A`, `49 49 2B 00`, `4D 4D 2B 00` | 0 |
| PCX | `Pcx` | `0A 00`, `0A 02`, `0A 03`, `0A 05` | 0 |
| `DirectDraw` surface | `Dds` | `44 44 53 20` | 0 |
| KTX 2 | `Ktx2` | `AB 4B 54 58 20 32 30 BB 0D 0A 1A 0A` | 0 |
| KTX 1 | `Ktx` | `AB 4B 54 58 20` | 0 |
| `OpenEXR` | `OpenExr` | `76 2F 31 01` | 0 |
| Radiance HDR | `Radiance` | `23 3F 52 41 44 49 41 4E 43 45` | 0 |
| JPEG XL | `JpegXl` | `FF 0A` or `00 00 00 0C 4A 58 4C 20 0D 0A 87 0A` | 0 |
| Windows cursor | `Cursor` | `00 00 02 00` | 0 |
| GIMP XCF | `GimpXcf` | `67 69 6D 70` | 0 |
| FITS | `Fits` | `53 49 4D 50 4C 45 20 20` | 0 |

**Audio and video**

| Format | `FileKind` | Magic | Offset |
| --- | --- | --- | --- |
| MIDI | `Midi` | `4D 54 68 64` | 0 |
| AIFF | `Aiff` | `46 4F 52 4D` | 0 |
| FLAC | `Flac` | `66 4C 61 43` | 0 |
| `WavPack` | `WavPack` | `77 61 76 70` | 0 |
| Core Audio | `CoreAudio` | `63 61 66 66` | 0 |
| AMR audio | `Amr` | `23 21 41 4D 52` (`#!AMR`) or `23 21 41 4D 52 2D 57 50` | 0 |
| Monkey's Audio | `MonkeyAudio` | `4D 41 43 20` | 0 |
| ISO base media | `IsoMedia` | `66 74 79 70` (`ftyp`) | 4 |
| SWF | `Swf` | `46 57 53`, `43 57 53`, `5A 57 53` | 0 |
| ASF / WMV | `Asf` | `30 26 B2 75 8E 66 CF 11` | 0 |
| MPEG program stream | `MpegProgramStream` | `00 00 01 BA` | 0 |

**Documents**

| Format | `FileKind` | Magic | Offset |
| --- | --- | --- | --- |
| PostScript | `PostScript` | `25 21 50 53` | 0 |
| `DjVu` | `Djvu` | `41 54 26 54 26 46 4F 52 4D` | 0 |
| Mobipocket | `Mobipocket` | `42 4F 4F 4B 4D 4F 42 49` (`BOOKMOBI`) | 60 |
| MS Compiled HTML | `Chm` | `49 54 53 46` | 0 |
| OLE compound file | `OleCompoundFile` | `D0 CF 11 E0 A1 B1 1A E1` | 0 |

**Executables, byte code and fonts**

| Format | `FileKind` | Magic | Offset |
| --- | --- | --- | --- |
| Mach-O | `MachO` | `CE FA ED FE`, `CF FA ED FE`, `FE ED FA CE`, `FE ED FA CF` | 0 |
| Dalvik | `Dalvik` | `64 65 78 0A` | 0 |
| Lua byte code | `Lua` | `1B 4C 75 61` | 0 |
| Windows shortcut | `WindowsShortcut` | `4C 00 00 00 01 14 02 00` | 0 |
| WOFF | `Woff` | `77 4F 46 46` | 0 |
| WOFF2 | `Woff2` | `77 4F 46 32` | 0 |
| Font collection | `FontCollection` | `74 74 63 66` | 0 |

**Data, columnar and machine learning**

| Format | `FileKind` | Magic | Offset |
| --- | --- | --- | --- |
| `NumPy` `.npy` | `Numpy` | `93 4E 55 4D 50 59` | 0 |
| HDF5 | `Hdf5` | `89 48 44 46 0D 0A 1A 0A` | 0 |
| MATLAB | `Matlab` | `4D 41 54 4C 42 20 35 2E 30` | 0 |
| Parquet | `Parquet` | `50 41 52 31` | 0 |
| Apache ORC | `Orc` | `4F 52 43` | 0 |
| Avro | `Avro` | `4F 62 6A 01` | 0 |
| Binary plist | `BinaryPlist` | `62 70 6C 69 73 74` | 0 |
| Python pickle | `Pickle` | `80 02`, `80 03`, `80 04`, `80 05` | 0 |
| GGUF | `Gguf` | `47 47 55 46` | 0 |
| R serialized data | `RData` | `52 44 58 32`, `52 44 58 33` | 0 |

**3D, game data, disk images and network**

| Format | `FileKind` | Magic | Offset |
| --- | --- | --- | --- |
| glTF binary | `GltfBinary` | `67 6C 54 46` | 0 |
| FBX binary | `FbxBinary` | `4B 61 79 64 61 72 61 20 46 42 58 20` | 0 |
| Stanford PLY | `Ply` | `70 6C 79 0A` (`ply\n`) or `70 6C 79 0D 0A` | 0 |
| Doom WAD | `DoomWad` | `49 57 41 44` | 0 |
| QCOW2 | `Qcow2` | `51 46 49 FB` | 0 |
| QCOW | `Qcow` | `51 46 49` | 0 |
| `VirtualBox` `VDI` | `VirtualBoxVdi` | `3C 3C 3C 20 4F 72 61 63 6C 65` | 0 |
| Virtual HD | `VirtualHd` | `63 6F 6E 65 63 74 69 78` | 0 |
| pcap | `Pcap` | `D4 C3 B2 A1`, `A1 B2 C3 D4`, `4D 3C B2 A1`, `A1 B2 3C 4D` | 0 |
| pcapng | `PcapNg` | `0A 0D 0D 0A` | 0 |
| `BitTorrent` metainfo | `BitTorrent` | `64 38 3A 61 6E 6E 6F 75 6E 63 65` | 0 |

### Formats that cannot be detected reliably

These were evaluated and deliberately left out:

- **3D Studio Max** — its magic is `4D 4D 00 2A`, byte-for-byte identical to
  `BigTIFF`. No byte-level rule can tell them apart.
- **Text-based formats** — `.txt`, `.csv`, `.json`, `.html`, `.svg`, `.fasta`
  and similar have no magic bytes. Guessing at them produces false positives.

Missing a format? [Open a pull request](https://github.com/Reim-developer/magical_rs/pulls)
with the magic bytes and the format specification.

## Examples

Runnable examples live in
[`examples/`](https://github.com/Reim-developer/magical_rs/tree/master/examples):

| Directory | Shows |
| --- | --- |
| `normal_usage` | Level 1, reading a file header |
| `magic_custom` | Level 2, custom compile-time rules |
| `dyn_magic` | Level 3, runtime rules |
| `async_dyn_magic` | Level 4, async rules |
| `unsafe_context` | Level 5, raw pointer rules |
| `wasm_rust` | The crate on `wasm32`, from Rust, with no binding in between |

`make examples` runs all six, and runs them rather than merely building them: an
example that compiles and is wrong is invisible to a build, and the out-of-bounds
read in `unsafe_context` compiled cleanly for two years before anything ran it.

`wasm_rust` earns its place for a specific reason. `make build-wasm` proves the
crate *compiles* for `wasm32-unknown-unknown`, and its own comment in the `Makefile`
is careful about how little that means — a wasm32 `std` has a file system, sockets
and threads that all compile and then fail at runtime. What that build cannot check
is whether a built module answers correctly. `wasm_rust` does: it builds a
four-export module from this crate, instantiates it with an empty import object, and
asserts the answer for nine headers — including one with its magic at offset 257 and
one at offset 32,769 that forces the module's memory to grow. A wrong answer is a
non-zero exit.

It is also the smallest honest comparison available: **32,847 bytes, 4 exports, 0
imports**, against the binding's **44,616 bytes, 17 exports, 0 imports**. The 12 KB
is what a *binding* is and a *library* is not - the encoded detection table, level 2
runtime rules, the introspection API and a released ABI. Neither number is a
benchmark and neither is better; they answer different questions.

The two sizes also answer a question worth asking on its own. Detection used to walk
all 114 rules in order, which cost 525 ns for a file the table does not recognise
and 5 ns for one that matches the first rule, so a hundred-fold spread came from
position in the table alone. It is now a first-byte index built during const
evaluation - `crates/magical_rs/src/magical/dispatch.rs` - and that case is 31 ns,
GIF at position 36 is 21 ns rather than 123, and PNG is unchanged at 5. The index
answers exactly what the walk answered, for every input; that is not established by
the tests agreeing with the old code on the cases they happened to try, it is
established by construction and then checked by differential tests against a
separately written linear scan over the table, its fixtures, its non-matching
inputs, and 4,096 pseudo-random buffers.

It also cost 1,404 bytes in the binding and 15,083 in the example. The same code
weighing ten times as much tells you the cost is code and not data, and that a
module which is nearly all code - which is what a library-only module is - pays the
most for it. If you are short on bytes rather than nanoseconds, that number is the
one to weigh.

The [`magical_py`](https://github.com/Reim-developer/magical_rs/tree/master/bindings/python)
bindings carry their own six runnable scripts in
[`bindings/python/examples/`](https://github.com/Reim-developer/magical_rs/tree/master/bindings/python/examples),
covering the API that is specific to Python: detecting from a path, from
memory, walking a directory tree, and telling a `None` result apart from a
failure to read the file. Their
[detection levels](https://github.com/Reim-developer/magical_rs/tree/master/bindings/python#detection-levels)
section covers levels 2, 3 and 4, the custom rules, and what each one costs.

The [`@reim-developer/magical-js`](https://github.com/Reim-developer/magical_rs/tree/master/bindings/nodejs)
bindings have six runnable scripts in
[`bindings/nodejs/examples/`](https://github.com/Reim-developer/magical_rs/tree/master/bindings/nodejs/examples),
covering what a JavaScript caller actually needs shown and what four lines of API
cannot: `detectPath` and the metadata it cannot answer,
`maxBytesRead` and the one case where it changes *answers* rather than cost,
`allKinds` tallied by media type, walking a tree, level 2 rules, and what the
TypeScript declarations actually buy. Five are JavaScript and run on `node`; the
sixth is TypeScript, and running it only proves the stripping works — `npm run types`
is what makes it an example rather than a JavaScript file with annotations.

`bindings/nodejs/scripts/gates.sh` runs all six. Run rather than merely type-checked
or parsed, because `npm test` already imports every module the package ships: the
examples are the only files there that nothing else would execute, which is exactly
why one that had drifted would stay green until somebody read it.

The [workflow](.github/workflows/nodejs_bindings.yml)
packs the tarball and installs it into a throwaway project on Linux, macOS and
Windows, so the published shape is checked rather than assumed.

## Development

The repository is a cargo workspace. `crates/magical_rs` is the library this
readme documents, and everything under `bindings/` and `examples/` is outside it,
which is why the root manifest has an `exclude` list.

`bindings/` holds three crates that share no lockfile and no toolchain:

| Directory | What it is | Published as |
| --- | --- | --- |
| `bindings/python` | `PyO3` extension and `magical_py` | a wheel, on `PyPI` |
| `bindings/asm` | the `wasm32-unknown-unknown` `cdylib` | nothing — `publish = false` |
| `bindings/nodejs` | the loader, the API and the tests | `@reim-developer/magical-js` on npm |

`bindings/asm` is the module the npm package loads. It is a sibling of the package
rather than part of it: the two are versioned apart, and only
`bindings/nodejs/scripts/build.mjs` crosses between them, compiling the crate,
copying `magical_js.wasm` in and verifying it. Splitting them into two *npm
packages* would be a mistake for the opposite reason — the memory ABI is a contract
between the loader and the module, and version skew there fails at `import()` in a
caller's project rather than at install time.

```bash
cargo build
cargo test
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
```

Or through the `Makefile`, which is what CI runs, so a local run and a CI run
cannot disagree about what green means:

```bash
make test        # cargo test, over the whole workspace
make linter      # clippy with --all-targets --all-features
make fmt         # rustfmt --check
```

Verify `no_std` still holds:

```bash
cargo build -p magical_rs --no-default-features --target thumbv7em-none-eabi
```

And that the WebAssembly claim in the first paragraph still holds:

```bash
cargo build -p magical_rs --target wasm32-unknown-unknown
```

`make examples` runs every crate under `examples/`. They are run rather than
built because `cargo test` at the root does not reach them, and an example that
compiles and is wrong is invisible to a build.

The Python bindings have their own gate:

```bash
bash ./scripts/gates.sh
```

The `NodeJS` bindings have one too, which also needs
`rustup target add wasm32-unknown-unknown`:

```bash
bash ./bindings/nodejs/scripts/gates.sh
```

It regenerates the format tables from `formats.json`, runs the binding's own Rust
tests, builds the module and checks it declares no imports, type-checks the
hand-written declarations, runs the test suite, then runs rustfmt and clippy.

## Benchmarks

[`benchmarks/`](benchmarks/) measures this crate against `infer` and libmagic on
one shared corpus: 286 generated buffers of 36,870 bytes, half of them with a
format planted at a declared signature and offset and half matching nothing. It is
outside the root workspace so that Criterion does not put three hundred packages
into the lockfile the first table of this readme says holds exactly one.

```bash
make bench           # criterion per case, then the pull-request summary
make bench-report    # the summary alone, about ten seconds
make bench-mutations # the harness's own mutation check
```

The numbers are published in the pull request, not here, and this section does
not quote them. A nanosecond figure is a fact about the machine that produced it;
putting one in a readme turns it into a claim about every machine, and it would be
the first claim on this page that nothing in the repository could check.

Three things about how it measures, because all three are ways a benchmark of
this crate could have been quietly flattering to it:

**The corpus is generated from this crate's own signature table.** So of course it
recognises all of the format cases. A correctness score over that corpus would be
measuring the generator, so there is no score. What the report prints instead is
what each library said about the four real files the repository already commits,
which is a question that can be answered. That split was not designed in: libmagic
reports `application/octet-stream` for a buffer holding a valid eight-byte PNG
signature and 36,862 zero bytes, and `image/png` for a real PNG whose first eight
bytes are the same eight, because its rules look past the header.

**Both entry points are measured.** A caller has a buffer or has a path, and the
two numbers are not close. From a file, `magical_rs` reads 36,870 bytes because
that is what its table needs in order to reach ISO 9660, and `infer` reads only
what its own table needs — so `infer` is the faster of the two from a path, and
that row is in the report.

**The harness is mutation-checked.** `make bench-mutations` applies 22 changes to
it, each one a way the report could flatter this crate or hide a library, and
requires a test to go red for each. All 22 are caught. Two of the twenty-two are
bugs this crate actually had: libmagic's pass evicted the corpus from cache and
charged the fast library for it, and a `NaN` median made every ratio in the table
print as `inf`. Both are written up in [`benchmarks/README.md`](benchmarks/README.md).

libmagic is behind the crate's `libmagic` feature because it is a C library and
`magic-sys`'s build script fails rather than degrading when it cannot find one. The
report says which libraries actually ran, so a two-row table is never mistaken for
a three-row one.

Three tests exist because the layout above is a thing that can drift, and none of
the drift is caught by a build:

- `crates/magical_rs/tests/workspace.rs` checks that every crate under
  `benchmarks/`, `bindings/` and `examples/` is named in `exclude`, and that each
  one reaches the library at its new home. An unlisted crate fails to build *on its
  own*, and not from `cargo test` at the root, which never looks at it — so the
  benchmark crate is in that test for the same reason the bindings are: drop it
  from the list and `cargo bench` stops working with an error that names the root
  manifest rather than the line that is wrong.
- `crates/magical_rs/tests/packaging.rs` checks the copies of `LICENSE` and
  `CHANGELOG.md` that sit beside the crate's manifest. Cargo will not package a
  file from outside the package — `include = ["../../LICENSE"]` is silently
  ignored, while `readme = "../../readme.md"` works — so those two are copied
  and this test is what keeps the copies honest.
- `crates/magical_rs/tests/dataset.rs` checks `formats.json` against this readme,
  against `SIGNATURE_KIND` and against the metadata generated from it.

## One dataset, three languages

`formats.json` at the repository root is the only place a format's display name,
short token, MIME type and extension are written down. It used to be three: a
hashtable in `scripts/gen_kinds.ps1`, a column of markdown tables in this readme,
and a name list parsed out of `pub enum FileKind`. A value that appeared in two of
them was checked by hand, which is a check nobody performs.

Two scripts read it, and neither is a third source of truth:

| Script | Writes | Run by |
| --- | --- | --- |
| `scripts/gen_kinds.ps1` | `kinds_meta.rs`, `magical_py/_kinds.py` | `npm run gen` in the Python binding, by hand otherwise |
| `scripts/gen_formats.mjs` | `_kinds.js`, `_kinds.d.ts` | `npm run gen` in the `NodeJS` binding, and its gate |

`abi_order` is the exception that proves the rule. It is the position of a variant
in `pub enum FileKind`, which is a fact about code rather than about a format, so it
is *read out of the declaration* by `scripts/build_formats.mjs` rather than typed
into the dataset by hand. It is also the JavaScript side's ABI — a kind crosses the
wasm boundary as its Rust discriminant — and `test/kinds.test.js` checks all 114
against what the compiled module reports.

The magic bytes stay in this readme rather than moving into the dataset, because
they are facts about the formats rather than metadata about this project's table.
`tests/readme_coverage.rs` checks them against the `SIGNATURE_KIND` code that
matches on them, so the bytes are verified against the matcher and the names
against the dataset. `tests/dataset.rs` then checks the two against each other, so
a format cannot be documented with one name and detected with another.


## License

MIT — see [LICENSE](https://github.com/Reim-developer/magical_rs/blob/master/LICENSE).

Versions `0.4.5` and earlier were published under the GNU General Public
License v3.0. Those grants are permanent and remain in force for anyone who
already received those versions. Only `0.5.0` and later are MIT-licensed.
