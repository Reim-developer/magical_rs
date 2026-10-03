# Detection levels

Five levels, cumulative in power and not in cost. Level 1 is almost always the
right answer; the rest exist for the cases where it is not.

| Level | Rules come from | Allocates | Needs `std` | Rust flag | Python | JavaScript |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | the built-in table | no | no | — | yes | yes |
| 2 | you, at compile time | no | no | — | yes | yes |
| 3 | you, at run time | yes | yes | `magical_dyn` | yes | **no** |
| 4 | you, awaited | yes | yes | `magical_async_dyn` | yes | **no** |
| 5 | raw pointers | no | yes | `unsafe_context` | **no** | **no** |

Read the "Why the bindings stop where they do" section at the bottom before
concluding a binding is missing something — two of the three gaps are structural.

---

## Level 1 — the built-in table

114 formats, no configuration. If your formats are ordinary files, this is the
level you want and the others are scope you do not need.

```rust
use magical_rs::magical::magic::FileKind;

let kind = FileKind::match_types(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
assert_eq!(kind, Some(FileKind::Png));
```

`None` means no rule matched, and nothing else. This level and level 2 both build
for `thumbv7em-none-eabi` with no feature flags at all.

### The order is not negotiable

`SIGNATURE_KIND` is neither alphabetical nor yours to rearrange. `ScriptExecute`
sits at 17 and `RAR` at 18 — swapped on purpose, because `#!AMR` is the literal
magic of AMR audio, and the shebang rule must be asked first or AMR files become
undetectable. Both bindings' tests pin that order, because the discriminants *are*
the ABI.

Two entries overlapping by design is not a bug to work around. `Ktx2` precedes
`Ktx` for the same reason: KTX1's magic is a prefix of KTX2's.

### What it costs

Detection is a first-byte index over that table, built during const evaluation:
**525 ns → 6.4 ns** for a file the table does not recognise, which is the case that
dominates a directory scan.

It answers exactly what a linear walk of the table answered, for every input. Both
halves of that sentence matter: the speed is in
`src/magical/dispatch.rs`, and the proof is differential tests against a separately
written scan — not the tests agreeing with the old code on the cases they happened
to try.

---

## Level 2 — custom rules at compile time

Your signatures, offsets and predicates, all resolved at compile time. `no_std`
compatible. Reach for this when your format is not exotic but is not in the table
either.

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

Predicates are plain `fn(&[u8]) -> bool`. Combine them with `all_matches!`,
`any_matches!` or `with_fn_matches!`.

### `magic_rules!` — a rule set as a table

The five-field `MagicCustom` literal asks for more than you usually have to say, so
`magic_rules!` is sugar for exactly that struct. It expands to `MagicCustom { .. }`
literals and nothing else, and it is **not** behind a flag: it adds no dependency,
allocates nothing, and emits no code unless invoked.

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
    (Kind::Unknown, b"Mahou"),
];

assert_eq!(
    match_types_custom(b"MagicalGirl", RULES, Kind::Unknown),
    Kind::MahouShoujo
);
```

The older `magic_custom!` spelling is still public and still compiles. Nothing
existing changed shape.

---

## Level 3 — rules decided at run time

When the rule itself is not known until run time, pass a closure.

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

Unlike the levels above, this **allocates and requires `std`**. The matcher must be
`Send + Sync + 'static`. The kind is a `dyn Any`, so `kind_downcast_ref::<T>()` is
how you get your own type back out — which is also why kinds may differ between
rules here and may not at level 2.

---

## Level 4 — asynchronous rules

For detection that has to await: a network lookup, a database, a service call.

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

`magical_rs` depends on no async runtime. The future is yours, to poll on whichever
executor you already use.

---

## Level 5 — raw pointers

```bash
cargo add magical_rs --features unsafe_context
```

Level 5 hands out `magic_rs` pointers and reads through them. It is `unsafe` by
definition, it is not exposed to Python at all, and it has no meaning across a
WebAssembly boundary — there is no `magic_rs` address space on the other side of an
instance. The Rust in `bindings/asm` does offer it to Rust callers, because that
crate builds as an `rlib` as well as a `cdylib`.

There is no reason to reach for it that is not "I profiled this and the bounds
check showed up".

---

## The same levels in Python

`magical_py` carries levels 1, 2, 3 and 4. Three things about them are not a
transliteration.

**Level 5 is absent** because it is `unsafe` by definition and there is no Python
counterpart worth shipping.

**Levels 3 and 4 need no flag and no dependency**, because they are Python and there
is nothing to switch on. Level 4 additionally runs on the caller's event loop rather
than being driven from Rust, since a Python awaitable usually needs the loop it was
called from.

**Level 2 is not the same level 2.** Rust's `MagicCustom` holds `&'static` slices, so
a rule assembled from Python data at run time could only be `Box::leak`ed — and a
process building rules in a loop would leak without bound. The binding reproduces
the comparison in Rust instead, and pins it to the same behaviour from both sides.

```python
import enum
from magical_py import MagicCustom, match_types_custom

class Kind(enum.Enum):
    """Yours. `kind` is carried through untouched, so this is your type."""
    Shoujo = "shoujo"
    Unknown = "unknown"

RULES = [
    MagicCustom(kind=Kind.Shoujo, signatures=[b"MagicalGirl"], offsets=[0]),
    MagicCustom(kind=Kind.Unknown, signatures=[b"Mahou"], offsets=[0]),
]

assert match_types_custom(b"MagicalGirl", RULES, Kind.Unknown) is Kind.Shoujo
assert match_types_custom(b"Mahou", RULES, Kind.Unknown) is Kind.Unknown
```

---

## The same levels in JavaScript

`@reim-developer/magical-js` carries levels 1 and 2, and the missing three are
missing **structurally**, not by omission.

**Levels 3 and 4 need a host function**, and a WebAssembly module can only call the
host by declaring an **import**. This module declares none, which is what lets it
load as `new WebAssembly.Instance(module, {})` with no generated glue file beside
it. One import would mean every caller supplies an import object. The binding stops
at two levels on purpose.

**Level 5 has no meaning at all**, for the address-space reason above.

The two predicates the table does have — `ScriptExecute` and `WEBP` — work, because
their predicates are compiled *into* the module and called from inside it. What
cannot cross the boundary is a predicate written in JavaScript.

### Why the Rust and the package are separate

They live beside each other rather than inside each other because they change for
different reasons and are versioned apart: the crate says `0.1.0` and is published
nowhere, while the npm package has its own version and its own release workflow. One
crate for both would tie two version numbers together for no benefit.

What they cannot be apart is the module and the loader that instantiates it, so
`bindings/nodejs/scripts/build.mjs` builds the crate, copies the artifact in, and
verifies it. That copy is the only thing crossing between the two directories.

---

## Choosing one

| Your situation | Level |
| --- | --- |
| Ordinary files | 1 |
| Your own format, known now, no allocation wanted | 2 |
| Rules from a config file or a database | 3 |
| The rule needs a network answer | 4 |
| None of the above | stop; you do not need a detection library's level 5 |

The most common mistake is reaching for 3 when 2 would do. Level 3 allocates, needs
`std`, and forces its matcher to be `Send + Sync + 'static`, all of which you pay to
avoid a `static` you could have written.

---

## Metadata across all three

`FileKind::display_name`, `FileKind::mime` and `FileKind::extension` in Rust;
`FileKind.description`, `.mime` and `.extension` in Python; `displayName`, `mime`
and `extension` in JavaScript — all answer from one table,
[`formats.json`](../formats.json) at the repository root. It is generated into each
language, so a format cannot be `image/png` in Rust and something else in JavaScript.

A format with no verified MIME type or extension answers `None`, `None` and `null`
respectively. **Nothing is invented**: a MIME type is served to a browser, and an
invented one is a wrong answer with a 200 status attached.

---

## Where to go next

- [Concepts](concepts.md) — table order, the read window, what `None` means
- [Across the three languages](across-languages.md) — the differences that would
  change your code while porting
- API reference: [Rust](api/rust.md) · [Python](api/python.md) ·
  [JavaScript](api/javascript.md)