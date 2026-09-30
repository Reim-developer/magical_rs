//! The crate on WebAssembly, with no binding in between.
//!
//! This is the crate as a dependency and nothing else: no `wasm-bindgen`, no
//! generated glue, no `NodeJS` package. `bindings/asm` does the same thing and adds
//! the encoded detection table, level 2 rule sets and a released, versioned memory
//! ABI; this file does the smallest thing that can work, because "it compiles for
//! `wasm32`" is a claim `make build-wasm` cannot really check -- and this is what
//! checks it, by running.
//!
//! # The memory contract
//!
//! Two rules, both of which are easy to get wrong and neither of which produces an
//! error when you do:
//!
//! 1. `memory.buffer` is detached and replaced whenever memory grows, so a
//!    `Uint8Array` captured at load time is a zero-length view afterwards. The
//!    loader re-derives its view on every call rather than caching one.
//! 2. A write at offset 0 is not free. This module has no detection table -- it calls
//!    the crate's matcher directly rather than decoding one -- but the crate's
//!    `static`s sit somewhere in linear memory, and overwriting them from the host
//!    side would be silent corruption rather than a trap.
//!
//! # The one thing you cannot pass
//!
//! A `&str`. Rust strings are `(pointer, length)` with no terminator, so handing
//! `str::as_ptr` across the boundary gives the caller no way to know where it ends.
//! There are two honest fixes and this uses the first: return the pointer *and* the
//! length as two values, and let the host read exactly that many bytes.
//!
//! The second is a NUL-terminated copy into a fixed buffer the host provides. That
//! works and costs a buffer, and it is what you reach for when the string has to
//! outlive the call. Neither is free and the choice belongs to whoever owns the
//! boundary.
//!
//! # Why no `rlib`
//!
//! Because there is nothing here to unit test from the Rust side: every export is a
//! pointer-arithmetic wrapper over `magical_rs`, and the crate's own tests cover the
//! logic underneath. What nothing in this repository covered before was "a wasm
//! module built from this crate, instantiated with an empty import object, answers
//! correctly for a real header" -- and `load.mjs` covers exactly that, by running.

#![deny(clippy::pedantic, clippy::all, clippy::nursery, clippy::perf)]

use magical_rs::magical::kinds_meta::ALL_KINDS;
use magical_rs::magical::magic::FileKind;

/// The value every "no match" answer uses.
///
/// An `i32` *result* crosses into JavaScript signed, so `u32::MAX` arrives as `-1`
/// rather than as `4294967295`. The loader compares against `-1`; a comparison
/// against `0xffffffff` never matches, which turns "no match" into "the 4294967295th
/// format" and yields `undefined` where the API promises `null`.
const NO_MATCH: u32 = u32::MAX;

/// The format whose discriminant is `kind`.
///
/// `FileKind` is a fieldless enum, so `kind as u32` *is* the discriminant, and that
/// number is the ABI: the JavaScript side indexes its name table by it. This is the
/// same conversion `bindings/asm` does, and it is why the order of the declaration in
/// `magic.rs` is load-bearing for every binding rather than a detail.
///
/// Going through `ALL_KINDS` rather than transmuting the `u32` back is deliberate.
/// A transmute of an arbitrary integer into an enum is undefined behaviour when the
/// value is out of range, and this function is `extern "C"`, which is the last place
/// in a program you want that. A linear scan of 114 entries costs nothing here and
/// turns an out-of-range value into a `None`.
fn from_discriminant(kind: u32) -> Option<FileKind> {
    ALL_KINDS.iter().copied().find(|found| *found as u32 == kind)
}

/// Detect the format of `len` bytes at `ptr`.
///
/// Returns a discriminant, or `-1` in JavaScript for "no signature matched".
///
/// # Safety
///
/// `ptr` must point at `len` readable bytes. A host that passes a stale pointer into
/// a grown `memory` gets a trap rather than a wrong answer, which is the good
/// outcome: the alternative is reading whatever replaced the buffer.
///
/// # Panics
///
/// Never. A wasm module that panics has nowhere to unwind to and the only thing it
/// can do is trap, which takes down the host's page rather than returning a wrong
/// answer. The release profile sets `panic = "abort"` as a backstop; not panicking
/// is the mechanism.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn which_kind_at(ptr: *const u8, len: usize) -> u32 {
    // SAFETY: the caller owns this contract, documented above and in `_wasm.js` on
    // the other side of it. `FileKind::match_types` is the crate's own level 1 --
    // the same function the Python and JavaScript bindings call -- so nothing here
    // reimplements matching and this example cannot answer differently from them.
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    FileKind::match_types(bytes).map_or(NO_MATCH, |kind| kind as u32)
}

/// The length in bytes of a format's display name, or 0 for one this build lacks.
///
/// Read this *before* `display_name_bytes`, because the two are the halves of one
/// value and the caller has no other way to know the length.
#[unsafe(no_mangle)]
pub extern "C" fn display_name_len(kind: u32) -> u32 {
    from_discriminant(kind).map_or(0, |found| {
        u32::try_from(found.display_name().len()).unwrap_or(u32::MAX)
    })
}

/// A pointer to a format's display name, valid until the module is dropped.
///
/// The bytes are `'static`, so this outlives any call -- there is no buffer to free
/// and no lifetime to get wrong. Read exactly `display_name_len` bytes from it.
#[unsafe(no_mangle)]
pub extern "C" fn display_name_bytes(kind: u32) -> *const u8 {
    from_discriminant(kind).map_or(std::ptr::null(), |found| found.display_name().as_ptr())
}
