//! Benchmarks for `magical_rs` against the libraries a reader would compare it
//! to, on one shared corpus.
//!
//! This crate is excluded from the root workspace -- see its `Cargo.toml` -- so
//! that adding a benchmark harness does not put three hundred packages into the
//! library's lockfile, and it is not published, because a benchmark is not an
//! artifact anybody depends on.
//!
//! # What is being claimed, and what is not
//!
//! **The timing table is a comparison.** Every library is handed the same
//! `Vec<u8>`, of the same length, in the same order, the same number of times.
//! None of them chose its own input, and the harness does not special-case one
//! of them. That is the whole claim, and it is a narrow one: on these bytes,
//! with this build profile, one search costs less than another.
//!
//! **The answer table is not.** The corpus's positive cases are generated from
//! `SIGNATURE_KIND` -- this crate's own library's own table -- so of course it
//! recognises all of them. A "detection accuracy" score computed over this
//! corpus would be measuring the generator, and no competitor could win it. It
//! is printed anyway, as a table of what each library said about each buffer,
//! because a disagreement between two libraries on identical bytes is
//! interesting in a way a score is not. See [`corpus`].
//!
//! **The corpus is not real files.** It is a real signature at a real offset in
//! a buffer of deterministic noise, which is what the rest of this repository's
//! tests use too, and it is the reason a 4 MB binary blob is not committed to a
//! library whose claim is that it ships no data files. A real PNG is not a
//! correct IHDR chunk followed by noise, and libmagic reads more of a buffer
//! than its first bytes. Anything in the answer table that depends on what is
//! *after* the header is this crate's corpus talking, not the libraries.
//!
//! **One-time costs are not in the per-call numbers.** libmagic loads a
//! database before it can answer anything; a library that does not is not a
//! faster detector, it is a library with nothing to load. It gets its own row.
//!
//! # Running it
//!
//! ```text
//! make bench            # probes for libmagic and runs everything it can
//! cargo bench -p ...   # not usable from the root; see the crate's README
//! ```
//!
//! Nothing here gates CI on a number. Shared runners vary by a factor of two
//! between jobs, and a benchmark that fails because the machine was busy is a
//! benchmark that gets deleted. The summary is written to the job's step
//! summary so the numbers are in the pull request where a human can read them.

#![deny(clippy::pedantic, clippy::all, clippy::nursery, clippy::perf)]

pub mod adapter;
pub mod corpus;
pub mod report;
