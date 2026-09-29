# `magical_rs`

Zero-dependency file type detection for Rust, with a customization layer that
stays out of your way until you need it.

`magical_rs` identifies files by their magic bytes. It has **no dependencies at
all** — not one — and it compiles for `no_std` targets, so it works in
embedded, kernel-adjacent, and WebAssembly builds where most detection crates
cannot go.

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
[`src/magical/signatures.rs`](https://github.com/Reim-developer/magical_rs/blob/master/src/magical/signatures.rs).
"Offset" is the byte position the magic is compared at.

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
[`src/magical/ext_fn/webp.rs`](https://github.com/Reim-developer/magical_rs/blob/master/src/magical/ext_fn/webp.rs)
because the format requires checking the `RIFF` header and the file size field
together. Shebangs use
[`src/magical/ext_fn/shebang.rs`](https://github.com/Reim-developer/magical_rs/blob/master/src/magical/ext_fn/shebang.rs)
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
[`src/magical/signatures_ext.rs`](https://github.com/Reim-developer/magical_rs/blob/master/src/magical/signatures_ext.rs).

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

The [`magical_py`](https://github.com/Reim-developer/magical_rs/tree/master/bindings/python)
bindings carry their own six runnable scripts in
[`bindings/python/examples/`](https://github.com/Reim-developer/magical_rs/tree/master/bindings/python/examples),
covering the API that is specific to Python: detecting from a path, from
memory, walking a directory tree, and telling a `None` result apart from a
failure to read the file. Their
[detection levels](https://github.com/Reim-developer/magical_rs/tree/master/bindings/python#detection-levels)
section covers levels 2, 3 and 4, the custom rules, and what each one costs.

## Development

```bash
cargo build
cargo test
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
```

Verify `no_std` still holds:

```bash
cargo build --no-default-features --target thumbv7em-none-eabi
```

## License

MIT — see [LICENSE](https://github.com/Reim-developer/magical_rs/blob/master/LICENSE).

Versions `0.4.5` and earlier were published under the GNU General Public
License v3.0. Those grants are permanent and remain in force for anyone who
already received those versions. Only `0.5.0` and later are MIT-licensed.
