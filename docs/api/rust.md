# Rust API reference

Everything the crate exports at level 1 and 2, plus where levels 3 to 5 live. For
what the calls *mean* rather than what they are called, read
[Concepts](../concepts.md) first.

The crate is `no_std` with no dependencies. Everything marked `#[cfg(feature = "std")]`
disappears without that feature, and nothing else does.

## Install

```toml
[dependencies]
magical_rs = "0.6"
```

## Modules

| Module | Holds |
| --- | --- |
| `magical::magic` | `FileKind` and the three level 1 entry points |
| `magical::bytes_read` | The read window, and reading a file |
| `magical::kinds_meta` | `ALL_KINDS` — every format, **sorted by name**, plus the metadata accessors |
| `magical::dispatch` | `first_match`, the shared scan, and its index |
| `magical::magic_custom` | `MagicCustom`, `match_types_custom` (level 2) |
| `magical::rules_dsl` | `magic_rules!` (level 2) |
| `magical::signatures` | The rule table itself |
| `magical::match_rules` | `CustomMatchRules` (level 5) |
| `magical::dyn_magic` | `DynMagicCustom` (level 3) |
| `magical::async_dyn_magic` | `AsyncDynMagic` (level 4) |
| `magical::ext_fn::{shebang, webp}` | The two rules that read structure |
| `magical::fluent` | `Detect`, `DetectRules`. Behind `magical_fluent` |

## Level 1 — the built-in table

| Call | Returns | Notes |
| --- | --- | --- |
| `FileKind::match_types(&[u8])` | `Option<FileKind>` | First match in `SIGNATURE_KIND` order, no window filter |
| `FileKind::match_with_max_read_rule(&[u8], usize)` | `Option<FileKind>` | Only rules whose own `max_bytes_read` fits the window. **`no_std` only** |
| `FileKind::match_with_custom_max_read(&[u8], usize)` | `Option<FileKind>` | No per-rule filter, but a minimum buffer length. **`no_std` only** |
| `read_file_header(&str, usize)` | `Result<Vec<u8>, io::Error>` | `#[cfg(feature = "std")]`. The first `usize` bytes of a file |
| `read_file_header_into(&str, usize, &mut Vec<u8>)` | `Result<(), io::Error>` | `#[cfg(feature = "std")]`. The same, into a buffer you own |
| `with_bytes_read()` | `usize` | 36,870 — the window that reaches every format |
| `DEFAULT_MAX_BYTES_READ` | `usize` | 2,048 — what one rule usually declares. **Not the same number** |
| `dispatch::first_match(&[u8], usize)` | `Option<usize>` | The table index the three above share |
| `dispatch::probes_for(&[u8])` | `usize` | How many rules an input causes to be tried. For tests and benchmarks |

The two `no_std`-only entry points are not an oversight: the `std` build has a
faster path for the same question, and the bindings call `dispatch::first_match`
rather than depending on either.

```rust
use magical_rs::magical::bytes_read::{read_file_header_into, with_bytes_read};
use magical_rs::magical::magic::FileKind;

// Reading many files: allocate once, not once per file.
let mut buffer = Vec::new();
let mut found = 0;
for path in std::fs::read_dir(".")? {
    let path = path?.path();
    if read_file_header_into(path.to_str().unwrap(), with_bytes_read(), &mut buffer).is_ok() {
        found += usize::from(FileKind::match_types(&buffer).is_some());
    }
}
println!("{found} recognised");
# Ok::<(), std::io::Error>(())
```

`read_file_header_into` clears the buffer **before** opening the file, so a failed
read leaves it empty rather than full of invented zeros. `read_file_header` is the
same function with the allocation moved inside.

## Metadata on `FileKind`

| Call | Returns | Example |
| --- | --- | --- |
| `FileKind::display_name()` | `&'static str` | `"JPEG"` |
| `FileKind::mime()` | `Option<&'static str>` | `Some("image/jpeg")` |
| `FileKind::extension()` | `Option<&'static str>` | `Some("jpg")` |
| `FileKind::variant_name()` | `&'static str` | `"Jpg"` |
| `FileKind::from_name(&str)` | `Option<FileKind>` | The reverse, and what the bindings look names up with |

All four are `const fn`. `None` from `mime` or `extension` means "there is none
registered", never "unknown" — which would be a guess.

`kinds_meta::ALL_KINDS` is all 114, **sorted by variant name** — `SevenZip`,
`AceCompressed`, `Aiff` — which is neither the order detection walks nor the order
the bindings exchange. Use it to enumerate formats; do not use it to reproduce an
answer. [Concepts](../concepts.md#three-orders-exist-and-the-two-that-matter-are-not-the-same-sequence)
has the three orders and the pair that shows them disagreeing.

## Level 2 — custom rules at compile time

A rule set is a `&'static [MagicCustom<K>]` where `K` is any type you like that is
`Clone`. Order is the answer: the first rule that matches wins.

```rust
use magical_rs::magic_rules;
use magical_rs::magical::magic_custom::{MagicCustom, match_types_custom};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    MahouShoujo,
    Unknown,
}

static RULES: &[MagicCustom<Kind>] = magic_rules![
    (Kind::MahouShoujo, b"MagicalGirl"),
    // Several signatures, or a non-zero offset: the explicit arm, which names
    // the offsets and the read size instead of inferring them.
    (@bytes Kind::Unknown, [b"Magical", b"Mahou"], [0], 16),
];

assert_eq!(match_types_custom(b"MagicalGirl", RULES, Kind::Unknown), Kind::MahouShoujo);
assert_eq!(match_types_custom(b"Mahou", RULES, Kind::Unknown), Kind::MahouShoujo);
assert_eq!(match_types_custom(b"nothing", RULES, Kind::Unknown), Kind::Unknown);
```

`magic_rules!` is sugar for `MagicCustom` struct literals, and the older
`magic_custom!` spelling is still public. Neither is gated: a `macro_rules!` costs
nothing until invoked.

| Call | Returns |
| --- | --- |
| `magic_rules![ .. ]` | A `&'static [MagicCustom<K>]` from a table |
| `magic_custom!( .. )` | One `MagicCustom` from named fields |
| `match_types_custom(&[u8], rules, fallback)` | `K` — the first match, or `fallback` |
| `any_matches!` / `all_matches!` / `with_fn_matches!` | Sugar for the `CustomMatchRules` variants |

**There is deliberately no `detect_or`.** It was here first, documented as a
`map_or` that does not evaluate the fallback unless needed, and that was false: a
by-value argument is evaluated before the call whatever the callee does with it. The
lazy version is `match_types_custom(..)` plus `unwrap_or_else` at the call site,
which a caller can write themselves.

## The fluent spelling, behind a flag

```toml
magical_rs = { version = "0.6", features = ["magical_fluent"] }
```

```rust
use magical_rs::magical::fluent::Detect;
use magical_rs::magical::magic::FileKind;

assert_eq!(b"GIF89a".detect(), Some(FileKind::GIF));
assert!(b"GIF89a".is(FileKind::GIF));
assert!(b"GIF89a".is_any([FileKind::Png, FileKind::GIF]));
```

`Detect::is` is **not** `detect() == Some(kind)`. It asks whether that one format's
own rule matches, ignoring every rule that would have been tried first. `Ktx` is
why that matters — its signature is a prefix of `Ktx2`'s, so a KTX2 file is the
earlier entry as far as `detect` is concerned and still "yes, it is KTX" as far as
`is` is concerned.

The flag is because this puts a method on `[u8]`, a type the caller does not own.
Turning it off leaves no trait, no type and no module behind.

## Levels 3, 4 and 5

| Level | Type | Entry point | Flag |
| --- | --- | --- | --- |
| 3 | `DynMagicCustom` | `match_dyn_types`, `match_dyn_types_all` | `magical_dyn` |
| 4 | `AsyncDynMagic` | `AsyncDynMagic::new` | `magical_async_dyn` |
| 5 | `CustomMatchRules::AllMatchesUnsafe` | — | `unsafe_context` |

[Detection levels](../detection-levels.md) says what each costs and when to reach for
it.

## `no_std`

Levels 1 and 2 build for `thumbv7em-none-eabi` with no feature flags at all. The
crate has no `std` reference outside `#[cfg(feature = "std")]` items, which
`make test-nostd` checks by building the target rather than by reading a manifest.

## Feature flags

| Flag | Default | Effect |
| --- | --- | --- |
| `std` | yes | Enables `read_file_header` and filesystem I/O |
| `magical_fluent` | no | `bytes.detect()` and friends |
| `magical_dyn` | no | Level 3 |
| `magical_async_dyn` | no | Level 4 |
| `unsafe_context` | no | Level 5 |

`magical_fluent` is the only one that is not about a detection level, and
[the readme](../../readme.md#why-magical_fluent-is-the-one-flag-that-is-not-about-a-level)
says why.
