//! A first-byte index over [`SIGNATURE_KIND`], built during const evaluation.
//!
//! # Why this exists
//!
//! Detection was `SIGNATURE_KIND.iter().find(..)`, which is O(position in the
//! table). Measured on the cases a caller actually has -- this repository, one
//! machine, `opt-level = 3`, 300,000 iterations each:
//!
//! | input | table position | before | after |
//! | --- | --- | --- | --- |
//! | PNG | 0 | 5 ns | 5 ns |
//! | ZIP | 4 | 23 ns | 6 ns |
//! | GIF | 36 | 123 ns | 21 ns |
//! | not a format | all 114 | **525 ns** | **31 ns** |
//! | empty buffer | all 114 | 525 ns | 18 ns |
//!
//! A hundred-fold spread from position alone, and the worst case is the common one:
//! a directory scan is mostly files this table does not recognise, and each of them
//! paid for all 114 entries. That case is 17 times faster now; the early-match case
//! is unchanged, which took an early exit to arrange -- see [`first_match`].
//!
//! # The sparse list compares integers
//!
//! The seven entries whose signature sits at a non-zero offset are compared as one
//! 64-bit integer rather than through `Magic::matches`. A signature of eight bytes
//! or fewer is held zero-padded in a `u64`, the input's eight bytes at the same
//! offset are loaded the same way, and the two are compared after masking -- one
//! load, one `and`, one `cmp`, against a fat-pointer load, a bounds check at an
//! offset the compiler cannot know, and a `memcmp` call.
//!
//! Measured on this machine, `opt-level = 3`, 300,000 iterations each, median of
//! nine passes, before and after measured in the same session with the same
//! harness so the two columns are comparable:
//!
//! | input | before | after | |
//! | --- | --- | --- | --- |
//! | nothing matches, 36,870 bytes | 28.8 ns | **6.4 ns** | 4.5x |
//! | nothing matches, 2,048 bytes | 23.5 ns | **5.2 ns** | 4.5x |
//! | empty buffer | 16.0 ns | **3.2 ns** | 5.0x |
//! | GIF, table position 36 | 15.4 ns | **8.9 ns** | 1.7x |
//! | `SQLite`, a sixteen-byte signature | 11.7 ns | **8.8 ns** | 1.3x |
//! | `Tar` at offset 257 | 59.7 ns | **46.6 ns** | 1.3x |
//! | `ISO` at offset 36,865 | 52.0 ns | **47.9 ns** | 1.1x |
//! | PNG, JPG, ZIP -- matches at the first probe | 5.4-5.9 ns | 5.6-6.0 ns | 0.96x |
//!
//! The last row is a 2-4% regression and it is the price of the rest. Those three
//! never reach the sparse list -- their bucket matches at the first probe, and the
//! `index >= best` exit stops the sparse walk before it starts -- so they pay for
//! the extra code and nothing else. Six nanoseconds against four-and-a-half is not
//! worth the fourteen the unrecognised case gave up, and a directory scan is mostly
//! unrecognised files.
//!
//! **Zero bytes.** `node scripts/wasm_sizes.mjs` reports 45,184 raw and 18,024
//! gzipped for the binding and 32,847 for the minimal example, before and after,
//! on a clean rebuild. The three small arrays cost 98 bytes and the code they
//! replaced cost the same.
//!
//! # What did not work, and why it is written down
//!
//! Flattening the *whole* search -- one probe per `(signature, offset)` pair, all
//! of them compared as integers, including the offset-0 ones -- is 2x *slower* on
//! every bucket case. Measured, three ways:
//!
//! | shape | PNG | GIF | no match | `ISO` at 36,865 |
//! | --- | --- | --- | --- | --- |
//! | the shipped code | 5.5 ns | 15 ns | 28 ns | 49 ns |
//! | one struct per probe | 63 ns | 69 ns | 71 ns | 73 ns |
//! | three parallel arrays | 7.3 ns | 23 ns | 55 ns | 16 ns |
//!
//! The first row of that table is the reason the change above is scoped to the
//! sparse list. `SIGNATURE_KIND[0]` is one cache line that the fast libraries
//! never leave, and a four-to-eight byte `memcmp` against it costs about what three
//! bounds-checked array indexes do -- so flattening the bucket path buys nothing
//! and costs a load. The struct version is worse still because every probe
//! carried a `&'static [u8]` tail for the twenty-one signatures longer than eight
//! bytes: 32 bytes per probe, and a pointer chase, for a comparison that almost
//! never runs.
//!
//! # What it does not change
//!
//! **Nothing.** Every answer this crate gives is byte-for-byte what the linear scan
//! gave, for every possible input. That is not a hope and it is not established by
//! the tests agreeing with the old implementation on the cases they happened to
//! try; it is established by construction:
//!
//! 1. An entry is *indexable* only if every offset it declares is `0`. Such an
//!    entry can match only when `bytes[0]` equals the first byte of one of its
//!    signatures, so it is reachable only through its own bucket. A file starting
//!    with any other byte cannot match it, whatever else it contains.
//! 2. Every entry that is not indexable -- a non-zero offset, or a predicate -- goes
//!    into [`ALWAYS`] and is tried for every input, exactly as before.
//! 3. The two lists are merged by **smallest table index**, not by trying one list
//!    and then the other. `first_match` returns the minimum index that matched, so
//!    the answer is the first match *in table order* whichever list it came from.
//!
//! Point 3 is the one that matters. An index that tried the bucket first and then
//! `ALWAYS` would answer differently whenever a bucket entry sat *after* an
//! `ALWAYS` entry that also matched -- and `ScriptExecute` at 17 sits before `RAR`
//! at 18, swapped, precisely because the table's order is load-bearing. Taking the
//! minimum makes the two lists' relative order irrelevant.
//!
//! # What it costs
//!
//! Measured, and the two numbers point opposite ways, so both are here.
//!
//! **The shipped binding: 43,212 -> 44,616 bytes, +1,404, or 3.2%.** That is the
//! figure that decides anything, because `bindings/asm` is what a browser downloads.
//!
//! **The minimal example: 17,764 -> 32,847 bytes, +15,083, or 84.9%.** Same code, an
//! order of magnitude more in absolute terms, and the reason is worth knowing: that
//! module is almost entirely code, so a fixed code cost is most of it. The cost of
//! this index is code, not data -- 657 bytes of `const` tables (257 `u16` slots, 136
//! entry indices, 7 `ALWAYS` indices) and the rest is traversal. A module that
//! carries a large encoded table absorbs it; a module that carries almost nothing
//! does not.
//!
//! What buys the 1,404 bytes: 525 ns -> 31 ns on a file the table does not
//! recognise, and GIF at table position 36 from 123 ns to 21 ns. What a caller
//! short on bytes should know is that the `wasm_rust` figure is the honest worst
//! case for a size-constrained module, not the binding figure.
//!
//! Measured with `opt-level = "s"`, `lto`, one codegen unit, on this machine. A
//! caller who wants their own number has `node scripts/wasm_sizes.mjs`.
// The casts further down are unchecked, and that is a deliberate consequence
// rather than a lint suppression: `u8::try_from` and `u16::try_from` are not
// `const fn` yet, so every alternative is a hand-written comparison in each of
// them. Each bound is asserted with a `const _` further down instead, where
// exceeding it is a compile error naming the array to widen.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    reason = "u8::try_from and u16::try_from are not const fn; the bounds are asserted below"
)]

use crate::magical::match_rules::MatchRules;
use crate::magical::signatures::{Magic, SIGNATURE_KIND, TABLE};

/// The table, in const context.
///
/// `signatures::TABLE` rather than the public `SIGNATURE_KIND`, because a `const fn`
/// cannot read a `static`. The two are the same array -- `SIGNATURE_KIND` is defined
/// as `TABLE` -- so reading one is reading the other.
///
/// Only the `const fn`s below use this. [`first_match`] reads `SIGNATURE_KIND`
/// directly, and the distinction is not tidiness: a `const fn` that a runtime caller
/// reaches is compiled twice, once as a callable function and once as a const
/// evaluator, and in the `magical-asm` module that measured 15 KB of the 15 KB this
/// index added. Reading the `static` on the runtime path leaves only the small
/// evaluator behind.
const fn entry_at(index: usize) -> &'static Magic {
    &TABLE[index]
}

/// How many entries the table has, in const context.
const fn entry_count() -> usize {
    TABLE.len()
}

/// Whether `index` can be reached through a first-byte bucket.
///
/// True only when every offset it declares is `0` and it has at least one
/// signature. Both halves matter:
///
/// * a non-zero offset means the comparison reads `bytes[offset]`, which has
///   nothing to do with `bytes[0]`, so the entry cannot be indexed by it;
/// * an entry with a predicate has to be *called* to be decided. `ScriptExecute`'s
///   `#!` is a prefilter the crate uses and `WEBP` has no bytes at all, and a
///   predicate that could match a file whose first byte is something else would be
///   skipped by indexing it.
const fn is_indexable(index: usize) -> bool {
    let magic = entry_at(index);
    if magic.signatures.is_empty() {
        return false;
    }
    if !matches!(magic.rules, MatchRules::Default) {
        return false;
    }
    let mut at = 0;
    while at < magic.offsets.len() {
        if magic.offsets[at] != 0 {
            return false;
        }
        at += 1;
    }
    true
}

/// Whether `index` declares a signature whose first byte is `first`.
///
/// Called only for an indexable entry, so the offsets are all `0` and "has a
/// signature starting with this byte" is the whole question. An empty signature is
/// skipped rather than indexed: it has no first byte, and `&bytes[0..0] == &[]` is
/// true, so such an entry would otherwise claim every input.
const fn starts_with(index: usize, first: u8) -> bool {
    let magic = entry_at(index);
    let mut at = 0;
    while at < magic.signatures.len() {
        let signature = magic.signatures[at];
        if !signature.is_empty() && signature[0] == first {
            return true;
        }
        at += 1;
    }
    false
}

/// How many entries go in `first`'s bucket.
const fn bucket_size(first: u8) -> usize {
    let mut count = 0;
    let mut at = 0;
    while at < entry_count() {
        if is_indexable(at) && starts_with(at, first) {
            count += 1;
        }
        at += 1;
    }
    count
}

/// The total number of candidate slots across all 256 buckets.
const fn candidate_total() -> usize {
    let mut total = 0;
    let mut first = 0_u16;
    while first < 256 {
        total += bucket_size(first as u8);
        first += 1;
    }
    total
}

/// How many entries are tried for every input, because they are not indexable.
#[cfg(test)]
const fn always_total() -> usize {
    let mut count = 0;
    let mut at = 0;
    while at < entry_count() {
        if !is_indexable(at) {
            count += 1;
        }
        at += 1;
    }
    count
}

// Both totals are derived from the table, so adding a format re-derives the array
// lengths and a stale one cannot compile rather than being a buffer overrun.
const CANDIDATES: usize = candidate_total();
/// How many entries are not indexable: a non-zero offset, or a predicate.
///
/// Test-only, and it is worth saying why it is not a runtime constant any more.
/// The walk used to iterate one `ALWAYS` list holding all of them, so its length
/// was the loop bound. The list is now three -- [`SPARSE`] for a byte signature at
/// a non-zero offset, [`PREDICATES`] for a predicate, and the bucket for the rest
/// -- each with its own bound derived from its own array's length, so nothing in
/// the shipped path needs this number. It is kept because it is the total the
/// three lists have to account for, and a table that grew an entry none of them
/// reaches would otherwise show up as a format that is not detected rather than as
/// an arithmetic failure.
#[cfg(test)]
const ALWAYS: usize = always_total();

// The two `as` casts above are unchecked in const context -- `u16::try_from` is not
// a const fn yet -- so the bound they rely on is asserted here, where a table that
// outgrew it is a compile error naming the arrays to widen rather than a silent
// wrap.
const _: () = assert!(
    entry_count() <= u8::MAX as usize,
    "more than 255 formats: ENTRIES and UNSORTED hold a u8 index each. Widen them to u16, \
     which costs 136 bytes in a module this size."
);
const _: () = assert!(
    candidate_total() <= u16::MAX as usize,
    "more than 65,535 first-byte candidates: widen SLOTS, which holds a u16 offset each."
);

/// Where each byte's candidates start in [`ENTRIES`], plus a total at index 256.
///
/// 257 rather than 256 so a bucket's end is `SLOTS[first + 1]` and no arithmetic
/// happens per call. `ENTRIES` is grouped by byte and ascending within a group,
/// which is what makes a file whose byte is `0x00` read 4 candidates rather than
/// scanning for them.
///
/// A `static` rather than a `const`, and that is not a preference. A `const` is
/// inlined at every use site, and this one is indexed at three of them inside
/// `first_match`; making it a `static` is what guarantees a single copy in
/// read-only data rather than whatever the optimiser decides to materialise.
static SLOTS: [u16; 257] = {
    let mut slots = [0_u16; 257];
    let mut running = 0_u16;
    let mut first = 0_usize;
    while first < 256 {
        slots[first] = running;
        running += bucket_size(first as u8) as u16;
        first += 1;
    }
    slots[256] = running;
    slots
};

/// `SIGNATURE_KIND` indices, grouped by first byte.
///
/// An entry with two signatures sharing a first byte appears once per byte, not
/// twice in one bucket: trying `matches` twice would give the same answer twice and
/// the second is pure cost.
static ENTRIES: [u8; CANDIDATES] = {
    let mut out = [0_u8; CANDIDATES];
    let mut cursor = 0_usize;
    let mut first = 0_usize;
    while first < 256 {
        let mut at = 0;
        while at < entry_count() {
            if is_indexable(at) && starts_with(at, first as u8) {
                out[cursor] = at as u8;
                cursor += 1;
            }
            at += 1;
        }
        first += 1;
    }
    out
};

/// The entries with a predicate, in table order.
///
/// These are the only entries [`first_match`] still reaches through
/// `SIGNATURE_KIND[..].matches(bytes)`. Everything else that is not indexable has
/// a byte signature at a non-zero offset, and those are in [`SPARSE`] as integers.
///
/// Ascending because the `index >= best` early exit is only sound on a sorted
/// list, and `sparse_is_ascending` is the test that says so.
static PREDICATES: [u8; PREDICATE_TOTAL] = {
    let mut out = [0_u8; PREDICATE_TOTAL];
    let mut cursor = 0_usize;
    let mut at = 0;
    while at < entry_count() {
        if is_predicate(at) {
            out[cursor] = at as u8;
            cursor += 1;
        }
        at += 1;
    }
    out
};

/// Whether entry `index` is matched by calling a predicate rather than by bytes.
const fn is_predicate(index: usize) -> bool {
    !matches!(entry_at(index).rules, MatchRules::Default)
}

/// How many entries are matched by a predicate.
const fn predicate_total() -> usize {
    let mut count = 0;
    let mut at = 0;
    while at < entry_count() {
        if is_predicate(at) {
            count += 1;
        }
        at += 1;
    }
    count
}

const PREDICATE_TOTAL: usize = predicate_total();

/// The number of signature bytes compared as one integer.
///
/// Eight, because that is the width of the load and of the register the compiler
/// has for it. It is *not* the longest signature: 21 of the 150 in the table are
/// longer than this, and [`SPARSE_TOTAL`] is asserted against the count of the
/// ones that are not, so a table that grew a longer signature is a compile error
/// naming this constant rather than a signature silently truncated to a prefix.
const INLINE: usize = 8;

/// A zero-filled width, for the `try_from` that a `get` has already made infallible.
static ZERO: [u8; INLINE] = [0; INLINE];

/// `MASKS[n]` is the low `n` bytes set.
///
/// This is the whole safety argument for comparing signatures as integers, so it is
/// worth being explicit about what it prevents. A signature is zero-padded to
/// `INLINE` bytes and the input's are not, so comparing the two integers directly
/// would compare the signature's padding against the input's bytes past its own
/// signature -- and would *pass* whenever those happened to be zero, which a
/// binary file contains a great deal of. Masking the input to the signature's own
/// length is what makes the comparison mean what `bytes[a..a + len] == sig` means.
static MASKS: [u64; INLINE + 1] = {
    let mut out = [0_u64; INLINE + 1];
    let mut at = 1;
    while at <= INLINE {
        out[at] = out[at - 1] | (0xFF << (8 * (at - 1)));
        at += 1;
    }
    out
};

/// The longest signature among the entries that reach the sparse list.
const fn longest_sparse_signature() -> usize {
    let mut longest = 0;
    let mut at = 0;
    while at < entry_count() {
        if is_sparse_probe(at) {
            let mut which = 0;
            while which < entry_at(at).signatures.len() {
                let len = entry_at(at).signatures[which].len();
                if len > longest {
                    longest = len;
                }
                which += 1;
            }
        }
        at += 1;
    }
    longest
}

/// Whether entry `index` contributes a probe at a non-zero offset.
///
/// The mirror image of [`is_indexable`]: a byte signature at an offset that is not
/// `0` cannot be reached through `bytes[0]`, so it is tried for every input, and
/// it is the one case where comparing the bytes as an integer pays. A predicate
/// has no bytes to inline and stays a call.
const fn is_sparse_probe(index: usize) -> bool {
    let magic = entry_at(index);
    if magic.signatures.is_empty() || !matches!(magic.rules, MatchRules::Default) {
        return false;
    }
    let mut at = 0;
    while at < magic.offsets.len() {
        if magic.offsets[at] != 0 {
            return true;
        }
        at += 1;
    }
    false
}

/// How many `(signature, offset)` pairs reach the sparse list.
const fn sparse_total() -> usize {
    let mut total = 0;
    let mut at = 0;
    while at < entry_count() {
        if is_sparse_probe(at) {
            let magic = entry_at(at);
            let mut usable = 0;
            let mut which = 0;
            while which < magic.signatures.len() {
                if !magic.signatures[which].is_empty() && magic.signatures[which].len() <= INLINE {
                    usable += 1;
                }
                which += 1;
            }
            let mut non_zero = 0;
            let mut o = 0;
            while o < magic.offsets.len() {
                if magic.offsets[o] != 0 {
                    non_zero += 1;
                }
                o += 1;
            }
            total += usable * non_zero;
        }
        at += 1;
    }
    total
}

const SPARSE_TOTAL: usize = sparse_total();

const _: () = assert!(
    longest_sparse_signature() <= INLINE,
    "a signature reaching the sparse list is longer than the integer comparison covers. Widen \
     INLINE, or move the entry to `is_indexable` by declaring its offset as 0. Silently \
     truncating it would match a prefix of the format."
);

/// The sparse probes' signature bytes, zero-padded, as one little-endian integer.
///
/// Four parallel arrays rather than one array of a struct, and the reason is the
/// measurement: a struct holding a `u64` is padded to sixteen bytes, so the
/// alternative loads twice the bytes per probe for the same answer. Measured with
/// the struct, this table was 7x *slower* than `&SIGNATURE_KIND[index]` and a
/// `memcmp`.
static SPARSE_HEAD: [u64; SPARSE_TOTAL] = {
    let mut out = [0_u64; SPARSE_TOTAL];
    let mut cursor = 0;
    let mut at = 0;
    while at < entry_count() {
        if is_sparse_probe(at) {
            let magic = entry_at(at);
            let mut which = 0;
            while which < magic.signatures.len() {
                if !magic.signatures[which].is_empty() && magic.signatures[which].len() <= INLINE {
                    let mut head = [0_u8; INLINE];
                    let mut byte = 0;
                    while byte < magic.signatures[which].len() {
                        head[byte] = magic.signatures[which][byte];
                        byte += 1;
                    }
                    let mut o = 0;
                    while o < magic.offsets.len() {
                        if magic.offsets[o] != 0 {
                            out[cursor] = u64::from_le_bytes(head);
                            cursor += 1;
                        }
                        o += 1;
                    }
                }
                which += 1;
            }
        }
        at += 1;
    }
    out
};

/// The sparse probes' offsets, and their entries and lengths.
///
/// Ascending by entry, because [`first_match`]'s `index >= best` early exit is
/// only sound on a sorted list -- see `always_is_ascending`, which is the test
/// that says so. Entries with more than one usable signature appear more than
/// once, which is sound for the same reason `min` is: the first one that matches
/// gives that entry's index and the rest are never reached.
/// The sparse probes' offsets, entries and lengths: one walk, three arrays.
///
/// Built together rather than by three copies of the same loop, because three
/// copies of a twenty-line `const fn` that have to agree is three chances not to.
/// Their counts are `SPARSE_TOTAL` each, so a table that grew a probe out of one
/// array and not the others would not compile.
const SPARSE: Sparse = build_sparse();

/// The three views of the sparse probe list, built at compile time.
struct Sparse {
    /// Where each probe's bytes have to be in the input.
    offset: [u32; SPARSE_TOTAL],
    /// Which entry each probe came from, for the answer and the read-window filter.
    entry: [u8; SPARSE_TOTAL],
    /// How long each probe's signature is, for the mask.
    len: [u8; SPARSE_TOTAL],
}

/// Walks the table once and fills all three arrays.
///
/// Ascending by entry, because [`first_match`]'s `index >= best` early exit is
/// only sound on a sorted list -- `always_is_ascending` is the test that says so.
/// An entry with more than one usable signature appears more than once, which is
/// sound for the same reason `min` is: the first one that matches gives that
/// entry's index and the rest are never reached.
const fn build_sparse() -> Sparse {
    let mut out = Sparse {
        offset: [0_u32; SPARSE_TOTAL],
        entry: [0_u8; SPARSE_TOTAL],
        len: [0_u8; SPARSE_TOTAL],
    };
    let mut cursor = 0;
    let mut at = 0;
    while at < entry_count() {
        if is_sparse_probe(at) {
            let magic = entry_at(at);
            let mut which = 0;
            while which < magic.signatures.len() {
                let signature = magic.signatures[which];
                if !signature.is_empty() && signature.len() <= INLINE {
                    let mut o = 0;
                    while o < magic.offsets.len() {
                        if magic.offsets[o] != 0 {
                            out.offset[cursor] = magic.offsets[o] as u32;
                            out.entry[cursor] = at as u8;
                            out.len[cursor] = signature.len() as u8;
                            cursor += 1;
                        }
                        o += 1;
                    }
                }
                which += 1;
            }
        }
        at += 1;
    }
    out
}

/// How many entries a given input causes [`first_match`] to try at worst.
///
/// Exposed for the tests, and for the benchmark, because "how many entries does a
/// non-matching file cost" is the number the whole index exists to move.
#[must_use]
pub fn probes_for(bytes: &[u8]) -> usize {
    bytes
        .first()
        .map_or(SPARSE_TOTAL + PREDICATE_TOTAL, |&first| {
            usize::from(SLOTS[usize::from(first) + 1] - SLOTS[usize::from(first)])
                + SPARSE_TOTAL
                + PREDICATE_TOTAL
        })
}

/// The index of the first entry in table order that matches `bytes`.
///
/// ## Semantics
///
/// Exactly `SIGNATURE_KIND.iter().position(|magic| magic.matches(bytes))`. The
/// `allowed_max_read` filter is applied the same way `match_with_max_read_rule`
/// applies it: an entry whose own `max_bytes_read` exceeds the window cannot be
/// returned, even when the buffer holds its signature.
///
/// ## Two lists, two early exits, one minimum
///
/// Both lists are ascending, and each is walked only until its own first match:
/// within a list the first match is that list's minimum, so there is nothing to
/// gain from looking further. Then `ALWAYS` is walked only while its entries are
/// still smaller than what the bucket found, and stops at the first one that is
/// not.
///
/// That second stop is the whole reason a match does not get slower, and it is not
/// a refinement. Measured with the stop absent, a PNG header went from 5 ns to 30 ns
/// -- the no-match case was 19 times faster and the match case six times slower,
/// because a `find` landing on entry 0 still had to walk all of `ALWAYS` before the
/// two lists could be compared. `ScriptExecute` sits at 17 and the first `ALWAYS`
/// entry at 5, so for a PNG, whose bucket match is index 0, the `ALWAYS` walk stops
/// before it starts.
///
/// Note the direction of the claim: the stop is an *optimisation*, not the argument.
/// `best` is a minimum in both walks, so removing the stop costs time and answers
/// nothing differently -- `always_is_ascending` is what makes the stop sound, and
/// without it the stop would be an unchecked assumption about a sorted list.
///
/// The answer is still the minimum across both lists, which is what makes it equal
/// to the linear scan however the two interleave.
///
/// ```
/// use magical_rs::magical::dispatch::first_match;
///
/// let png = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
/// assert_eq!(first_match(&png, usize::MAX), Some(0));
///
/// // ISO 9660 declares a read size of 36,870, so a 2,048-byte window declines it
/// // even though the buffer here is long enough to hold its signature.
/// let mut iso = [0u8; 40_000];
/// iso[32_769..32_774].copy_from_slice(b"CD001");
/// assert_eq!(first_match(&iso, 36_870), Some(10));
/// assert_eq!(first_match(&iso, 2_048), None);
/// ```
#[must_use]
#[inline]
pub fn first_match(bytes: &[u8], allowed_max_read: usize) -> Option<usize> {
    let mut best: usize = usize::MAX;

    // The bucket for `bytes[0]`, if there is a first byte. An empty buffer has no
    // bucket and cannot match an offset-0 signature either, which is the answer
    // the linear scan gives for it too.
    if let Some(&first) = bytes.first() {
        let at = first as usize;
        let to = usize::from(SLOTS[at + 1]);
        let mut cursor = usize::from(SLOTS[at]);
        while cursor < to {
            let index = usize::from(ENTRIES[cursor]);
            let magic = &SIGNATURE_KIND[index];
            if magic.max_bytes_read <= allowed_max_read && magic.matches(bytes) {
                // A minimum, for the same reason the `ALWAYS` walk below uses one:
                // the ascending order already guarantees this is the smallest, so
                // `min` and assignment agree, and a future change to either cannot
                // silently start preferring a later entry.
                best = if index < best { index } else { best };
                break;
            }
            cursor += 1;
        }
    }

    // The sparse probes, compared as integers. `SPARSE` is ascending by entry, so
    // the guard below is sound: an entry at or past `best` cannot improve on it,
    // and neither can anything after it. It is an optimisation, not the correctness
    // argument -- `best` is a minimum here, so deleting the guard costs time and
    // answers nothing differently. `sparse_is_ascending` is what makes the guard
    // sound in the first place; without that test the guard would be an unchecked
    // assumption about a sorted list.
    let mut cursor = 0;
    while cursor < SPARSE_TOTAL {
        let index = usize::from(SPARSE.entry[cursor]);
        if index >= best {
            break;
        }
        let len = usize::from(SPARSE.len[cursor]);
        let at = SPARSE.offset[cursor] as usize;
        // `map_or_else` rather than a `match`, because the `None` arm has to come
        // first in a `match` and that reads as though the narrow path were the
        // common one. It is not: it needs fewer than eight bytes at the probe's
        // offset, which for a 36,870-byte buffer and an offset of 257 never
        // happens.
        let hit = bytes.get(at..at + INLINE).map_or_else(
            || {
                len <= INLINE
                    && bytes.get(at..at + len) == Some(&SPARSE_HEAD[cursor].to_le_bytes()[..len])
            },
            |wide| {
                let got = u64::from_le_bytes(<[u8; INLINE]>::try_from(wide).unwrap_or(ZERO));
                (got & MASKS[len]) == SPARSE_HEAD[cursor]
            },
        );
        if hit && SIGNATURE_KIND[index].max_bytes_read <= allowed_max_read {
            // A minimum, not an assignment. The guard already rules out anything
            // larger, but a min states the intent and holds if the guard is ever
            // loosened.
            best = index;
            break;
        }
        cursor += 1;
    }

    // The predicate entries, which have no bytes to inline and stay calls.
    let mut cursor = 0;
    while cursor < PREDICATE_TOTAL {
        let index = usize::from(PREDICATES[cursor]);
        if index >= best {
            break;
        }
        let magic = &SIGNATURE_KIND[index];
        if magic.max_bytes_read <= allowed_max_read && magic.matches(bytes) {
            best = index;
            break;
        }
        cursor += 1;
    }

    if best == usize::MAX { None } else { Some(best) }
}

#[cfg(test)]
mod tests {
    // `alloc`, not `std`. The tests build fixture buffers, and the crate is
    // `no_std` under `--no-default-features` -- which `make test-nostd` runs -- so
    // reaching for `std` here would make the whole module fail to compile in the
    // one configuration that exists to prove the crate needs no `std`.
    extern crate alloc;
    use super::{
        ALWAYS, CANDIDATES, ENTRIES, PREDICATE_TOTAL, PREDICATES, SLOTS, SPARSE, SPARSE_TOTAL,
        entry_count, first_match, is_indexable, probes_for,
    };
    use crate::magical::match_rules::MatchRules;
    use crate::magical::signatures::SIGNATURE_KIND;
    use alloc::collections::BTreeSet;
    use alloc::{vec, vec::Vec};

    /// One byte's bucket, as the indices it holds.
    ///
    /// In the test module rather than beside the tables because it allocates, and
    /// the module this code lives in is `no_std` under `--no-default-features`.
    /// `first_match` and `probes_for` read `ENTRIES` and `SLOTS` in place.
    fn bucket_of(first: usize) -> Vec<usize> {
        (usize::from(SLOTS[first])..usize::from(SLOTS[first + 1]))
            .map(|cursor| usize::from(ENTRIES[cursor]))
            .collect()
    }

    /// The linear scan, written out again on purpose.
    ///
    /// This is the thing every claim below is measured against, and it has to be a
    /// separate implementation rather than a call into the production path -- a test
    /// that compared the index against itself would agree with every mistake.
    fn linear(bytes: &[u8], allowed_max_read: usize) -> Option<usize> {
        SIGNATURE_KIND
            .iter()
            .position(|magic| magic.max_bytes_read <= allowed_max_read && magic.matches(bytes))
    }

    /// The three lists partition the table: every entry is in exactly one of them.
    ///
    /// Without this, an entry in neither would be skipped and detection would
    /// quietly lose a format, and an entry in two of them would be tried twice.
    ///
    /// Three lists rather than two, because the entries that are not indexable
    /// split: a byte signature at a non-zero offset is compared as an integer from
    /// `SPARSE`, and a predicate has no bytes to inline and stays a call.
    #[test]
    fn every_entry_is_in_exactly_one_list() {
        let mut in_bucket = [false; 256];
        for index in ENTRIES {
            in_bucket[usize::from(index)] = true;
        }
        let mut in_sparse = [false; 256];
        for index in SPARSE.entry {
            in_sparse[usize::from(index)] = true;
        }
        let mut in_predicate = [false; 256];
        for index in PREDICATES {
            in_predicate[usize::from(index)] = true;
        }

        for at in 0..entry_count() {
            let lists = [
                ("bucket", in_bucket[at]),
                ("sparse", in_sparse[at]),
                ("predicate", in_predicate[at]),
            ];
            let here = lists.iter().filter(|(_, in_it)| *in_it).count();
            assert_eq!(
                here,
                1,
                "entry {at} ({}) is in {here} of the three lists: {lists:?}",
                SIGNATURE_KIND[at].kind.variant_name(),
            );
        }

        assert_eq!(
            ENTRIES.len(),
            CANDIDATES,
            "the candidate total does not match"
        );

        // `SPARSE_TOTAL` counts *probes* and `ALWAYS` counts *entries*, so they
        // are not comparable and adding them was the bug in the first version of
        // this assertion. What has to add up is the number of distinct entries
        // the sparse list reaches, plus the predicates, against the number of
        // entries that are not indexable -- otherwise a non-indexable entry with a
        // signature the integer comparison cannot hold is in no list at all, and
        // that is a format silently not detected.
        let mut distinct: BTreeSet<usize> = SPARSE.entry.iter().map(|i| usize::from(*i)).collect();
        distinct.extend(PREDICATES.iter().map(|i| usize::from(*i)));
        assert_eq!(
            distinct.len(),
            ALWAYS,
            "{} distinct entries across the sparse and predicate lists, against {ALWAYS} \
             entries that are not indexable. An entry in neither is a format that is not \
             detected.",
            distinct.len(),
        );
    }

    /// Each bucket is ascending, so a file reads candidates in table order.
    #[test]
    fn every_bucket_is_ascending_and_disjoint() {
        for first in 0..256_usize {
            let bucket: Vec<usize> = bucket_of(first);
            let mut previous = None;
            for index in bucket {
                if let Some(previous) = previous {
                    assert!(
                        previous < index,
                        "bucket 0x{first:02x} holds {previous} then {index}, which is not ascending",
                    );
                }
                previous = Some(index);
            }
        }
        assert_eq!(usize::from(SLOTS[256]), CANDIDATES);
    }

    /// Every indexable entry is in at least one bucket, keyed by a real byte.
    ///
    /// The failure this catches is an entry whose signatures are all empty: it is
    /// indexable by the rule and reachable by no bucket, so detection would skip it
    /// and return `None` where the linear scan answers something.
    #[test]
    fn every_indexable_entry_is_reachable() {
        for (at, magic) in SIGNATURE_KIND.iter().enumerate() {
            if !is_indexable(at) {
                continue;
            }
            let reached = (0..256_usize).any(|first| bucket_of(first).contains(&at));
            assert!(
                reached,
                "entry {at} ({}) is indexable but in no bucket",
                magic.kind.variant_name(),
            );
        }
    }

    /// The sparse list is ascending by entry, which the early exit relies on.
    ///
    /// The failure this catches is silent and total: with the list unsorted, the
    /// `index >= best` break stops at the wrong place, so a match at a late entry is
    /// reported in preference to an earlier one -- which is the linear scan's answer
    /// inverted. Every differential test below would catch it, and this says why.
    ///
    /// Non-strict, because an entry with more than one usable signature appears more
    /// than once. That is sound: the first probe of an entry that matches gives that
    /// entry's index, and the rest are never reached.
    #[test]
    fn sparse_is_ascending() {
        for cursor in 1..SPARSE_TOTAL {
            assert!(
                SPARSE.entry[cursor - 1] <= SPARSE.entry[cursor],
                "the sparse list holds {} then {}, which is not ascending by entry",
                SPARSE.entry[cursor - 1],
                SPARSE.entry[cursor],
            );
        }
        for cursor in 1..PREDICATE_TOTAL {
            assert!(
                PREDICATES[cursor - 1] < PREDICATES[cursor],
                "the predicate list holds {} then {}, which is not ascending",
                PREDICATES[cursor - 1],
                PREDICATES[cursor],
            );
        }
    }

    /// Every sparse probe's entry, offset and length are the ones the walk will use.
    ///
    /// The integer comparison reads `SPARSE.len` to build its mask and `SPARSE.offset`
    /// to index, so a table where either is wrong answers something other than what
    /// it claims. And a mask of the wrong length is exactly the failure the
    /// differential tests below cannot see, because they only ever plant whole
    /// signatures -- the bytes past a short signature are never anything.
    #[test]
    fn every_sparse_probe_is_a_real_signature_at_its_own_offset() {
        for cursor in 0..SPARSE_TOTAL {
            let entry = usize::from(SPARSE.entry[cursor]);
            let offset = SPARSE.offset[cursor] as usize;
            let len = usize::from(SPARSE.len[cursor]);
            let magic = &SIGNATURE_KIND[entry];

            assert_ne!(offset, 0, "a sparse probe at offset 0 belongs in a bucket");
            assert!(
                magic.offsets.contains(&offset),
                "entry {entry} ({}) declares offsets {:?} and the probe claims {offset}",
                magic.kind.variant_name(),
                magic.offsets,
            );
            assert!(
                magic.signatures.iter().any(|s| s.len() == len),
                "entry {entry} ({}) has no signature of length {len}",
                magic.kind.variant_name(),
            );
        }
    }

    /// The slot table is a valid partition of the candidate list.
    #[test]
    fn slots_are_monotonic() {
        for first in 0..256_usize {
            assert!(
                SLOTS[first] <= SLOTS[first + 1],
                "slot {first} is {} and slot {} is {}",
                SLOTS[first],
                first + 1,
                SLOTS[first + 1],
            );
        }
    }

    /// The point of the whole thing: identical answers, everywhere.
    #[test]
    fn the_index_agrees_with_the_linear_scan_on_every_fixture() {
        let mut checked = 0;
        for (position, magic) in SIGNATURE_KIND.iter().enumerate() {
            for signature in magic.signatures {
                for &offset in magic.offsets {
                    if !matches!(magic.rules, MatchRules::Default) {
                        continue;
                    }
                    let mut buffer = vec![0_u8; offset + signature.len() + 64];
                    buffer[offset..offset + signature.len()].copy_from_slice(signature);
                    for window in [usize::MAX, magic.max_bytes_read, 64] {
                        assert_eq!(
                            first_match(&buffer, window),
                            linear(&buffer, window),
                            "entry {position} ({}), window {window}",
                            magic.kind.variant_name(),
                        );
                        checked += 1;
                    }
                }
            }
        }
        assert!(checked > 300, "only {checked} cases were checked");
    }

    /// The two predicate entries, matched on real input rather than skipped.
    ///
    /// [`the_index_agrees_with_the_linear_scan_on_every_fixture`] skips every entry
    /// with a predicate, because a predicate is not a byte pattern and there is no
    /// buffer to construct from it. That skip is a hole: the predicate entries are
    /// exactly the ones that must be reached for every input, so an index that
    /// indexed them by first byte would pass that test and answer wrongly here.
    ///
    /// Each case below is a real file header for the format, so both the index and
    /// the scan must name it, plus a near miss that must name nothing.
    #[test]
    fn the_index_agrees_with_the_linear_scan_on_predicate_entries() {
        let mut shebang = b"#!/bin/sh\n".to_vec();
        shebang.resize(512, 0);
        let mut webp = b"RIFF".to_vec();
        webp.extend_from_slice(&1_000_u32.to_le_bytes());
        webp.extend_from_slice(b"WEBPVP8 ");
        webp.resize(1_012, 0);

        // A WAV, which is `RIFF` at 0 and not `WEBP` at 8. The predicate is what
        // rejects it, so this is the case a first-byte key could not decide.
        let mut wav = b"RIFF".to_vec();
        wav.extend_from_slice(&36_u32.to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.resize(44, 0);

        for (label, buffer) in [
            ("shebang", shebang.clone()),
            ("webp", webp.clone()),
            ("wav", wav),
            ("hash without path", b"#!AMR\n".to_vec()),
        ] {
            for window in [usize::MAX, 64, 2_048] {
                assert_eq!(
                    first_match(&buffer, window),
                    linear(&buffer, window),
                    "{label}, window {window}",
                );
            }
        }

        // And the positive answers, so this test fails if a predicate entry is
        // indexed by a byte it does not require, rather than merely agreeing with
        // the scan on the near misses above.
        assert!(
            linear(&shebang, usize::MAX).is_some(),
            "the scan misses a shebang"
        );
        assert!(
            linear(&webp, usize::MAX).is_some(),
            "the scan misses a webp"
        );
    }

    /// Every predicate entry is in `ALWAYS`, and a stated reason why.
    ///
    /// Stated as a test rather than left as a comment because the failure is not a
    /// wrong answer on any input the fixtures happen to build -- it is a wrong
    /// answer on the inputs nobody thought of, which is the kind of bug that ships.
    #[test]
    fn every_predicate_entry_is_always_checked() {
        let mut checked = 0;
        for (at, magic) in SIGNATURE_KIND.iter().enumerate() {
            if matches!(magic.rules, MatchRules::Default) {
                continue;
            }
            assert!(
                !is_indexable(at),
                "entry {at} ({}) has a predicate and is indexable, so it is only tried \
                 when bytes[0] matches; a predicate is free to match any first byte",
                magic.kind.variant_name(),
            );
            assert!(
                PREDICATES.contains(&(at as u8)),
                "entry {at} ({}) has a predicate and is in no list, so it is never tried",
                magic.kind.variant_name(),
            );
            checked += 1;
        }
        assert!(checked > 0, "the table declares no predicate entry at all");
    }

    /// And on inputs that match nothing, at every window the table offers.
    #[test]
    fn the_index_agrees_with_the_linear_scan_on_non_matching_input() {
        // 0x2E is `.`, which no signature in the table starts with, and a buffer of
        // zeros starts with `0x00` which several do.
        let mut cases: Vec<Vec<u8>> = Vec::new();
        for first in [0x00_u8, 0x01, 0x2E, 0x23, 0xFF, 0x7F, 0x42, 0x50] {
            for length in [0_usize, 1, 2, 3, 16, 2_048, 36_870] {
                cases.push(vec![first; length]);
                let mut varied = vec![first; length];
                for (at, byte) in varied.iter_mut().enumerate() {
                    *byte = first ^ (at as u8);
                }
                cases.push(varied);
            }
        }
        // A tar header at 257 and an ISO one at 32,769: two of the `ALWAYS` entries a
        // first-byte index cannot reach, in buffers that also carry noise at 0.
        let mut tar = vec![0x2E_u8; 600];
        tar[257..262].copy_from_slice(b"ustar");
        cases.push(tar);
        let mut iso = vec![0x2E_u8; 40_000];
        iso[32_769..32_774].copy_from_slice(b"CD001");
        cases.push(iso);

        // The two predicate entries, which are the reason `is_indexable` refuses
        // them. Each of these must be reached for *every* input, and the ones below
        // are also cases where the first byte and the predicate disagree: a buffer
        // starting `RIFF` that is not WEBP, and a `#!` line with no path in it,
        // which is the AMR audio header rather than a script. An index that trusted
        // `bytes[0]` for these would answer `None` where the scan answers `None`
        // too -- but a buffer starting `RIFF....WEBP` must answer `WEBP` and one
        // starting `#!/bin/sh` must answer `ScriptExecute`, neither of which a
        // first-byte key alone establishes.
        let mut riff = vec![0x52_u8; 40];
        riff[0..4].copy_from_slice(b"RIFF");
        riff[8..12].copy_from_slice(b"WAVE");
        riff[4..8].copy_from_slice(&36_u32.to_le_bytes());
        cases.push(riff.clone());

        let mut webp = riff;
        webp[8..12].copy_from_slice(b"WEBP");
        webp[4..8].copy_from_slice(&1_000_u32.to_le_bytes());
        cases.push(webp);

        cases.push(b"#!/bin/sh\necho hi\n".to_vec());
        cases.push(b"#!/usr/bin/env python3\n".to_vec());
        // `#!` then no separator before the newline: not a shebang, and the AMR
        // header the doc comment above names.
        cases.push(b"#!AMR\n....".to_vec());
        // A shebang long enough that the separator is past the 128-byte scan limit.
        let mut long = b"#!/".to_vec();
        long.extend(core::iter::repeat_n(b'a', 200));
        long.extend_from_slice(b"/bin/sh\n");
        cases.push(long);

        for buffer in &cases {
            for window in [usize::MAX, 2_048, 36_870, 262] {
                assert_eq!(
                    first_match(buffer, window),
                    linear(buffer, window),
                    "buffer of {} bytes starting 0x{:02x}, window {window}",
                    buffer.len(),
                    buffer.first().copied().unwrap_or(0),
                );
            }
        }
        assert!(cases.len() > 50, "only {} cases", cases.len());
    }

    /// Deterministic pseudo-random input, so the comparison covers bytes the
    /// fixtures do not reach.
    ///
    /// A fixed LCG rather than a dependency: the point is reproducibility across
    /// machines, and a seed constant gives that for free. 4,096 buffers across
    /// every window, because the failure being guarded against is a difference on
    /// some input nobody thought of.
    #[test]
    fn the_index_agrees_with_the_linear_scan_on_pseudorandom_input() {
        let mut state = 0x2545_F491_4F6C_DD1D_u64;
        for case in 0..4_096_u64 {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let length = (state as usize) % 64;
            let buffer: Vec<u8> = (0..length)
                .map(|_| {
                    state = state
                        .wrapping_mul(6_364_136_223_846_793_005)
                        .wrapping_add(1_442_695_040_888_963_407);
                    (state >> 33) as u8
                })
                .collect();
            for window in [usize::MAX, 2_048, 36_870] {
                assert_eq!(
                    first_match(&buffer, window),
                    linear(&buffer, window),
                    "case {case}, {} bytes, window {window}",
                    buffer.len(),
                );
            }
        }
    }

    /// The number this index exists to move, asserted as a direction.
    ///
    /// Not a threshold: the absolute count moves when a format is added, and a test
    /// that fails because the table grew by one is a test that gets deleted. What
    /// must hold is that a file matching nothing costs far less than the whole
    /// table, and that the file a caller scans a directory with -- one this table
    /// does not recognise -- is the cheap case rather than the expensive one.
    #[test]
    fn a_non_matching_file_probes_a_handful_of_entries() {
        let junk = vec![0x2E_u8; 2_048];
        let probes = probes_for(&junk);

        assert!(
            probes < entry_count() / 4,
            "a non-matching file probes {probes} of {} entries, which is not the win this \
             index exists for",
            entry_count(),
        );
        assert!(
            probes < probes_for(SIGNATURE_KIND[0].signatures[0]),
            "a non-matching file probes more entries than the first table entry does",
        );
    }

    /// A file matching an early entry does not pay for the whole table either.
    ///
    /// This is the case the linear scan already handled well -- 5 ns for PNG -- and
    /// it must not get worse. An index that added a bucket lookup in front of a
    /// single `find` would have saved the non-match case and taxed the match case.
    #[test]
    fn an_early_match_is_not_slower_to_probe() {
        let total = entry_count();
        let png = SIGNATURE_KIND[0].signatures[0];
        assert!(
            probes_for(png) < total / 4,
            "PNG probes {} of {total} entries",
            probes_for(png),
        );
    }
}
