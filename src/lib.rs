#![doc = include_str!("../readme.md")]
#![deny(clippy::pedantic, clippy::all, clippy::nursery, clippy::perf)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(any(feature = "magical_dyn", feature = "magical_async_dyn"))]
extern crate std;

pub mod magical {
    pub mod bytes_read;

    pub mod ext_fn {
        pub mod shebang;
        pub mod webp;
    }

    pub mod async_dyn_magic;
    pub mod dyn_magic;
    pub mod magic;
    pub mod magic_custom;
    pub mod match_rules;
    pub mod signatures;
    pub mod signatures_ext;
}
