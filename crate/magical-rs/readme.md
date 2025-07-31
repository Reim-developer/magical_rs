# magical_rs 🪄

> **Detect file types by magic bytes — zero dependencies, pure Rust.**

`magical_rs` is a lightweight, dependency-free Rust crate that detects file types by matching **magic bytes (signatures)** at specific offsets — including tricky formats like `.iso` with signatures **32KB into the file**.

No external tools. No bloated dependencies. Just fast, reliable file type detection.

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

| Format                            | Notes                                                            |
| --------------------------------- | ---------------------------------------------------------------- |
| PNG                               | `‰PNG` at offset 0                                               |
| Bitmap (BMP)                      | `BM` at offset 0                                                 |
| GZIP                              | `1F 8B` at offset 0                                              |
| BZIP2                             | `BZh` (e.g., `BZh9`) at offset 0                                 |
| ZIP / PkgZip (JAR, APK, etc.)     | `PK` at offset 0                                                 |
| TAR                               | `ustar` at offset 257                                            |
| MS-DOS Executable (COM/EXE)       | `MZ` at offset 0 (DOS header)                                    |
| JPG / JPEG                        | Start with `ÿØÿ` (`FF D8 FF`)                                    |
| Java Class File                   | `CAFEBABE` at offset 0                                           |
| MP3 (MPEG Audio)                  | Often starts with `ID3` tag or `FF FB` (MPEG-1 Layer 3)          |
| ISO 9660                          | `CD001` at offset 32769, 34817, or 36865                         |
| RPM                               | Binary header after lead; signature in first few hundred bytes   |
| SQLite                            | `SQLite format 3\0` at offset 0                                  |
| XML                               | Text-based: starts with `<?xml` or `<!DOCTYPE`                   |
| ICO (Icon)                        | `00 00 01 00` (icon) or `00 00 02 00` (cursor) at offset 0       |
| WebAssembly (WASM)                | `\0asm` (`00 61 73 6D`) at offset 0                              |
| DEB (Debian package)              | `!<arch>` at offset 0 (ar archive)                               |
| RAR                               | `Rar!` (`52 61 72 21 1A 07 00`) at offset 0                      |
| Script / Executable               | Shebang: `#!` at offset 0 (e.g., `#!/bin/sh`)                    |
| ELF (Executable and Linkable)     | `\x7fELF` (`7F 45 4C 46`) at offset 0                            |
| OGG (Ogg Vorbis, Opus, etc.)      | `OggS` (`4F 67 67 53`) at offset 0                               |
| Photoshop (8BPS)                  | `8BPS` (`38 42 50 53`) at offset 0                               |
| Blender (.blend)                  | `BLENDER` followed by version (e.g., `BLENDER-v293`) at offset 0 |
| TrueType Font (TTF)               | `00 01 00 00` or `ttcf` at offset 4                              |
| OpenType Font (OTF)               | `OTTO` (`4F 54 54 4F`) at offset 4                               |
| Module (Environment Modules)      | `MODULE\0\0\0` or similar (custom binary format)                 |
| Windows Imaging Format (WIM)      | `MSCF` (`4D 53 43 46`) at offset 0                               |
| SLOB (StarDict Binary Dictionary) | `SLOB` magic at start                                            |
| Serialized Java Data              | `AC ED` (`STREAM_MAGIC`) at offset 0                             |
| Creative Voice File (VOC)         | `Creative Voice File\0` at offset 0                              |
| AU Audio File Format              | `.snd` header: `2E 73 6E 64` at offset 0                         |
| OpenGL Iris Performer (IV)        | Rare; may use `InfiniteReality` or IRIX-based header             |
| Noodlesoft Hazel                  | `HZLR` or `HZL` magic; used in Hazel file manager archives       |
| VBScript Encoded (VBE)            | Starts with `#@~^` (`23 40 7E 5E`); obfuscated VBScript          |
| WebP                              | `RIFFxxxxWEBP` container; `file_size` field must be >= 4         |
| Unknown                           | Fallback when no signature matches                               |

---

## Usage

Add to `Cargo.toml`:

```toml
[dependencies]
magical_rs = "0.0.1"
```