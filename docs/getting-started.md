# Getting started

Install, and identify one file. Every example here is real and runs; the tests in
`crates/magical_rs/tests/docs_examples.rs` and the bindings' own example suites
execute them.

## Install

| | Command |
| --- | --- |
| Rust | `cargo add magical_rs` |
| Python | `pip install magical-py` |
| JavaScript | `npm install @reim-developer/magical-js` |

The Rust crate has **no dependencies** — `cargo tree --edges normal` prints itself
and nothing else, and the lockfile holds exactly one package.

## Rust

```rust
use magical_rs::magical::bytes_read::read_file_header;
use magical_rs::magical::magic::FileKind;
use magical_rs::magical::bytes_read::with_bytes_read;

fn main() -> std::io::Result<()> {
    // `with_bytes_read()` is the window that reaches every format in the table.
    let header = read_file_header("photo.jpg", with_bytes_read())?;

    match FileKind::match_types(&header) {
        Some(kind) => {
            println!("{:?}", kind.variant_name());   // "Jpg"
            println!("{:?}", kind.mime());           // Some("image/jpeg")
            println!("{:?}", kind.extension());      // Some("jpg")
            println!("{}", kind.display_name());      // "JPEG"
        }
        None => println!("not a format this table knows"),
    }
    Ok(())
}
```

`match_types` allocates nothing. It reads an index, indexes the table, and copies
out a `FileKind`.

## Python

```python
from magical_py import FileKind, detect

kind = detect("photo.jpg")

kind is FileKind.Jpg      # True, a real enum member your editor can complete
kind.value                 # 'jpg', a stable id for serialising
kind.mime                  # 'image/jpeg'
kind.extension             # 'jpg'
kind.description           # 'JPEG'
```

`detect` takes a path or anything open for binary reading, so a socket, a pipe and
a `zipfile.ZipExtFile` work as well as a file on disk.

## JavaScript

```js
import { detectPath, displayName, mime, extension } from "@reim-developer/magical-js";

const kind = detectPath("photo.jpg");   // "Jpg" — a string union, or null

if (kind !== null) {
  displayName(kind);   // "JPEG"
  mime(kind);          // "image/jpeg"
  extension(kind);     // "jpg"
}
```

Everything is synchronous. `detectPath` returns the kind, not a `Promise` — the
module is compiled from Rust and offers no async surface, which is stated plainly
rather than papered over with an awaitable that resolves immediately.

## The data-first spelling, for scanning a directory

`detect` puts the function first, which is right for a question asked once. A scan
asks about every file and mostly the answer is no, so there the file goes first.

**Python** has this directly:

```python
from magical_py import FileKind, detected

for path in directory.iterdir():
    what = detected(path)
    if what.matches_any(FileKind.Png, FileKind.GIF, FileKind.Jpg):
        ...
```

Nothing is read until a method needs an answer, and the bytes are read once however
many methods you call. `matches_any` is one line over `matches`, and `matches` asks
about one named format ignoring table order — a different question from
`what.kind is kind`, and `Ktx` is the case that shows it.

**Rust** has it behind a flag:

```rust
#[cfg(feature = "magical_fluent")]
use magical_rs::magical::fluent::Detect;

#[cfg(feature = "magical_fluent")]
fn scan(paths: &[std::path::PathBuf]) {
    for path in paths {
        let bytes = magical_rs::magical::bytes_read::read_file_header(
            path.to_str().unwrap(),
            magical_rs::magical::bytes_read::with_bytes_read(),
        ).unwrap();
        if bytes.detect().is_some() { }
    }
}
```

The flag is because it puts a method on `[u8]` — a name on a type the caller does
not own. Turning it off leaves no trait, no type and no module behind.

**JavaScript** cannot: a primitive cannot carry a method the module owns, so
`detectBytes(bytes)` stays a function call.

## Bytes you already have

```rust
use magical_rs::magical::magic::FileKind;
assert_eq!(FileKind::match_types(b"GIF89a"), Some(FileKind::GIF));
```

```python
from magical_py import detect_bytes
detect_bytes(header)       # FileKind.Png | None
```

```js
import { detectBytes } from "@reim-developer/magical-js";
detectBytes(header);       // "Png" | null
```

Only the leading bytes are inspected, so passing a whole large file is wasteful.
36,870 bytes is always enough; see [Concepts](concepts.md#the-read-window) for when
less will do.

## Two things to know before you rely on it

**A short answer and a wrong answer look alike without a window.** Give
`detect_bytes` 2,048 bytes cut from an ISO and it returns `None`, because the magic
sits at offset 36,865 and never entered the buffer. A bare `None` reads as "this is
not an ISO", which is not what happened. Name the window and the same call still
returns `None`, and now it means something you can act on.

**Detection reads magic bytes only.** A `.jpg` holding PNG data is reported as PNG.
A name is a claim by whoever wrote it; a magic number is a fact about the bytes.

Both are expanded in [Concepts](concepts.md).

## Where to go next

- [Concepts](concepts.md) — table order, the read window, what `None` means
- [Detection levels](detection-levels.md) — when the built-in table is not enough
- The API reference for your language: [Rust](api/rust.md),
  [Python](api/python.md), [JavaScript](api/javascript.md)
- [Supported formats](../readme.md#supported-formats) — all 114, with their bytes