//! # `magical_rs`
//!
//! **A zero-dependency, no_std-friendly Rust crate that detects file types with surgical
//! precision by matching magic bytes at exact offsets —
//! including deep-offset formats like ISO and TAR. Built for speed, safety, and extensibility.**
//!
//! ---
//!
//! ## Level of use
//!
//! **Level 1, uses built-in file detection via signature:**
//! * At this level, you will use the `magical_rs` built-in API, which supports detecting ~50 file types via signatures.
//! * List of currently supported file types [here](#supported-file-types)
//! * By the way, you can contribute new file signatures [here](https://github.com/Reim-developer/magical_rs/pulls)
//!
//! * Examples:
//!
//!```no_run
//!use magical_rs::magical::bytes_read::{read_file_header, with_bytes_read};
//!use magical_rs::magical::magic::FileKind;
//!
//!pub fn my_detect_file() {
//!     let max_byte_read = with_bytes_read();
//!
//!     let bytes = read_file_header("img/2.iso", max_byte_read).unwrap();
//!
//!     match FileKind::match_types(&bytes) {
//!         Some(k) => println!("{k:?}"),
//!         None => println!("Could not detect ISO file."),
//!     }
//!}
//!```
//! * More examples of level 1 can be found [here](https://github.com/Reim-developer/magical_rs/tree/master/examples/dyn_magic)
//!
//! ---
//!
//! **Level 2, untilimited compiler-time customization with infinite function pointers:**
//! * At this level, you can customize file signatures, offsets, and more.
//!
//! * Here, you can also use function pointers for complex logic. In theory, you can do almost anything at compiler-time. And thanks to that, you can detect any file type you want.
//!
//! * Also, supports the use of infinite function pointers at once. And function pointers have macros with syntax-sugar supports at well.
//!
//! * Examples:
//!```rust
//!use magical_rs::{any_matches, magic_custom, match_custom};
//!
//!#[derive(Clone, Copy, PartialEq, Eq, Debug)]
//!enum FileKind {
//!  Shoujo,
//!  UnknownFallback,
//!}
//!
//!fn is_shoujo(bytes: &[u8]) -> bool {
//!   bytes.starts_with(b"Magic!")
//!}
//!
//!fn is_not_shoujo(bytes: &[u8]) -> bool {
//!   !bytes.starts_with(b"Magic!")
//!}
//!
//!pub fn magic_custom_any() {
//!    let rule = magic_custom! (
//!        signatures: [],
//!        offsets: [],
//!        max_bytes_read: 2451,
//!        kind: FileKind::Shoujo,
//!        rules: any_matches!(is_shoujo, is_not_shoujo)
//!    );
//!
//!    let result = match_custom! (
//!        bytes: b"Magic!",
//!        rules: [rule],
//!        fallback: FileKind::UnknownFallback
//!    );
//!
//!    assert_eq!(result, FileKind::Shoujo);
//!    assert_ne!(result, FileKind::UnknownFallback);
//!}
//!```
//! * There are many ways to implement it, and you can find them [here](https://github.com/Reim-developer/magical_rs/tree/master/examples/magic_custom)
//!
//! **Level 1 & 2 both support `no_std`.**
//!
//! ---
//!
//! **Level 3, custom run-time file detection with infinite logic**
//! * At this level, you can customize the runtime logic. You can do anything. You can detect any type of file, even if it changes at run-time. You can emit AI, send requests to the Open-AI API, even spawn processes. The only limit is your imagination.
//!
//! * So, you need to unlock this feature by:
//!
//!```bash
//!cargo add magical_rs --features magical_dyn
//!```
//! ---
//! * Examples:
//!```no_run
//!# #[cfg(feature = "magical_dyn")]
//!use magical_rs::magical::dyn_magic::DynMagicCustom;
//!
//!# #[cfg(feature = "magical_dyn")]
//!fn my_detect_rule() -> impl Fn(&[u8]) -> bool {
//!   let require_bytes = b"MagicalGirl";
//!
//!   |bytes: &[u8]| bytes.starts_with(require_bytes) && bytes.len() == require_bytes.len()
//!
//!}
//!
//!# #[cfg(feature = "magical_dyn")]
//!fn detect_custom_file(file_bytes: &'static [u8]) -> bool {
//!
//!   let detect_fn = my_detect_rule();
//!   let rule = DynMagicCustom::new(detect_fn, String::from("Is Mahou Shoujo Detect."), 32);
//!
//!   let kind = rule.kind_downcast_ref::<String>();
//!
//!    match kind {
//!          Some(k) => println!("{k}"), /* Is Mahou Shoujo Detect. */
//!          None => println!("Kind not found."),
//!     }
//!    rule.matches(file_bytes)
//!}
//!```
//! * Many examples of use can be found [here](https://github.com/Reim-developer/magical_rs/tree/master/examples/dyn_magic)
//!
//! * Warning: Use only if you really know what you are doing.
//!
//! ---
//!
//! **Level 4, Configure, code rules, deploy endlessly and without limits in asynchronous.**
//!* Here you can design file detection rules with any logic no matter how complex in asynchronous environment.
//!  There are no specific limitations other than your own skill level.
//!  Only use it when you know what you're doing, and only use it when you really need
//!  to identify files in an asynchronous environment.
//!  If you are new to Rust or unsure, stop here and just use level 2 below.
//!  You're not as good as you think.
//!  Unless you really understand what the hell you're doing.
//!  And if you really know what you're doing, congratulations!
//!  You have one of the most powerful file recognition systems in the Rust ecosystem.
//!
//!* Don't blame me and other maintainers for your ignorance if
//!  something bad happends because I warned you in advance.
//!
//! ```no_run
//! # #[cfg(feature = "magical_async_dyn")]
//! use {
//!     async_std::task,
//!     magical_rs::magical::async_dyn_magic::AsyncDynMagic,
//!     magical_rs::magical::async_dyn_magic::match_dyn_types_as,
//!     std::time::Duration
//! };
//!
//! # #[cfg(feature = "magical_async_dyn")]
//! async fn magic_async_detect() {
//!     let func_detect = |bytes: &[u8]| {
//!         let owned_bytes = bytes.to_vec();
//!
//!         Box::pin(async move {
//!             println!("Rest for 1 second");
//!
//!             task::sleep(Duration::from_millis(1000)).await;
//!             owned_bytes.starts_with(b"Magical")
//!         })
//!     };
//!
//!     let rule = AsyncDynMagic::new(func_detect, "Magical_File", 128);
//!     let rules = vec![rule];
//!
//!     let result = match_dyn_types_as::<&str>(b"Magical", &rules).await;
//!     match result {
//!         Some(r) => println!("Magical File Detect: {r}"),
//!         None => println!("Magical File Not Found"),
//!     }
//! }
//!
//! ```
//!
//! ---
//!
//! ## Supported File Types
//!
//! | Format                            | Notes                                                                                                 |
//! | --------------------------------- | ----------------------------------------------------------------------------------------------------- |
//! | PNG                               | `‰PNG` at offset 0                                                                                    |
//! | Bitmap (BMP)                      | `BM` at offset 0                                                                                      |
//! | GZIP                              | `1F 8B` at offset 0                                                                                   |
//! | BZIP2                             | `BZh` (e.g., `BZh9`) at offset 0                                                                      |
//! | ZIP / `PkgZip` (JAR, APK, etc.)   | `PK` at offset 0                                                                                      |
//! | TAR                               | `ustar` at offset 257                                                                                 |
//! | MS-DOS Executable (COM/EXE)       | `MZ` at offset 0 (DOS header)                                                                         |
//! | JPG / JPEG                        | Start with `ÿØÿ` (`FF D8 FF`)                                                                         |
//! | Java Class File                   | `CAFEBABE` at offset 0                                                                                |
//! | MP3 (MPEG Audio)                  | Often starts with `ID3` tag or `FF FB` (MPEG-1 Layer 3)                                               |
//! | ISO 9660                          | `CD001` at offset 32769, 34817, or 36865                                                              |
//! | RPM                               | Binary header after lead; signature in first few hundred bytes                                        |
//! | `SQLite`                          | `SQLite format 3\0` at offset 0                                                                       |
//! | XML                               | Text-based: starts with `<?xml` or `<!DOCTYPE`                                                        |
//! | ICO (Icon)                        | `00 00 01 00` (icon) or `00 00 02 00` (cursor) at offset 0                                            |
//! | WebAssembly (WASM)                | `\0asm` (`00 61 73 6D`) at offset 0                                                                   |
//! | DEB (Debian package)              | `!<arch>` at offset 0 (ar archive)                                                                    |
//! | RAR                               | `Rar!` (`52 61 72 21 1A 07 00`) at offset 0                                                           |
//! | Script / Executable               | Shebang: `#!` at offset 0 (e.g., `#!/bin/sh`)                                                         |
//! | ELF (Executable and Linkable)     | `\x7fELF` (`7F 45 4C 46`) at offset 0                                                                 |
//! | OGG (Ogg Vorbis, Opus, etc.)      | `OggS` (`4F 67 67 53`) at offset 0                                                                    |
//! | Photoshop (8BPS)                  | `8BPS` (`38 42 50 53`) at offset 0                                                                    |
//! | Blender (.blend)                  | `BLENDER` followed by version (e.g., `BLENDER-v293`) at offset 0                                      |
//! | TrueType Font (TTF)               | `00 01 00 00` or `ttcf` at offset 4                                                                   |
//! | OpenType Font (OTF)               | `OTTO` (`4F 54 54 4F`) at offset 4                                                                    |
//! | Module (Environment Modules)      | `MODULE\0\0\0` or similar (custom binary format)                                                      |
//! | Windows Imaging Format (WIM)      | `MSCF` (`4D 53 43 46`) at offset 0                                                                    |
//! | SLOB (Star Dict Binary Dictionary)| `SLOB` magic at start                                                                                 |
//! | Serialized Java Data              | `AC ED` (`STREAM_MAGIC`) at offset 0                                                                  |
//! | Creative Voice File (VOC)         | `Creative Voice File\0` at offset 0                                                                   |
//! | AU Audio File Format              | `.snd` header: `2E 73 6E 64` at offset 0                                                              |
//! | OpenGL Iris Performer (IV)        | Rare; may use `InfiniteReality` or IRIX-based header                                                  |
//! | Noodlesoft Hazel                  | `HZLR` or `HZL` magic; used in Hazel file manager archives                                            |
//! | VB Script Encoded (VBE)           | Starts with `#@~^` (`23 40 7E 5E`); obfuscated VB Script                                              |
//! | WebP                              | `RIFFxxxxWEBP` container; `file_size` field must be >= 4                                              |
//! | Apple Icon Image                  | `icns` at offset 0                                                                                    |
//! | GIF                               | `GIF87a` or `GIF89a` at offset 0                                                                      |
//! | JPEG2000                          | `\0\0\0\x0C\0\njP\x20\x20\r\n\x87\n` or `\xFF\x4F\xFF\x51` at offset 0                                |
//! | PDF                               | `%PDF` at offset 0                                                                                    |
//! | Apple Dis kImage                  | `koly` at offset 1048576 (rare), or `cafe`/`ed2k` in header; often starts with zeros but magic at end |
//! | Cabinet                           | `MSCF` at offset 0                                                                                    |
//! | Matroska Media Container          | `\x1A\x45\xDF\xA3` at offset 0                                                                        |
//! | Rich Text Format                  | `{\\rtf` at offset 0                                                                                  |
//! | Photo Cap Template                | No standard public signature; often `.pct` or `.tpl`; may be proprietary                              |
//! | Ace Compressed                    | `ACE` followed by version byte at offset 0 (e.g., `ACE\x01`)                                          |
//! | Flash Video                       | `FLV\x01` at offset 0                                                                                 |
//! | Unknown                           | Fallback when no signature matches                                                                    |
//! | VMDK File                         | `0x4B, 0x44, 0x4D`                                            at off set `0`                          |
//! | Google Chrome Extension           | `0x43, 0x72, 0x32, 0x34`                            at off set `0`                                    |
//!
//! ## License
//! * `magical_rs` is licensed under the GNU General Public License v3.0.

#![deny(clippy::pedantic, clippy::all, clippy::nursery, clippy::perf)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "magical_dyn")]
extern crate std;

#[cfg(feature = "magical_async_dyn")]
extern crate std;

pub mod magical {
    pub mod bytes_read;

    pub mod ext_fn {
        pub mod webp;
    }

    pub mod async_dyn_magic;
    pub mod dyn_magic;
    pub mod magic;
    pub mod magic_custom;
    pub mod match_rules;
    pub mod signatures;
}
