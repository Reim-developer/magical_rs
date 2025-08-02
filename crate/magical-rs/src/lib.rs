//! # File Magic Detection
//!
//! This crate detects file types by matching magic bytes (signatures) at specific offsets.
//!
//! ---
//!
//! ## How It Works
//!
//! - Each file type (e.g., PNG, ISO, ZIP) has one or more known byte signatures.
//! - Some signatures are at offset 0 (e.g., PNG), but others are at large offsets (e.g., ISO at 32769).
//! - To detect **all supported types**, you must read enough bytes from the file header.
//!
//! ---
//!
//! ## Example:
//! * With default constant [`DEFAULT_MAX_BYTES_READ`]:
//!
//! [`DEFAULT_MAX_BYTES_READ`]: https://docs.rs/magical_rs/0.0.4/magical_rs/magical/bytes_read/constant.DEFAULT_MAX_BYTES_READ.html
//!
//! ```no_run
//! use magical_rs::magical::bytes_read::DEFAULT_MAX_BYTES_READ;
//! use magical_rs::magical::{
//!        bytes_read::{read_file_header, with_bytes_read},
//!        magic::FileKind,
//! };
//!
//! let png_file = "example.png";
//! let header_bytes = read_file_header(png_file, DEFAULT_MAX_BYTES_READ).unwrap();
//!
//! assert_eq!(FileKind::match_types(&header_bytes).unwrap(), FileKind::Png);
//! assert!(FileKind::match_types(&header_bytes).is_some());
//! ```
//!
//! ---
//!
//! * Use with [`with_bytes_read`]:
//! ```no_run
//! use magical_rs::magical::bytes_read::DEFAULT_MAX_BYTES_READ;
//! use magical_rs::magical::{
//!     bytes_read::{read_file_header, with_bytes_read},
//!     magic::FileKind,
//! };
//!
//! let iso_file = "example.iso";
//! let bytes_max = with_bytes_read();
//! let wrong_max = DEFAULT_MAX_BYTES_READ;
//! let header_bytes = read_file_header(iso_file, bytes_max).unwrap();
//!
//! assert_eq!(FileKind::match_types(&header_bytes).unwrap(), FileKind::ISO);
//! assert!(FileKind::match_types(&header_bytes).is_some());
//! ```
//! ---
//!
//! ## Warning: Use [`with_bytes_read`] for **correct detection.**
//!
//! * Always use [`with_bytes_read`] to determine how many bytes to read.
//!
//! [`with_bytes_read`]: https://docs.rs/magical_rs/0.0.4/magical_rs/magical/bytes_read/fn.with_bytes_read.html
//! * Do not use [`DEFAULT_MAX_BYTES_READ`] unless you only
//!   care about common formats (PNG, JPG, etc.).
//!
//! ---
//!
//! ### Why?
//!
//! - [`DEFAULT_MAX_BYTES_READ`] is too small for formats like: `.iso` (needs ~36KB)
//! - If you read only 2048 bytes, those files will be misclassified as `Unknown`.
//!
//! ---
//!
//! * Correct usage:
//! ```no_run
//! fn get_iso_sig_sucess() {
//!     use magical_rs::magical::bytes_read::{with_bytes_read, read_file_header,  DEFAULT_MAX_BYTES_READ};
//!     use magical_rs::magical::magic::FileKind;
//!   
//!     /* Auto-calculated safe size */
//!     let max_bytes = with_bytes_read();
//!
//!     let header = read_file_header("file.iso", max_bytes).unwrap();
//!     let kind = FileKind::match_types(&header);
//!     assert!(kind.is_some())
//! }
//! ```
//! ---
//! * Avoid:
//! ```no_run
//! fn get_iso_sig_fail() {
//!     use magical_rs::magical::magic::FileKind;
//!     use magical_rs::magical::bytes_read::{with_bytes_read, read_file_header, DEFAULT_MAX_BYTES_READ};
//!     
//!     /* Will fail to detect. */
//!     let header = read_file_header("file.iso", DEFAULT_MAX_BYTES_READ).unwrap();
//!     let kind = FileKind::match_types(&header);
//!     assert!(kind.is_none())
//! }
//! ```
//! ---
//! ## No Std Features
//!
//! - `magical_rs` is designed to be `no_std`-friendly out of the box.
//!   While the default build includes `std` for convenience (e.g., file I/O utilities), the core detection logic is built on zero-allocation, `&[u8]`-based matching — making it fully compatible with embedded systems, kernels, WASM, and other constrained environments.
//! - Zero dependency on `std`: The core signature matching engine uses only `core`.
//! - No heap allocation: All rules are `&'static`, and matching is done via slicing and comparison.
//! - `const fn`-friendly utilities: Helper functions like `no_std_max_bytes` can be evaluated at compile time.
//! - Extensible without `Vec` or `Box`: Use `MagicCustom<K>` with `&'static` data for custom detection logic.
//!
//! * To use `magical_rs` in a `no_std` context, you can use `Cargo`
//! ```bash
//! cargo add magical_rs --features no_std
//! ```
//! ---
//! **Example:**
//! * With `DEFAULT_MAX_BYTES_READ`:
//!
//! ```no_run
//! #![no_std]
//! #![no_main]
//! # #[cfg(not(feature = "std"))]
//! fn is_png() {
//!     use magical_rs::magical::bytes_read::DEFAULT_MAX_BYTES_READ;
//!     use magical_rs::magical::magic::FileKind;
//!
//!     const PNG_BYTES: &[u8] = &[
//!         0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
//!     ];
//!
//!     let result = FileKind::match_with_max_read_rule(PNG_BYTES, DEFAULT_MAX_BYTES_READ).unwrap();
//!
//!     assert_eq!(result, FileKind::Png);
//! }
//! ```
//!
//! ---
//!
//! * With customize max bytes read:
//!
//! ```no_run
//! #![no_std]
//! #![no_main]
//! # #[cfg(not(feature = "std"))]
//!fn is_png_with_8_bytes() {
//!     use magical_rs::magical::magic::FileKind;
//!
//!     const PNG_BYTES: &[u8] = &[
//!         0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
//!     ];
//!     const MY_MAX_BYTES_READ: usize = 8;
//!     let result = FileKind::match_with_custom_max_read(PNG_BYTES, MY_MAX_BYTES_READ).unwrap();
//!
//!     assert_eq!(result, FileKind::Png);
//! }
//! ```
//!
//! ---
//!
//! * With customize signature:
//! ```no_run
//! #![no_std]
//! #![no_main]
//! # #[cfg(not(feature = "std"))]
//! fn my_custom_match() {
//!     use magical_rs::magical::magic_custom::{CustomMatchRules, MagicCustom, match_types_custom};
//!
//!     #[derive(Debug, Clone, Copy, PartialEq)]
//!         enum ShoujuFile {
//!         MahouShouju,
//!         Unknown,
//!     }
//!
//!     fn is_shoujo_girl(bytes: &[u8]) -> bool {
//!         bytes.starts_with(b"MagicalGirl")
//!     }
//!
//!     static SHOUJO_RULE: MagicCustom<ShoujuFile> = MagicCustom {
//!         signatures: &[],
//!         offsets: &[],
//!         max_bytes_read: 2048,
//!         kind: ShoujuFile::MahouShouju,
//!         rules: CustomMatchRules::WithFn(is_shoujo_girl),
//!     };
//!
//!     let magical_girl = b"MagicalGirl";
//!     let result = match_types_custom(magical_girl, &[SHOUJO_RULE], ShoujuFile::Unknown);
//!
//!     assert_eq!(result, ShoujuFile::MahouShouju);
//!     assert_ne!(result, ShoujuFile::Unknown);
//! }
//! ```
//!
//! ## Dyn Magical Features
//!
//! **Description:**
//! * This crate supports runtime-flexible file type detection via the optional `magical_dyn` feature.
//! * Enable it when you need closures, dynamic dispatch, and arbitrary return types — perfect for plugins, config-driven systems, or interactive tools.
//! ---
//! **Features:**
//! - Closures as matchers: Use inline logic like `|bytes| bytes.starts_with(b"MAGIC")`.
//! - Arbitrary `kind` types: Store any type (`&str`, `String`, structs, enums, etc.) as the detection result.
//! - Dynamic downcasting: Recover original types safely using `.kind_downcast_ref::<T>()`.
//! - Zero-cost when disabled: This feature is **off by default** and does not affect `no_std` or performance-critical use cases.
//! ---
//! **Note:**
//! - Requires `std`: This feature uses `Box<dyn Fn>` and `Box<dyn Any>`, so it only works in `std` environments.
//! - Not compatible with `no_std`: If your target doesn’t have `std`, do not enable `magical_dyn`.
//! - Heap allocation: Uses `Box` — not suitable for `static` contexts unless combined with `LazyLock`.
//! ---
//! **To use this feature:**
//! * Add with `Cargo`
//! ```bash
//! cargo add magical-rs --features magical_dyn
//!  ```
//! ---
//! **Example:**
//! * Dynamic file detection:
//! ```rust
//! # #[cfg(feature = "magical_dyn")]
//! fn my_dyn_magic_custom() {
//!     use magical_rs::magical::dyn_magic::DynMagicCustom;
//!
//!     let rule_fn = |bytes: &[u8]| bytes.starts_with(b"Shoujo");
//!     let rule = DynMagicCustom::new(rule_fn, "Magical", 32);
//!
//!     assert!(rule.matches(b"Shoujo<3"));
//!     assert!(!rule.matches(b"Not Shoujo Here..."));
//! }
//! ```
//! ---
//! ```
//! # #[cfg(feature = "magical_dyn")]
//! fn my_match_dyn() {
//!     use magical_rs::magical::dyn_magic::{DynMagicCustom, match_dyn_types_as};
//!
//!     let rules = vec![
//!             DynMagicCustom::new(|bytes: &[u8]| bytes.starts_with(b"PNG"), "image/png", 8),
//!             DynMagicCustom::new(
//!             |bytes: &[u8]| bytes.starts_with(b"Shoujo"),
//!             String::from("MagicalGirl"),
//!             6969,
//!         ),
//!     ];
//!
//!     let data = b"Shoujo";
//!     let result = match_dyn_types_as::<String>(data, &rules).unwrap();
//!
//!     assert_eq!(result, &"MagicalGirl".to_string());
//! }
//! ```

#![deny(clippy::pedantic, clippy::all, clippy::nursery, clippy::perf)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "magical_dyn")]
extern crate std;

pub mod magical {
    pub mod bytes_read;

    pub mod ext_fn {
        pub mod webp;
    }

    pub mod dyn_magic;
    pub mod magic;
    pub mod magic_custom;
    pub mod match_rules;
    pub mod signatures;
}
