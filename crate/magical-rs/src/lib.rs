//! # File Magic Detection
//!
//! This crate detects file types by matching magic bytes (signatures) at specific offsets.
//!
//! ## How It Works
//!
//! - Each file type (e.g., PNG, ISO, ZIP) has one or more known byte signatures.
//! - Some signatures are at offset 0 (e.g., PNG), but others are at large offsets (e.g., ISO at 32769).
//! - To detect **all supported types**, you must read enough bytes from the file header.
//!
//! ## Critical: Use `with_bytes_read()` for Correct Detection
//!
//! Always use [`with_bytes_read()`] to determine how many bytes to read.
//! Do **not** use `DEFAULT_MAX_BYTES_READ` unless you only care about common formats (PNG, JPG, etc.).
//!
//! ### Why?
//!
//! - `DEFAULT_MAX_BYTES_READ = 2048` is **too small** for formats like:
//!   - `.iso` (needs ~36KB)
//! - If you read only 2048 bytes, those files will be misclassified as `Unknown`.
//!
//! **Correct usage:**
//! ```rust
//! let max_bytes = magical_rs::with_bytes_read(); // ← Auto-calculated safe size
//! let header = read_file_header("file.iso", max_bytes)?;
//! let kind = FileKind::match_types(&header);
//! ```
//!
//! **Avoid:**
//! ```rust
//! let header = read_file_header("file.iso", DEFAULT_MAX_BYTES_READ)?; // ← Will fail to detect!
//! ```
//!
//! See [`with_bytes_read()`] for details.

#![deny(clippy::pedantic, clippy::all, clippy::nursery, clippy::perf)]

pub mod magical {
    pub mod bytes_read;
    pub mod magic;
    pub mod signatures;
}
