# Documentation

`magical_rs` is one crate and three bindings. These pages are the reference for
all four, and this file is the index.

Start with [Getting started](getting-started.md) if you have not used the library
yet. The rest assumes you have identified one file once.

## What is here

| Page | Read it when |
| --- | --- |
| [Getting started](getting-started.md) | You want to identify a file. Install, and the first call in all three languages |
| [Concepts](concepts.md) | You want to know what an answer *means* — table order, the read window, and why a `None` is not always the same `None` |
| [Detection levels](detection-levels.md) | The built-in table is not enough, and you are choosing between custom rules at compile time, at runtime, async, and raw pointers |
| [Rust API](api/rust.md) | You are writing Rust |
| [Python API](api/python.md) | You are writing Python |
| [JavaScript API](api/javascript.md) | You are writing JavaScript or TypeScript |
| [Across the three languages](across-languages.md) | You are porting, or you want to know why two of them do not offer something the third does |

The [supported formats table](../readme.md#supported-formats) lives in the
repository readme rather than here. It is generated from
[`formats.json`](../formats.json) and held against the compiled table in both
directions, so it belongs next to the claim it supports.

## What is checked, and what is not

This matters more than it usually does, because a reference that is quietly wrong
is worse than no reference, so it is worth being exact about which parts of these
pages are held to something and which are not.

**Checked.** Every Rust example in these pages is compiled and run by
`crates/magical_rs/tests/docs_examples.rs`, in the same way
`readme_examples.rs` holds the readme's. Every Python example is run by
`bindings/python/tests/test_docs.py` and every JavaScript one by
`bindings/nodejs/test/docs.test.js`. Every relative link resolves, by
`crates/magical_rs/tests/docs_links.rs`. The format list in the readme is held
against `SIGNATURE_KIND` and against `formats.json` in both directions by
`tests/readme_coverage.rs` and `tests/dataset.rs`.

Those four harnesses were not written before the pages. They exist because writing
the pages found real errors in them: a Rust example calling a function that is
`#[cfg(not(feature = "std"))]`, a `magic_rules!` entry using a macro arm that does
not exist, cross-links resolving a directory too high, and two JavaScript examples
whose answers were the wrong *case* — `"jpg"` and `"JPEG image"` where the module
returns `"Jpg"` and `"JPEG"`. None of those would have been caught by reading the
prose again, and the casing ones could not have been caught by a type checker
either, because a string that is not a `FileKind` is not a type error.

**Not checked.** That these pages list *every* public name. A function added to a
binding without a row here is a documentation gap, not a broken build. Where a
page claims completeness it says so and means it by that standard; where it does
not, it says that too.

**Deliberately excluded.** Anything version-specific is written as a number with
its reason rather than as a number alone, so a future bump is an edit in one place
instead of a silent drift. `docs/` has no test of its own checking that those
numbers are current — the readme's are, and these pages link to the same ones.

## Conventions used here

- **"Level N"** means detection level N. There are five, they are cumulative, and
  [detection-levels.md](detection-levels.md) says what each one costs.
- **The read window** is the number of leading bytes a rule is allowed to look at.
  It is the single most consequential parameter in the library and the one most
  often set wrongly. [Concepts](concepts.md#the-read-window) explains it.
- **`None` / `null` is not one thing.** From the unfiltered call it means exactly
  one thing: no rule matched. From a call that names a window it can *also* mean "no
  rule whose read size fits the window you named matched" — a statement about the
  window rather than about the file, and not distinguishable from the return value.
  [Concepts](concepts.md#none-means-one-thing-except-when-it-does-not) says which is
  which per call, and why every binding takes the window as a parameter rather than
  fixing it.
- Code examples use the real magic bytes rather than placeholders, so they can be
  pasted into a file and run.