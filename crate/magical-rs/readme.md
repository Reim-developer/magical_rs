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

| Format     | Notes                            |
| ---------- | -------------------------------- |
| PNG        | !`‰PNG` at offset 0              |
| JPG        | !Start with `ÿØÿ`                |
| GIF        | !`GIF87a` / `GIF89a`             |
| ZIP        | `PK` at offset 0                 |
| GZIP       | `1F 8B`                          |
| BZIP2      | `BZh9`                           |
| TAR        | `ustar` at offset 257            |
| ISO        | `CD001` at 32769+ (yes, really!) |
| RPM        | Signature in header              |
| DEB        | `!<arch>` + control/data         |
| ELF        | `\x7fELF`                        |
| PDF        | `%PDF-`                          |
| XML        | Text-based detection             |
| ICO        | Icon format                      |
| WASM       | WebAssembly binary               |
| Java Class | `CAFEBABE`                       |
| Unknown    | Fallback                         |

---

## Usage

Add to `Cargo.toml`:

```toml
[dependencies]
magical_rs = "0.0.1"
```