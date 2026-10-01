//! The bytes every library is measured on, and why they are generated rather
//! than committed.
//!
//! **No binary is committed.** The rest of this repository builds its fixtures
//! from the magic bytes in `SIGNATURE_KIND` for the same reason, and the two
//! corpora are the same corpus. A 36,870-byte file per format is about 4 MB of
//! blobs whose provenance nobody can check and whose licences nobody can
//! audit, sitting in a library whose entire claim is that it needs no data
//! files at all.
//!
//! **The corpus is built from `magical_rs`'s own table, and that is a bias you
//! have to read past.** Every positive case is a real signature written at a
//! real offset into a buffer of filler, so of course this crate recognises all
//! of them: the table is the generator. A "correctness" number computed over
//! this corpus would be a restatement of the table rather than a measurement
//! of it, and no other library could be beaten on it without being beaten by the
//! generator. So the report prints the answers as a *diagnostic* -- what each
//! library said about each buffer, so a reader can see where they disagree --
//! and never as a score. The timing table is the part that is a comparison, and
//! it is fair for the reason the corpus is the same bytes: no library chose its
//! own input.
//!
//! **The filler is checked, not assumed.** Random bytes in 36 KB will
//! occasionally contain a real signature, and a case that matches a *different*
//! format than the one it was built for is a bug in the corpus that would
//! quietly make every library look wrong. [`negatives`] therefore advances the
//! seed until the filler matches nothing in the table, and asserts it, rather
//! than trusting that a seed happened to come out clean.

use std::path::{Path, PathBuf};

use magical_rs::magical::magic::FileKind;
use magical_rs::magical::match_rules::MatchRules;
use magical_rs::magical::signatures::SIGNATURE_KIND;

/// The number of bytes in every buffer handed to every library.
///
/// [`with_bytes_read`], which is the largest `max_bytes_read` in
/// `SIGNATURE_KIND` -- 36,870, because ISO 9660's magic sits at offset 36,865.
/// It is this crate's own library's number, which is exactly why the report
/// prints the two other sizes next to it: `DEFAULT_MAX_BYTES_READ` is 2,048,
/// and a caller who has not read that far cannot detect a third of this table.
/// The comparison is not "all three libraries need the same bytes" -- it is not
/// true -- it is "all three libraries are given the same bytes", which is the
/// only way a timing difference can be read as being about the search rather
/// than about how much was handed to it.
///
/// A `const` with the number written out, rather than a call to
/// `with_bytes_read`, because the benchmark harness needs a length to declare a
/// `Throughput` with and `with_bytes_read` is a runtime function -- it reads a
/// `static` table, which a `const` cannot do. `read_size_is_the_full_header_size`
/// below is what keeps the two from drifting apart.
pub const READ_SIZE: usize = 36_870;

/// One buffer, and what this crate's own table says is in it.
pub struct Case {
    /// A short stable label, used as a benchmark name and a table row.
    ///
    /// `Png` rather than `PNG`, and `Iso@36865` rather than `ISO 9660 at offset
    /// 36865`: the variant name is the identifier the library itself publishes,
    /// and anything longer than it ends up in a `cargo bench` argument that has
    /// to be quoted.
    pub name: String,

    /// What `FileKind::match_types` is expected to answer.
    ///
    /// [`None`] for the negative cases, where the point is that nothing matches.
    /// It is an expectation about *this crate*, and no other library is scored
    /// against it -- see the module documentation.
    pub expected: Option<FileKind>,

    /// The bytes. Always exactly [`READ_SIZE`] long.
    pub bytes: Vec<u8>,
}

/// Every signature in the table, at every offset it is declared at.
///
/// One case per `(entry, signature, offset)` triple rather than one per entry,
/// because a table entry can declare several of each and the interesting part of
/// the index is that the offset decides which entry is even looked at. `Tar` at
/// offset 257 and `ISO` at 32,769 are in here for the same reason as `Png` at
/// zero: they are the cases where a table order that ignores offsets would give
/// a different answer.
///
/// Entries whose rule is a predicate rather than a byte signature are skipped,
/// and the count is reported rather than passed over in silence: `ScriptExecute`
/// and `WEBP` are matched by reading structure, not a fixed byte string, so
/// there is nothing to write down.
#[must_use]
pub fn positives() -> Vec<Case> {
    let mut cases = Vec::new();

    for entry in SIGNATURE_KIND {
        if matches!(entry.rules, MatchRules::WithFn(_)) {
            continue;
        }
        for (which, signature) in entry.signatures.iter().enumerate() {
            for &offset in entry.offsets {
                let mut bytes = filler(FILLER_SEED, READ_SIZE);
                bytes[offset..offset + signature.len()].copy_from_slice(signature);
                // The index is in the name only when the entry has more than one
                // signature, because otherwise three cases would all be called
                // `MP3@0` and a table with three identical row labels is a table
                // nobody can read. The simple names stay simple so they can be
                // typed into `cargo bench` without quoting.
                let name = if entry.signatures.len() == 1 {
                    format!("{}@{offset}", entry.kind.variant_name())
                } else {
                    format!("{}@{offset}-{which}", entry.kind.variant_name())
                };
                cases.push(Case {
                    name,
                    expected: Some(entry.kind),
                    bytes,
                });
            }
        }
    }

    cases
}

/// How many table entries are matched by a predicate rather than a byte string,
/// and so are absent from [`positives`].
#[must_use]
pub fn predicate_rule_entries() -> usize {
    SIGNATURE_KIND
        .iter()
        .filter(|entry| matches!(entry.rules, MatchRules::WithFn(_)))
        .count()
}

/// The real files this repository already commits, and what they are.
///
/// Four files, in `crates/magical_rs/tests/`, kept there because a signature
/// table is only trustworthy if something checks it against a file a program
/// actually produced. They are the wrong size to be a corpus -- 2 MB of ISO
/// next to 427 bytes of `.class` -- and they were not chosen to be
/// representative of anything. They are here anyway, for the reason
/// [`real_files`] gives.
///
/// # Panics
///
/// If the four files are not where this crate expects them. A missing fixture
/// means the repository has been restructured, and returning an empty list would
/// turn that into an answer table with no rows and a note nobody reads.
#[must_use]
pub const fn real_file_names() -> [&'static str; 4] {
    ["1.png", "2.iso", "3.class", "4.webp"]
}

/// The real files, read whole.
///
/// The one part of the answer table that is not talking to itself. Every other
/// buffer in this crate is a signature planted in noise, and that is fine for
/// timing -- the cost of a search does not care what is in the buffer -- and
/// useless for asking whether two libraries agree, because a library that reads
/// past the header is reading noise and a real file's bytes past the header are
/// exactly what it wants.
///
/// This was found the hard way rather than decided in advance: libmagic reports
/// `application/octet-stream` for a buffer holding a valid 8-byte PNG signature
/// and 36,862 zero bytes, and `image/png` for a real PNG whose first eight bytes
/// are the same eight. Its PNG rules look at the IHDR chunk, which a signature
/// does not write. With no real files in the corpus, the answer table's 125 rows
/// were all that one fact wearing 125 different hats.
///
/// # Panics
///
/// If any of the four files cannot be read. A missing fixture means the
/// repository has been restructured, and returning a shorter list would turn that
/// into an answer table with a row missing and no error.
#[must_use]
pub fn real_files() -> Vec<Case> {
    let dir = tests_directory();
    real_file_names()
        .iter()
        .map(|name| {
            let path = dir.join(name);
            let bytes = std::fs::read(&path)
                .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
            // The expected kind is whatever this crate says, not a hard-coded
            // list: if `1.png` ever stopped being a PNG the table should show
            // that rather than assert the old answer still holds.
            Case {
                name: (*name).to_owned(),
                expected: FileKind::match_types(&bytes),
                bytes,
            }
        })
        .collect()
}

/// The `crates/magical_rs/tests` directory.
///
/// Found by walking up to the Makefile rather than by counting `..`, for the
/// reason `tests/workspace.rs` in the library gives: a correct depth today is a
/// second thing to update when the layout changes, and the failure otherwise is
/// a panic in a benchmark whose name says nothing about paths.
///
/// Public because the report needs the same four paths, and two functions
/// walking to the same directory by two different routes is how they end up
/// disagreeing.
///
/// # Panics
///
/// If no ancestor of this crate's directory holds a Makefile, which is the only
/// way this can fail and which means the repository has been restructured.
#[must_use]
pub fn tests_directory() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|dir| dir.join("Makefile").is_file())
        .unwrap_or_else(|| {
            panic!(
                "no ancestor of {} holds a Makefile, so the real files cannot be found",
                env!("CARGO_MANIFEST_DIR")
            )
        })
        .join("crates")
        .join("magical_rs")
        .join("tests")
}

/// Buffers that match nothing, which is the case the dispatch index exists for.
///
/// A linear scan of 114 entries costs the same whether it finds something on
/// the first try or on the hundredth, so a benchmark of positives alone
/// measures a table that is already warm at both ends. These are the buffers
/// that make the difference between an index and a scan visible at all, and they
/// are half the corpus for that reason rather than as a curiosity.
#[must_use]
pub fn negatives(count: usize) -> Vec<Case> {
    // A different seed per negative so they are not 8 copies of one another,
    // and so that "the answer is stable across the corpus" means something.
    (0..count)
        .map(|i| {
            let seed = FILLER_SEED ^ (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            let bytes = clean_filler(seed, READ_SIZE);
            Case {
                name: format!("junk{i}"),
                expected: None,
                bytes,
            }
        })
        .collect()
}

/// The seed the first filler is generated from.
///
/// Written down rather than derived from a hash of anything, because a corpus
/// that changes when an unrelated file is renamed is a corpus whose numbers
/// cannot be compared between two runs of two commits. The benchmark's job is
/// to be the same input twice.
const FILLER_SEED: u64 = 0x6D61_6769_6361_6C72; // "magicalr"

/// [`filler`] with the seed advanced until nothing in the table matches.
///
/// An arbitrary byte pattern would do most of the time, and "most of the time"
/// is the problem: one positive case out of 114 gaining a second format at a
/// random offset changes one row of the answer table, and nobody can tell later
/// whether the corpus or the library moved. Advancing the seed costs nothing and
/// makes the property a checked fact.
fn clean_filler(seed: u64, len: usize) -> Vec<u8> {
    let mut candidate = seed;
    for _ in 0..1_024 {
        let bytes = filler(candidate, len);
        if FileKind::match_types(&bytes).is_none() {
            return bytes;
        }
        candidate = candidate.wrapping_add(1);
    }
    panic!(
        "no filler in 1024 seeds matched nothing, so the corpus cannot be built: every buffer of \
         {len} bytes is a format. That is a bug in the signature table, not in the benchmark."
    );
}

/// A deterministic byte pattern, from xorshift64.
///
/// Not `rand`: a seeded `rand` would be one more dependency whose version can
/// change what the corpus contains between two runs, and this is ten lines with
/// no way for it to drift.
///
/// The bytes are *not* drawn from the full 0..=255 range. A real file's body is
/// arbitrary bytes, but a real file also has a first few dozen bytes that its
/// format says something about, and this one has nothing to say -- so the tail
/// is arbitrary and the head is whatever the signature wrote. Reporting libmagic
/// on a buffer of pure noise is honest, and reporting it on a buffer of noise
/// that begins with a valid header is equally honest, and the second is closer
/// to what a caller has.
fn filler(seed: u64, len: usize) -> Vec<u8> {
    let mut state = seed | 1;
    let mut out = vec![0_u8; len];
    for byte in &mut out {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        // The top 8 of the 32 bits that are left, rather than a truncation of
        // the whole 64. `state >> 33` is 31 bits wide, so `as u8` would throw
        // away two thirds of what the shift just produced and clippy is right to
        // call that a truncation: the corpus would be 31 bits of entropy wearing
        // a 64-bit generator's name.
        *byte = ((state >> 33) & 0xFF) as u8;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use magical_rs::magical::bytes_read::with_bytes_read;

    /// The corpus is shared, so its size is a claim the report makes in every
    /// row. If it drifts, two runs are not comparable.
    ///
    /// The `const` exists because the harness needs a length, and the library
    /// computes its own at runtime; this is the assertion that keeps the written
    /// number equal to the computed one.
    #[test]
    fn read_size_is_the_full_header_size() {
        assert_eq!(
            READ_SIZE,
            with_bytes_read(),
            "READ_SIZE is written out because with_bytes_read() is not const; if a format moved \
             to a higher offset this is the assertion that catches it"
        );
        assert_eq!(READ_SIZE, 36_870, "ISO 9660's signature sits at 36,865");
    }

    /// One case per declared `(signature, offset)`, and the count written down
    /// rather than recomputed by whoever reads the report.
    #[test]
    fn positives_cover_every_declared_offset() {
        let expected: usize = SIGNATURE_KIND
            .iter()
            .filter(|entry| !matches!(entry.rules, MatchRules::WithFn(_)))
            .map(|entry| entry.signatures.len() * entry.offsets.len())
            .sum();
        assert_eq!(positives().len(), expected);
        // Both formats that exist only to be read at an offset are here, which
        // is the property the index is for.
        assert!(positives().iter().any(|c| c.name == "Tar@257"));
        assert!(positives().iter().any(|c| c.name == "ISO@36865"));
    }

    /// A case built from the table must be answered with the table's kind. If
    /// this fails, the corpus is lying to every benchmark in the crate.
    #[test]
    fn every_positive_is_detected_as_its_own_kind() {
        for case in positives() {
            assert_eq!(
                FileKind::match_types(&case.bytes),
                case.expected,
                "case {} does not detect as its own kind",
                case.name
            );
        }
    }

    /// The filler really is inert, for every negative and for the tail of every
    /// positive. A corpus whose junk matches something makes every "did not
    /// detect" cell in the report wrong.
    #[test]
    fn no_negative_matches_anything() {
        for case in negatives(8) {
            assert_eq!(
                FileKind::match_types(&case.bytes),
                None,
                "case {}",
                case.name
            );
        }
    }

    /// Every buffer is the same size, including the ones whose signature sits
    /// past 32 KB. A short ISO case would be a case libmagic never sees.
    #[test]
    fn every_case_is_the_same_size() {
        assert!(positives().iter().all(|c| c.bytes.len() == READ_SIZE));
        assert!(negatives(8).iter().all(|c| c.bytes.len() == READ_SIZE));
    }

    /// Two calls produce the same bytes, which is what makes a benchmark
    /// comparable to the run before it.
    #[test]
    fn the_corpus_is_deterministic() {
        assert_eq!(filler(FILLER_SEED, 512), filler(FILLER_SEED, 512));
        assert_eq!(positives()[0].bytes, positives()[0].bytes);
    }

    /// Every case name is unique.
    ///
    /// Not cosmetic. Three cases were all called `MP3@0` before the signature
    /// index went into the name, and a table with three identical row labels
    /// cannot be read -- and neither can a `cargo bench` argument, since there is
    /// no way to say which of the three MP3s you meant.
    #[test]
    fn case_names_are_unique() {
        let positives = positives();
        let negatives = negatives(8);
        let mut names: Vec<&str> = positives
            .iter()
            .chain(&negatives)
            .map(|case| case.name.as_str())
            .collect();
        let total = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), total, "two cases share a name");
    }

    /// Every real file is one this crate's own library can name.
    ///
    /// If `1.png` ever stopped being a PNG the answer table would print an
    /// expectation of "nothing" and the row would read as a detection failure
    /// rather than as the table being out of date.
    #[test]
    fn every_real_file_is_detected() {
        for case in real_files() {
            assert!(
                case.expected.is_some(),
                "{} is in the repository's own fixture set and this crate does not recognise it, \
                 so the answer table's expectation column is wrong",
                case.name
            );
        }
    }

    /// The real files are not the same size as each other, which is why they are
    /// not in the timing corpus.
    #[test]
    fn real_files_have_real_and_varied_sizes() {
        let sizes: Vec<usize> = real_files().iter().map(|c| c.bytes.len()).collect();
        assert!(
            sizes.iter().all(|&n| n > 0),
            "a fixture is empty: {sizes:?}"
        );
        assert!(
            sizes.windows(2).any(|w| w[0] != w[1]),
            "every fixture is the same size, which is what a synthetic corpus looks like: {sizes:?}"
        );
    }
}
