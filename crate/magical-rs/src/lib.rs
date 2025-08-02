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
//! # #[cfg(not(feature = "no_std"))]
//! # use magical_rs::magical::bytes_read::DEFAULT_MAX_BYTES_READ;
//! # #[cfg(not(feature = "no_std"))]
//! # use magical_rs::magical::{
//! #        bytes_read::{read_file_header, with_bytes_read},
//! #        magic::FileKind,
//! # };
//!
//! # #[cfg(not(feature = "no_std"))]
//! # let png_file = "example.png";
//! # #[cfg(not(feature = "no_std"))]
//! # let header_bytes = read_file_header(png_file, DEFAULT_MAX_BYTES_READ).unwrap();
//!
//! # #[cfg(not(feature = "no_std"))]
//! # assert_eq!(FileKind::match_types(&header_bytes), FileKind::Png);
//! # #[cfg(not(feature = "no_std"))]
//! # assert_ne!(FileKind::match_types(&header_bytes), FileKind::Unknown);
//! ```
//!
//! ---
//!
//! * Use with [`with_bytes_read`]:
//! ```no_run
//! # #[cfg(not(feature = "no_std"))]
//! # use magical_rs::magical::bytes_read::DEFAULT_MAX_BYTES_READ;
//! # #[cfg(not(feature = "no_std"))]
//! # use magical_rs::magical::{
//! #    bytes_read::{read_file_header, with_bytes_read},
//! #    magic::FileKind,
//! # };
//! # #[cfg(not(feature = "no_std"))]
//! # let iso_file = "example.iso";
//! # #[cfg(not(feature = "no_std"))]
//! # let bytes_max = with_bytes_read();
//! # #[cfg(not(feature = "no_std"))]
//! # let wrong_max = DEFAULT_MAX_BYTES_READ;
//! # #[cfg(not(feature = "no_std"))]
//! # let header_bytes = read_file_header(iso_file, bytes_max).unwrap();
//! # #[cfg(not(feature = "no_std"))]   
//! # assert_eq!(FileKind::match_types(&header_bytes), FileKind::ISO);
//! # #[cfg(not(feature = "no_std"))]
//! # assert_ne!(FileKind::match_types(&header_bytes), FileKind::Unknown);
//! ```
//! ---
//!
//! ## Warning: Use [`with_bytes_read`] for **correct detection.**
//!
//! * Always use [`with_bytes_read`] to determine how many bytes to read.
//!
//! [`with_bytes_read`]: https://docs.rs/magical_rs/0.0.4/magical_rs/magical/bytes_read/fn.with_bytes_read.html
//! * Do **NOT** use [`DEFAULT_MAX_BYTES_READ`] unless you only care about common formats (PNG, JPG, etc.).
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
//! * **Correct usage:**
//! ```no_run
//! # # [cfg(not(feature = "no_std"))]
//! # use magical_rs::magical::bytes_read::{with_bytes_read, read_file_header,  DEFAULT_MAX_BYTES_READ};
//! # #[cfg(not(feature = "no_std"))]
//! # use magical_rs::magical::magic::FileKind;
//! #  
//! # #[cfg(not(feature = "no_std"))]
//! # let max_bytes = with_bytes_read(); // ← Auto-calculated safe size
//! #
//! # #[cfg(not(feature = "no_std"))]
//! # let header = read_file_header("file.iso", max_bytes).unwrap();
//! # #[cfg(not(feature = "no_std"))]
//! # let kind = FileKind::match_types(&header);
//! ```
//!
//! ---
//!
//! * **Avoid:**
//! ```no_run
//! #[cfg(not(feature = "no_std"))]
//! # use magical_rs::magical::bytes_read::{with_bytes_read, read_file_header, DEFAULT_MAX_BYTES_READ};
//!
//! #[cfg(not(feature = "no_std"))]
//! # let header = read_file_header("file.iso", DEFAULT_MAX_BYTES_READ).unwrap(); // ← Will fail to detect!
//! ```
//! ---
//!
//! See [`magical_rs`] for more information.
//!
//! ---
//!
//! [`magical_rs`]: https://docs.rs/magical_rs/0.1.0/magical_rs
//!
#![deny(clippy::pedantic, clippy::all, clippy::nursery, clippy::perf)]
#![cfg_attr(feature = "no_std", no_std)]

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
