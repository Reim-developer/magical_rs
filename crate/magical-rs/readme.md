# magical_rs

> **Detect file types by magic bytes — zero dependencies, pure Rust.**

`magical_rs` is a lightweight, dependency-free Rust crate that detects file types by matching **magic bytes (signatures)** at specific offsets — including tricky formats like `.iso` with signatures **32KB into the file**.

No external tools. No bloated dependencies. Just fast, reliable file type detection.

---

## Table of Contents:
- [magical\_rs](#magical_rs)
  - [Table of Contents:](#table-of-contents)
  - [Features](#features)
  - [Supported File Types](#supported-file-types)
  - [How to Install](#how-to-install)
  - [Example](#example)
  - [License](#license)

---

## Features

- **Zero dependencies** — pure Rust, `no_std`-friendly (with minor tweaks).
- **Accurate detection** — supports common formats (PNG, JPG, ZIP) and **deep-offset types like ISO, RPM, TAR**.
- **Smart header reading** — reads just enough bytes to detect all known types.
- **Python-compatible** — exposes clean PyO3 bindings for Python interop.
- **Fast & safe** — uses compile-time constants and safe slicing.
- **Extensible** — easy to add new signatures.

---

## Supported File Types

| Format                            | Notes                                                                                                 |
| --------------------------------- | ----------------------------------------------------------------------------------------------------- |
| PNG                               | `‰PNG` at offset 0                                                                                    |
| Bitmap (BMP)                      | `BM` at offset 0                                                                                      |
| GZIP                              | `1F 8B` at offset 0                                                                                   |
| BZIP2                             | `BZh` (e.g., `BZh9`) at offset 0                                                                      |
| ZIP / PkgZip (JAR, APK, etc.)     | `PK` at offset 0                                                                                      |
| TAR                               | `ustar` at offset 257                                                                                 |
| MS-DOS Executable (COM/EXE)       | `MZ` at offset 0 (DOS header)                                                                         |
| JPG / JPEG                        | Start with `ÿØÿ` (`FF D8 FF`)                                                                         |
| Java Class File                   | `CAFEBABE` at offset 0                                                                                |
| MP3 (MPEG Audio)                  | Often starts with `ID3` tag or `FF FB` (MPEG-1 Layer 3)                                               |
| ISO 9660                          | `CD001` at offset 32769, 34817, or 36865                                                              |
| RPM                               | Binary header after lead; signature in first few hundred bytes                                        |
| SQLite                            | `SQLite format 3\0` at offset 0                                                                       |
| XML                               | Text-based: starts with `<?xml` or `<!DOCTYPE`                                                        |
| ICO (Icon)                        | `00 00 01 00` (icon) or `00 00 02 00` (cursor) at offset 0                                            |
| WebAssembly (WASM)                | `\0asm` (`00 61 73 6D`) at offset 0                                                                   |
| DEB (Debian package)              | `!<arch>` at offset 0 (ar archive)                                                                    |
| RAR                               | `Rar!` (`52 61 72 21 1A 07 00`) at offset 0                                                           |
| Script / Executable               | Shebang: `#!` at offset 0 (e.g., `#!/bin/sh`)                                                         |
| ELF (Executable and Linkable)     | `\x7fELF` (`7F 45 4C 46`) at offset 0                                                                 |
| OGG (Ogg Vorbis, Opus, etc.)      | `OggS` (`4F 67 67 53`) at offset 0                                                                    |
| Photoshop (8BPS)                  | `8BPS` (`38 42 50 53`) at offset 0                                                                    |
| Blender (.blend)                  | `BLENDER` followed by version (e.g., `BLENDER-v293`) at offset 0                                      |
| TrueType Font (TTF)               | `00 01 00 00` or `ttcf` at offset 4                                                                   |
| OpenType Font (OTF)               | `OTTO` (`4F 54 54 4F`) at offset 4                                                                    |
| Module (Environment Modules)      | `MODULE\0\0\0` or similar (custom binary format)                                                      |
| Windows Imaging Format (WIM)      | `MSCF` (`4D 53 43 46`) at offset 0                                                                    |
| SLOB (StarDict Binary Dictionary) | `SLOB` magic at start                                                                                 |
| Serialized Java Data              | `AC ED` (`STREAM_MAGIC`) at offset 0                                                                  |
| Creative Voice File (VOC)         | `Creative Voice File\0` at offset 0                                                                   |
| AU Audio File Format              | `.snd` header: `2E 73 6E 64` at offset 0                                                              |
| OpenGL Iris Performer (IV)        | Rare; may use `InfiniteReality` or IRIX-based header                                                  |
| Noodlesoft Hazel                  | `HZLR` or `HZL` magic; used in Hazel file manager archives                                            |
| VBScript Encoded (VBE)            | Starts with `#@~^` (`23 40 7E 5E`); obfuscated VBScript                                               |
| WebP                              | `RIFFxxxxWEBP` container; `file_size` field must be >= 4                                              |
| AppleIconImage                    | `icns` at offset 0                                                                                    |
| GIF                               | `GIF87a` or `GIF89a` at offset 0                                                                      |
| JPEG2000                          | `\0\0\0\x0C\0\njP\x20\x20\r\n\x87\n` or `\xFF\x4F\xFF\x51` at offset 0                                |
| PDF                               | `%PDF` at offset 0                                                                                    |
| AppleDiskImage                    | `koly` at offset 1048576 (rare), or `cafe`/`ed2k` in header; often starts with zeros but magic at end |
| Cabinet                           | `MSCF` at offset 0                                                                                    |
| MatroskaMediaContainer            | `\x1A\x45\xDF\xA3` at offset 0                                                                        |
| RichTextFormat                    | `{\\rtf` at offset 0                                                                                  |
| PhotoCapTemplate                  | No standard public signature; often `.pct` or `.tpl`; may be proprietary                              |
| AceCompressed                     | `ACE` followed by version byte at offset 0 (e.g., `ACE\x01`)                                          |
| FlashVideo                        | `FLV\x01` at offset 0                                                                                 |
| Unknown                           | Fallback when no signature matches                                                                    |

---

## How to Install

* Add to `Cargo.toml`:

```toml
[dependencies]
magical_rs = "0.0.4"
```
---
* Or, use with `Cargo` package manager:
```bash
cargo add magical_rs
```

## Example

* With default constant `DEFAULT_MAX_BYTES_READ`:

```rust
use magical_rs::magical::bytes_read::DEFAULT_MAX_BYTES_READ;
use magical_rs::magical::{
        bytes_read::{read_file_header, with_bytes_read},
        magic::FileKind,
};

let png_file = "example.png";
let header_bytes = read_file_header(png_file, DEFAULT_MAX_BYTES_READ).unwrap();

assert_eq!(FileKind::match_types(&header_bytes), FileKind::Png);
assert_ne!(FileKind::match_types(&header_bytes), FileKind::Unknown);
```

---

* Use with `with_bytes_read()`:
```rust
use magical_rs::magical::bytes_read::DEFAULT_MAX_BYTES_READ;
use magical_rs::magical::{
    bytes_read::{read_file_header, with_bytes_read},
    magic::FileKind,
};

let iso_file = "example.iso";
let bytes_max = with_bytes_read();
let wrong_max = DEFAULT_MAX_BYTES_READ;

let header_bytes = read_file_header(iso_file, bytes_max).unwrap();

assert_eq!(FileKind::match_types(&header_bytes), FileKind::ISO);
assert_ne!(FileKind::match_types(&header_bytes), FileKind::Unknown);
```

## License
* `magical_rs` is licensed under the GNU General Public License v3.0. [See here](LICENSE)