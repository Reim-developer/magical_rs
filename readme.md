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
verified by anything.

| | | Checked by |
| --- | --- | --- |
| 0 dependencies | not "few" — `cargo tree --edges normal` prints this crate and nothing else, and the lockfile holds exactly one package | `Cargo.lock`; there is nothing to opt out of |
| 114 formats | pinned in every binding and every generated file | `tests/table_size.rs`, `tests/dataset.rs`, `bindings/nodejs/test/kinds.test.js` |
| 0 WebAssembly imports | measured on the built artifact, not assumed | `bindings/nodejs/scripts/build.mjs` fails the build otherwise |
| 17 exports, 43,212 bytes | measured at build time and printed | `bindings/nodejs/scripts/build.mjs` |
| Works on `wasm32` | compiled **and run**, from Rust, with no binding | `examples/wasm_rust/`, run by `make examples` |
| Works without `std` | built for `thumbv7em-none-eabi` | `make test-nostd` |
| Every format's own rule matches itself | all 114 entries, from the table | `tests/signature_coverage.rs` |
| No signature matches at an undeclared offset | all 114 entries | `tests/signature_coverage.rs` |
| Padding is never misdetected | every entry, at every declared length | `tests/signature_coverage.rs` |
| Truncated input never panics | every entry, at every truncation | `tests/signature_coverage.rs` |
| The readme's format table matches the code | both directions | `tests/readme_coverage.rs` |
| Every documented example compiles | the rustdoc tests run on this file | `cargo test --doc` |

What is **not** claimed: that it is faster than anything else, that it is
battle-tested, or that the format list is exhaustive. It is one author's crate with
one author's tests. The formats it deliberately does not detect are listed under
[Supported formats](#supported-formats), and the reasons are there too.

## The three bindings

All three detect the same 114 formats through the same detection table. The
metadata they answer from — a display name, a short token, a MIME type, an
extension — is generated from one file, [`formats.json`](formats.json), so a format
cannot be `image/png` in Rust and something else in Python.

| | Crate | `PyPI` | `npm` |
| --- | --- | --- | --- |
| Rust | `magical_rs` 0.6.4 | — | — |
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
as a 42 KB WebAssembly module with hand-written generics:

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

## Why it might fit your project

**Zero dependencies.** The `Cargo.toml` dependency list is empty. Nothing to
audit, nothing to keep up to date, no version conflicts in your tree.

**`no_std` support.** Levels 1 and 2 build for bare-metal targets. Verified
against `thumbv7em-none-eabi`.

**WebAssembly.** The 114-entry table is data, so it is close to incompressible
already and there is little left to optimise: a module exporting this crate at
`opt-level = "s"` measures **17,764 bytes with 4 exports** —
[`examples/wasm_rust`](examples/wasm_rust) builds exactly that and asserts the
answer for nine headers — and the npm binding's, which adds the encoded table, level
2 rules and a released ABI, measures **43,212 bytes with 17 exports**. Both declare
**zero imports**, which is what removes the glue file; `make examples` runs the
first and `bindings/nodejs/scripts/build.mjs` fails the build if either claim goes
false.

The profile's `opt-level` is `"s"`, and that is a middle default rather than a
measured optimum. [`scripts/wasm_sizes.mjs`](scripts/wasm_sizes.mjs) builds the
binding's module at every level and prints what it costs, because the figure this
paragraph used to quote — "243 bytes between `z` and `s`" — was wrong in both the
number and its direction: `"z"` is 183 bytes **larger**. The reason is the table. On
a module that is nearly all code, `examples/wasm_rust`, `"z"` saves 9,557 of 17,764
bytes. The spread on the shipped module is 2.4%.

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

## Feature flags

| Flag | Default | Effect |
| --- | --- | --- |
| `std` | yes | Enables `read_file_header` and filesystem I/O |
| `magical_dyn` | no | Level 3, runtime rules |
| `magical_async_dyn` | no | Level 4, async rules |
| `unsafe_context` | no | Level 5, raw pointer rules |

Building with `--no-default-features` gives a `no_std` library with only levels
1 and 2 available.

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

It is also the smallest honest comparison available: **17,764 bytes, 4 exports, 0
imports**, against the binding's **43,212 bytes, 17 exports, 0 imports**. The 25 KB
is what a *binding* is and a *library* is not — the encoded detection table, level 2
runtime rules, the introspection API and a released ABI. Neither number is a
benchmark and neither is better; they answer different questions.

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

Three tests exist because the layout above is a thing that can drift, and none of
the drift is caught by a build:

- `crates/magical_rs/tests/workspace.rs` checks that every crate under
  `bindings/` and `examples/` is named in `exclude`, and that each one reaches
  the library at its new home. An unlisted crate fails to build *on its own*,
  and not from `cargo test` at the root, which never looks at it.
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
