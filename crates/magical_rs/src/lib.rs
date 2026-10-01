// The crate's own `readme.md`, one level up from `src/`.
//
// **This path is the packaged crate's path, not the repository's.** `Cargo.toml`
// says `readme = "../../readme.md"`, and cargo honours it by *copying* the
// repository's readme into the archive under the name `readme.md`, at the root of
// the package. In the repository, where this file is three levels below the
// readme, `../../../readme.md` is the path that works — and it is the path that
// fails on `cargo publish`, which compiles the copied package rather than the
// source tree:
//
//   error: couldn't read `src/../../../readme.md`: No such file or directory
//
// One level up resolves in both places, because inside the package `src/..` is the
// package root where cargo put it, and inside the repository `src/..` is the crate
// directory, which does not contain a `readme.md`.
//
// That asymmetry is the whole reason the crate carries a checked-in copy rather
// than reaching upward, and `tests/packaging.rs` is what keeps the copy equal to
// the root readme. Three levels up was correct for the repository and wrong for
// the archive, and nothing in CI compiled the archive — `cargo build` never sees
// the copied package, so 0.6.4's published docs built and the *next* version's
// verify failed.
#![doc = include_str!("../readme.md")]
#![deny(clippy::pedantic, clippy::all, clippy::nursery, clippy::perf)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(any(feature = "magical_dyn", feature = "magical_async_dyn"))]
extern crate std;

pub mod magical {
    pub mod bytes_read;
    pub mod dispatch;

    pub mod ext_fn {
        pub mod shebang;
        pub mod webp;
    }

    pub mod async_dyn_magic;
    pub mod dyn_magic;

    // The `cfg` is on this line and not only inside the module, so that turning
    // `magical_fluent` off leaves no trace in the public API — not a trait, not a
    // type, and not a module that exists and holds nothing. `tests/fluent.rs`
    // asserts that by reading this file, and the failure it would otherwise cause
    // is one a caller finds out about from documentation.
    #[cfg(feature = "magical_fluent")]
    pub mod fluent;

    pub mod kinds_meta;
    pub mod magic;
    pub mod magic_custom;
    pub mod match_rules;
    pub mod rules_dsl;
    pub mod signatures;
    pub mod signatures_ext;
}
