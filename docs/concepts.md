# Concepts

What a detection answer means, and the three parameters that change it. Read this
once and the reference pages will not need explaining.

## Detection is a table walk, and order is the answer

`magical_rs` holds 114 rules in one fixed order. A call walks them and returns the
**first** one whose bytes match. Two consequences follow, and both surprise people:

**A format late in the table is unreachable for any buffer an earlier rule also
matches.** `Ktx2` sits before `Ktx` precisely because KTX1's magic is a
byte-for-byte prefix of KTX2's; without that ordering a KTX2 file would be reported
as KTX, which is wrong, and no amount of asking the other way round fixes it.

### Three orders exist, and the two that matter are not the same sequence

This is worth being exact about, because two different lists are both called "the
order" of the table, they disagree, and picking the wrong one produces answers
that are wrong rather than answers that fail.

| | What it is | `Png` | `Ktx2` | `Ktx` |
| --- | --- | --- | --- | --- |
| **detection order** — index into `SIGNATURE_KIND` | the order a match walks | 0 | **52** | **53** |
| **the ABI** — `FileKind as isize` | the discriminant each binding exchanges | 0 | **63** | **62** |
| `kinds_meta::ALL_KINDS` | every format, sorted by variant name | 75 | 47 | 46 |

**Detection order is what decides an answer.** `dispatch::first_match` returns an
index into `SIGNATURE_KIND`, and `dispatch::probes_for` counts against it. `Ktx2`
sits at 52 and `Ktx` at 53 because KTX1's magic is a byte-for-byte prefix of
KTX2's: ask `Ktx` first and every KTX2 file is reported as KTX1, which is wrong
and no amount of asking more carefully fixes it.

**The ABI is what the bindings exchange, and it is a different sequence.**
`ScriptExecute` has discriminant 18 and `RAR` has 17, while in the table
`ScriptExecute` is at 17 and `RAR` at 18 — the two lists are not permutations of
one another in the same order, they are permutations of one another *differently*.
The enum is declared in table order, and the two exceptions are the pairs whose
correct detection order differs from their name order.

Nothing breaks from this, because nothing is supposed to convert between them.
`dispatch` returns a table index, and a caller that wants a `FileKind` indexes
`SIGNATURE_KIND` with it. A program that reads a discriminant and treats it as a
table index gets a different format and no warning.

**`ALL_KINDS` is sorted by name and matches neither.** It is the right list for
"what does this library support?" — it is what the documentation table and the
bindings' own metadata are generated from — and the wrong one for reproducing an
answer. `ALL_KINDS[47]` is `Ktx2`; `SIGNATURE_KIND[47]` is a different format
entirely.

```rust
use magical_rs::magical::kinds_meta::ALL_KINDS;
use magical_rs::magical::magic::FileKind;
use magical_rs::magical::signatures::SIGNATURE_KIND;

// `Png` is the first rule detection tries, the first discriminant, and nowhere
// near the first entry of `ALL_KINDS`.
assert_eq!(SIGNATURE_KIND.iter().position(|m| m.kind == FileKind::Png), Some(0));
assert_eq!(FileKind::Png as isize, 0);
assert_eq!(ALL_KINDS.iter().position(|k| *k == FileKind::Png), Some(75));

// The pair that shows detection order and ABI order disagreeing.
assert_eq!(SIGNATURE_KIND.iter().position(|m| m.kind == FileKind::Ktx2), Some(52));
assert_eq!(FileKind::Ktx2 as isize, 63);
```

## The read window

Every rule declares the number of leading bytes it needs — its `max_bytes_read`.
The furthest signature in the table is ISO 9660's, at offset **36,865**, so
`with_bytes_read()` is **36,870**.

Two different numbers are both called "the default", and confusing them is the
most common mistake with this library:

| | Value | What it is |
| --- | --- | --- |
| `with_bytes_read()` | 36,870 | The largest `max_bytes_read` in the table. Enough for anything |
| `DEFAULT_MAX_BYTES_READ` | 2,048 | What a single rule usually declares. Enough for 113 of 114 |

**Exactly one rule needs more than 2,048 bytes: ISO 9660**, and all three of its
offsets (32,769 / 34,817 / 36,865) are past that point. Reading 2,048 bytes instead
of 36,870 is measurably cheaper and costs you that one format.

The window is **a claim about how much of the file you read**, not a minimum the
buffer is checked against. Classifying a 100-byte file with a 2,048-byte window is
ordinary and correct.

### Narrowing the window is a decision, not an optimisation

```rust
use magical_rs::magical::dispatch::first_match;
use magical_rs::magical::magic::FileKind;
use magical_rs::magical::signatures::SIGNATURE_KIND;

let mut iso = [0u8; 40_000];
iso[32_769..32_774].copy_from_slice(b"CD001");

assert_eq!(first_match(&iso, 36_870).map(|i| SIGNATURE_KIND[i].kind),
           Some(FileKind::ISO));
// The same bytes, a 2,048-byte window. ISO's own read size does not fit, so it
// is never tried.
assert_eq!(first_match(&iso, 2_048), None);
```

`first_match` takes the window and returns the table index, because that is what
the two bindings call. `FileKind::match_with_max_read_rule` is the same question
answered more conveniently — but it exists **only without `std`**, because the
`std` build has a faster path for it and the bindings use `dispatch::first_match`
directly.

If your inputs are not ISOs, pass 2,048 and keep the difference. If they might be,
do not: a format that silently stops being detected is a worse bug than a slower
read.

## `None` means one thing, except when it does not

This is the sharpest edge in the library, so it is worth being blunt about.

**`match_types` returns `None` for exactly one reason:** no rule in the whole table
matched these bytes.

**`match_with_max_read_rule(bytes, window)` returns `None` for two reasons**, and
they are not distinguishable from the return value:

1. no rule whose read size fits `window` matched, or
2. a rule that would have matched needs bytes past `window` and so was never tried.

The second is a statement about the window, not about the file. If it is possible,
name the window so the caller can tell the two apart — which is what every binding
does by taking `max_bytes_read` as a parameter rather than fixing it.

In Python and JavaScript this is why `detect_bytes(data, max_bytes_read=2048)`
returning `None` is not the same claim as `detect_bytes(data)` returning `None`.
Both return `None`; only the first tells you it was asked a narrower question.

### A short file is not a padded one

A file shorter than the window comes back **short**, not zero-padded. This is not a
detail — a zero-padded buffer invents bytes, and a format is free to end its magic
with a zero. TIFF's is `II*\0` and PCX's is `0A 00`, so a three-byte file `II*`
read into a zero-filled 2,048-byte buffer completed its own signature and was
reported as a TIFF. `read_file_header` truncates, and `FileKind::match_types` on the
bytes it returns agrees with `FileKind::match_types` on the same bytes held in
memory.

## Detection reads bytes and never the name

The file name and its extension are never consulted. A `.jpg` containing PNG data
is `Png`. That is what you want from a detection library — a name is a claim by
whoever wrote it, and a magic number is a fact about the bytes — and it is why
`FileKind::extension()` is metadata *about the answer* rather than an input to it.

## Some rules are not fixed byte patterns

Two of the 114 read structure rather than a fixed prefix, and both are there
because the fixed pattern alone would be wrong:

- **`ScriptExecute`** (`#!`) requires a `/` later on the first line. `#!AMR` is the
  literal magic of AMR audio, so without the check the script rule claimed every
  AMR file and they became undetectable.
- **`WEBP`** requires the full `RIFF....WEBP` layout, not just `RIFF`.

## Some signatures are only two bytes

Nine formats match on a two-byte prefix and nothing more: `Arj`, `Bitmap`, `Gzip`,
`MP3`, `MSDOS`, `Pcx`, `Pickle`, `SerializedJavaData` and `Zlib`. Those are the
values the formats themselves specify, so a file starting with those bytes is
reported as that format even when it is not one. Each one's bytes are in the
[format table](../readme.md#supported-formats).

Treat detection as a strong hint rather than proof, and validate the result where a
wrong answer would be harmful.

## What a match costs

Detection is a table walk. It was a linear walk until a first-byte index was added,
which took an unrecognised file from 525 ns to 31 ns, and the entries whose
signatures sit past offset zero were then compared as one 64-bit integer rather
than through `memcmp`, which took it to 6.4 ns. A file the table does not
recognise is the common case in a directory scan, so it is the case that was worth
optimising. Numbers and method are in the
[repository readme](../readme.md#benchmarks).

## Reading a file at all

Reading is a separate concern from matching, and the two entry points differ in
what they promise.

| | Reads | Allocates | Notes |
| --- | --- | --- | --- |
| `read_file_header(path, limit)` | the first `limit` bytes | a fresh `Vec` per call | One file at a time |
| `read_file_header_into(path, limit, &mut buf)` | the same | none, if you pass a buffer | For scanning a directory |

`read_file_header_into` also **clears the buffer before opening the file**, so a
failed read leaves it empty rather than full of zeros. Zeros are not inert: `bytes[0]`
is a real byte and lands in a real bucket, so a caller that ignores the error could
be handed a format for a file that was never opened.