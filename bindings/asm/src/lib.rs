//! The WebAssembly module behind the `@reim-developer/magical-js` package.
//!
//! # Where the two halves are
//!
//! This crate is the Rust half and it ships nothing of its own: it is not
//! published to crates.io, and it is not the npm package either. The package is
//! `bindings/nodejs` — this file's JavaScript half — and `scripts/build.mjs` in
//! there compiles this crate and copies the artifact to `magical_js.wasm` beside
//! the loader that instantiates it.
//!
//! Two directories rather than one, because they are two things that change for
//! different reasons. This side changes when the crate's API or the memory ABI
//! does; that side changes when the public JavaScript API or the generated format
//! tables do. And they are versioned apart: the crate here says `0.1.0` and is
//! not published anywhere, while the npm package has its own version and its own
//! release workflow. Putting them in one crate would tie the two version numbers
//! together for no benefit.
//!
//! What they *cannot* be apart is the module and its loader. The memory ABI below
//! is a contract between this file and `_wasm.js`, and a package shipping one
//! without the other would install cleanly and throw on its first call.
//!
//! # What this is, and what it deliberately is not
//!
//! Every function here is a raw `extern "C"` export. There is no `wasm-bindgen`,
//! no `wasm-pack`, and no generated glue: the module declares **no imports at
//! all**, so a loader instantiates it with an empty import object and reads the
//! bytes out of an exported `memory`. That was measured rather than assumed —
//! `WebAssembly.Module.imports` on the built module returns `[]`.
//!
//! The alternative was `wasm-bindgen`, which was rejected for a reason worth
//! writing down. It would add a toolchain to CI that this repository does not
//! have, it requires a generated `bg.js` beside the module, and it *generates*
//! the TypeScript declarations — which is the one thing this binding cannot use,
//! because the public API is hand-written to carry generics that inference
//! produces. Owning the type surface means owning the memory ABI too.
//!
//! # The memory contract
//!
//! Byte input crosses the boundary as a pointer and a length. The caller writes
//! into linear memory and passes the address; this side never allocates a buffer
//! for the caller, and never retains the pointer past the call. Two rules the
//! JavaScript side must follow, both of which are easy to get wrong:
//!
//! 1. `memory.buffer` is detached and replaced whenever memory grows, so a
//!    `Uint8Array` captured at load time becomes a zero-length view. The loader
//!    rebuilds the view on every call rather than caching one.
//! 2. A write at offset 0 can be past the end of a small initial memory, so the
//!    loader grows it first and re-derives the view afterwards.
//!
//! # What is not here
//!
//! Levels 3 and 4 of the crate decide rules by calling back into a host
//! language. Doing that from wasm requires the module to *import* a function,
//! which is exactly the property worth giving up: a module with no imports loads
//! from any byte array with no glue. JavaScript-side predicates are a different
//! API rather than a port, so this binding covers levels 1 and 2 and the
//! README says so plainly.

// The same lint groups the library crate and the Python binding's crate deny, so
// every crate in this repository is held to one bar and a stricter standard never
// drifts in one direction. `bindings/nodejs/scripts/gates.sh` also passes them on
// the clippy command line; the attribute is here so that `cargo build` alone is
// enough, and so that the two cannot disagree about which lints are on.
#![deny(clippy::pedantic, clippy::all, clippy::nursery, clippy::perf)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::OnceLock;

use magical_rs::magical::bytes_read::{
    DEFAULT_MAX_BYTES_READ, DEFAULT_OFFSET, ISO_MAX_BYTES_READ, ISO_OFFSETS, TAR_MAX_BYTES_READ,
    TAR_OFFSETS, with_bytes_read,
};
use magical_rs::magical::dispatch;
use magical_rs::magical::magic::FileKind;
use magical_rs::magical::magic_custom::{CustomMatchRules, MagicCustom, match_types_custom};
use magical_rs::magical::match_rules::MatchRules;
use magical_rs::magical::signatures::SIGNATURE_KIND;

/// The value every "no match" answer uses.
///
/// A rule index is a `usize`, so this is out of range for any real rule set and
/// cannot collide with one.
///
/// # It reads back as -1 in JavaScript
///
/// WebAssembly's JavaScript API converts an `i32` *result* with `ToInt32`, that
/// is, as a signed value, so `u32::MAX` arrives in JavaScript as `-1` rather than
/// as `4294967295`. `_wasm.js` therefore compares against `-1`. Nothing else
/// crosses as a number large enough to notice: a `FileKind` discriminant is under
/// 114 and a rule index is under the rule count.
///
/// This is recorded here as well as there because either comment read on its own
/// is enough to get it wrong, and the failure is silent — a comparison against
/// `0xffffffff` never matches, which turns "no match" into "the 4294967295th
/// format" and yields `undefined` where the API promises `null`.
const NO_MATCH: u32 = u32::MAX;

// ---------------------------------------------------------------------------
// Level 1: the built-in table
// ---------------------------------------------------------------------------

/// Detect `len` bytes at `ptr`, or return [`NO_MATCH`].
///
/// The return value is a `FileKind` discriminant, which is what makes the enum's
/// declaration order part of this ABI. `FileKind` is a fieldless enum, so
/// `FileKind::Png as i32` is 0 and stays 0 as long as `Png` stays first in
/// `src/magical/magic.rs`. That is a load-bearing assumption, so the JavaScript
/// generator's output is checked against the built table rather than trusted:
/// `test/kinds.test.js` compares all 114 names against what this module reports.
///
/// # Safety
///
/// `ptr` must describe `len` initialized bytes inside this module's linear
/// memory, and the caller must not mutate them until the call returns.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn which_kind_at(ptr: *const u8, len: usize) -> u32 {
    // SAFETY: forwarded from this function's own contract.
    let bytes = unsafe { borrow(ptr, len) };
    FileKind::match_types(bytes).map_or(NO_MATCH, |kind| narrow(kind as usize))
}

/// Detect, ignoring every rule whose own read size exceeds `max_bytes_read`.
///
/// This reproduces `FileKind::match_with_max_read_rule`, which the crate compiles
/// only under `not(feature = "std")` and this binding cannot use because it needs
/// `std`. The body is the same expression the crate's is, and the comment there
/// is the reason it is spelled out rather than delegated: filtering first and
/// finding second is what makes a narrow window drop the expensive rules instead
/// of merely failing to reach them.
///
/// # Safety
///
/// `ptr` must describe `len` initialized bytes inside this module's linear
/// memory, and the caller must not mutate them until the call returns.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn which_kind_max_at(
    ptr: *const u8,
    len: usize,
    max_bytes_read: usize,
) -> u32 {
    // SAFETY: forwarded from this function's own contract.
    let bytes = unsafe { borrow(ptr, len) };
    // The crate's own dispatch index rather than a scan of the table. It answers
    // the same question with the same rule -- `first_match` returns the first entry
    // in `SIGNATURE_KIND` order whose `max_bytes_read` fits the window and whose
    // signature matches, and the crate's differential tests hold it to exactly the
    // scan this replaces. The scan was written out here because the crate only
    // compiles `match_with_max_read_rule` under `no_std`; the index is public and
    // unconditional, so there is no reason to keep reimplementing the loop.
    dispatch::first_match(bytes, max_bytes_read).map_or(NO_MATCH, |index| {
        narrow(SIGNATURE_KIND[index].kind as usize)
    })
}

/// Report whether one named format's own rule matches, ignoring the table.
///
/// `kind` is a discriminant, and an unknown one reports `false` rather than
/// trapping: the JavaScript side validates names against its own generated table
/// and this is the backstop for a name that got past it. "No such format" and
/// "did not match" are different answers, and the JavaScript layer raises for
/// the first before it ever gets here.
///
/// # Safety
///
/// `ptr` must describe `len` initialized bytes inside this module's linear
/// memory, and the caller must not mutate them until the call returns.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kind_matches_at(kind: u32, ptr: *const u8, len: usize) -> bool {
    // SAFETY: forwarded from this function's own contract.
    let bytes = unsafe { borrow(ptr, len) };
    SIGNATURE_KIND
        .iter()
        .find(|magic| magic.kind as u32 == kind)
        .is_some_and(|magic| magic.matches(bytes))
}

/// Build a view over bytes the caller wrote into linear memory.
///
/// # Safety
///
/// `ptr` must describe `len` initialized bytes inside this module's linear
/// memory, and the caller must not mutate them for the duration of the call.
/// Every caller here is the JavaScript loader, which writes the bytes immediately
/// before calling and reads nothing during. The returned slice does not outlive
/// the call.
#[inline]
const unsafe fn borrow<'a>(ptr: *const u8, len: usize) -> &'a [u8] {
    // `from_raw_parts` requires a non-null, aligned pointer even for a
    // zero-length slice, and the loader passes `0` for an empty `Uint8Array`
    // because offset 0 is where it copies input to. Returning `&[]` for the
    // empty case is what makes that call well defined rather than undefined
    // behaviour that happens to work.
    if len == 0 {
        return &[];
    }
    // SAFETY: forwarded from this function's own contract, and the empty case
    // above already returned.
    unsafe { std::slice::from_raw_parts(ptr, len) }
}

// ---------------------------------------------------------------------------
// The table, as one blob
// ---------------------------------------------------------------------------

/// Identifier at the head of the blob, so a mismatched pair fails loudly.
const BLOB_MAGIC: &[u8; 4] = b"MTB1";

/// Index into the blob's constant block.
///
/// A flat block of `u32` rather than a dozen exports, because a dozen exports is
/// a dozen chances for the JavaScript side to spell one of them wrong and get a
/// number that looks plausible. One blob, one pointer, one length, and a decoder
/// that reads the layout out of this file.
mod constant {
    pub const DEFAULT_MAX_BYTES_READ: usize = 0;
    pub const BYTES_READ: usize = 1;
    pub const DEFAULT_OFFSET: usize = 2;
    pub const ISO_MAX_BYTES_READ: usize = 3;
    pub const TAR_MAX_BYTES_READ: usize = 4;
    pub const COUNT: usize = 5;
}

/// The whole table and the crate's read limits, encoded once into one buffer.
///
/// Handed to JavaScript as a pointer and a length rather than as a live view of
/// `Magic`, because reading `Magic` across the boundary would mean depending on
/// `#[repr(Rust)]` field order, which is not a stable layout and would break
/// silently on a compiler update instead of loudly. This encoding is written out
/// by hand, byte for byte, and `test/table.test.js` decodes it and checks the
/// result against the crate's own numbers.
fn blob() -> &'static [u8] {
    static BLOB: OnceLock<Vec<u8>> = OnceLock::new();
    BLOB.get_or_init(build_blob)
}

fn build_blob() -> Vec<u8> {
    let mut out = Vec::with_capacity(4096);

    // Written *through* the indices rather than as a bare array, so the
    // `constant` module is load-bearing instead of a comment. A reordered index
    // would otherwise hand JavaScript a plausible wrong number — a read size of
    // 2048 where 36,870 belongs — and every symptom would look like a file being
    // truncated.
    let mut constants = [0u32; constant::COUNT];
    constants[constant::DEFAULT_MAX_BYTES_READ] = narrow(DEFAULT_MAX_BYTES_READ);
    constants[constant::BYTES_READ] = narrow(with_bytes_read());
    constants[constant::DEFAULT_OFFSET] = narrow(DEFAULT_OFFSET);
    constants[constant::ISO_MAX_BYTES_READ] = narrow(ISO_MAX_BYTES_READ);
    constants[constant::TAR_MAX_BYTES_READ] = narrow(TAR_MAX_BYTES_READ);

    out.extend_from_slice(BLOB_MAGIC);
    push_u32(&mut out, narrow(constant::COUNT));
    for value in constants {
        push_u32(&mut out, value);
    }

    push_offsets(&mut out, ISO_OFFSETS);
    push_offsets(&mut out, TAR_OFFSETS);

    push_u32(&mut out, narrow(SIGNATURE_KIND.len()));
    for magic in SIGNATURE_KIND {
        // The discriminant is stored per entry rather than implied by position,
        // because position and discriminant are not the same thing. Measured: 71 of
        // these 114 entries are not in declaration order, and the two that shadow
        // each other are swapped — `ScriptExecute` (18) sits at position 17, `RAR`
        // (17) at position 18. A decoder that indexed by position would attribute
        // every rule to the wrong format, and for that pair it would quietly report
        // the wrong winner. `SIGNATURE_KIND`'s order is the detection loop's
        // business; this encoding deliberately does not inherit it.
        push_u32(&mut out, narrow(magic.kind as usize));
        // One flag today, in bit 0. A wider field is here so a second flag is an
        // additive change rather than a layout break for a decoder already
        // shipped.
        let flags = u32::from(matches!(magic.rules, MatchRules::WithFn(_)));
        push_u32(&mut out, flags);
        push_u32(&mut out, narrow(magic.max_bytes_read));
        push_offsets(&mut out, magic.offsets);
        push_u32(&mut out, narrow(magic.signatures.len()));
        for signature in magic.signatures {
            push_u32(&mut out, narrow(signature.len()));
            out.extend_from_slice(signature);
        }
    }

    out
}

fn push_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_offsets(out: &mut Vec<u8>, offsets: &[usize]) {
    push_u32(out, narrow(offsets.len()));
    for offset in offsets {
        push_u32(out, narrow(*offset));
    }
}

/// Fit a `usize` into the `u32` the blob speaks.
///
/// On `wasm32-unknown-unknown` this is the identity, which is the only place the
/// module actually runs. It is written as a real conversion rather than an `as`
/// cast because the crate is also compiled for the host, where the `rlib` half
/// of `crate-type` exists and `usize` is 64 bits wide. A silent truncation there
/// would be a blob whose read sizes are wrong by a factor of four billion, so the
/// unreachable case saturates instead, and `narrow_saturates_rather_than_wrapping`
/// says so.
fn narrow(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

/// Widen the `u32` JavaScript speaks back into the `usize` the crate uses.
///
/// The mirror of [`narrow`], and saturating for the same reason: on the host the
/// `rlib` half of `crate-type` exists, and `u32 as usize` on a 32-bit target is
/// free while on a 64-bit one it would be a truncation nobody would notice.
fn widen(value: u32) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

/// Address of the encoded table. Valid for the life of the module.
#[unsafe(no_mangle)]
pub extern "C" fn table_ptr() -> *const u8 {
    blob().as_ptr()
}

/// Length of the encoded table, in bytes.
#[unsafe(no_mangle)]
pub extern "C" fn table_len() -> u32 {
    narrow(blob().len())
}

// ---------------------------------------------------------------------------
// Scratch buffers
// ---------------------------------------------------------------------------

// One input buffer and one answer buffer, reused and grown on demand.
//
// They exist because address 0 is *not* free. Linear memory holds the module's
// static data and its allocator's heap as well as the caller's bytes, and the two
// side-by-side layout puts `SIGNATURE_KIND` and the blob high up in the initial
// memory — `table_ptr()` reports 1,114,120 in a module whose memory starts at
// 1,114,112 bytes. A caller handing over a one-megabyte file therefore lands
// squarely in the middle of the detection table if the loader writes at offset 0.
//
// That corruption is silent, which is what makes it worth this. The module keeps
// returning answers; they are simply answers from a shredded table. And it only
// appears for buffers large enough to reach the data, so a test suite built from
// short fixtures never sees it — the failure arrives in somebody's production
// directory scan and nowhere else.
//
// So the module allocates, and the loader asks where to write.
thread_local! {
    static INPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static ANSWERS: RefCell<Vec<u32>> = const { RefCell::new(Vec::new()) };
}

/// Grow the input buffer to hold `len` bytes and return its address.
///
/// Never null, and never dangling: an empty buffer is still grown to one byte,
/// because `slice::from_raw_parts` rejects a null pointer even for a zero-length
/// slice and the loader must be able to ask for zero bytes.
///
/// The address is valid until the next call to this export, which may reallocate
/// and move it. The loader re-asks on every call rather than caching, because a
/// cached one is a pointer into freed memory after the second file of a
/// different size.
#[unsafe(no_mangle)]
pub extern "C" fn input_ptr(len: u32) -> *mut u8 {
    INPUT.with(|input| {
        let mut input = input.borrow_mut();
        input.resize(widen(len).saturating_add(1), 0);
        input.as_mut_ptr()
    })
}

/// Grow the answer buffer to hold `count` `u32`s and return its address.
///
/// Sized by the caller from `rules_len`, which is the upper bound on how many
/// rules can match. Like `input_ptr`, at least one element is always allocated so
/// the pointer is never null.
#[unsafe(no_mangle)]
pub extern "C" fn answer_ptr(count: u32) -> *mut u32 {
    ANSWERS.with(|answers| {
        let mut answers = answers.borrow_mut();
        answers.resize(widen(count).max(1), 0);
        answers.as_mut_ptr()
    })
}

// ---------------------------------------------------------------------------
// Level 2: rules built at runtime
// ---------------------------------------------------------------------------

/// A finished rule set.
///
/// The `&'static` slices are what `MagicCustom` demands, and a slice pointing
/// into another allocation cannot be `'static`. They are reached by leaking
/// boxed allocations once per finished rule set, which is a deliberate trade: a
/// rule set in JavaScript is a thing you build at startup and keep, not
/// something you build per file. The alternative is rebuilding it on every
/// `matchTypesCustom` call, which allocates per call to save bytes that a
/// process holding a few rule sets will never notice.
struct RuleSet {
    rules: &'static [MagicCustom<'static, u32>],
}

// Rule sets under construction, keyed by the handle the caller holds. Thread
// local rather than global: the module is instantiated once per isolate and
// there is no thread to share across in the only environment it runs in, so a
// `Mutex` would be paying for a concurrency model this binding does not have.
//
// `NEXT_HANDLE` wraps past 0 and back to 1 so that 0 keeps meaning "no handle".
// A collision with a live handle would silently replace another caller's rule
// set, so it is worth saying out loud: it takes four billion rule sets in one
// process, and each is consumed at `rules_finish` before the next is handed out.
thread_local! {
    static BUILDING: RefCell<HashMap<u32, Builder>> = RefCell::new(HashMap::new());
    static FINISHED: RefCell<HashMap<u32, RuleSet>> = RefCell::new(HashMap::new());
    static NEXT_HANDLE: RefCell<u32> = const { RefCell::new(1) };
}

/// One rule being built: any signature at any offset, which is the `Default` arm
/// of the crate's level 2 and the only one reachable without a host callback.
#[derive(Default)]
struct BuildingRule {
    signatures: Vec<Vec<u8>>,
    offsets: Vec<usize>,
}

impl BuildingRule {
    fn is_empty(&self) -> bool {
        self.signatures.is_empty() && self.offsets.is_empty()
    }

    /// A rule with no signature, or none to compare at, can never match. The two
    /// are checked separately because either one alone is enough to make the rule
    /// dead, and a dead rule is worse than no rule: it looks configured.
    fn is_complete(&self) -> bool {
        !self.signatures.is_empty() && !self.offsets.is_empty()
    }
}

/// A rule set being built, one rule at a time.
#[derive(Default)]
struct Builder {
    rules: Vec<BuildingRule>,
}

impl Builder {
    /// The rule currently being built, created on first use.
    fn current(&mut self) -> &mut BuildingRule {
        if self.rules.is_empty() {
            self.rules.push(BuildingRule::default());
        }
        self.rules.last_mut().expect("just pushed when empty")
    }
}

/// Start a new rule set and return its handle.
///
/// Handles start at 1 so that 0 can mean "none" without a second convention.
#[unsafe(no_mangle)]
pub extern "C" fn rules_new() -> u32 {
    NEXT_HANDLE.with(|next| {
        let mut next = next.borrow_mut();
        let handle = *next;
        *next = next.wrapping_add(1).max(1);
        BUILDING.with(|building| {
            building.borrow_mut().insert(handle, Builder::default());
        });
        handle
    })
}

/// Append a signature to the rule currently being built.
///
/// # Safety
///
/// `ptr` must describe `len` initialized bytes inside this module's linear
/// memory. The bytes are copied before the call returns, so the caller need only
/// keep them alive for the duration of the call, not after.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rules_add_signature(handle: u32, ptr: *const u8, len: usize) {
    // SAFETY: this function's own contract.
    let bytes = unsafe { borrow(ptr, len) };
    BUILDING.with(|building| {
        if let Some(builder) = building.borrow_mut().get_mut(&handle) {
            builder.current().signatures.push(bytes.to_vec());
        }
    });
}

/// Append an offset to the rule currently being built.
#[unsafe(no_mangle)]
pub extern "C" fn rules_add_offset(handle: u32, offset: u32) {
    BUILDING.with(|building| {
        if let Some(builder) = building.borrow_mut().get_mut(&handle) {
            builder.current().offsets.push(widen(offset));
        }
    });
}

/// Close the rule being built and start the next one.
///
/// Called *between* rules rather than after the last one, so the trailing empty
/// rule this leaves behind is dropped by `rules_finish` instead of having to be
/// rejected.
#[unsafe(no_mangle)]
pub extern "C" fn rules_next_rule(handle: u32) {
    BUILDING.with(|building| {
        if let Some(builder) = building.borrow_mut().get_mut(&handle) {
            builder.rules.push(BuildingRule::default());
        }
    });
}

/// Finish a rule set and make it usable.
///
/// Returns `false` for an empty set, a set with no rules, or a rule with no
/// signature or no offset. Those are rejected here rather than left to match
/// nothing forever, because a rule set that can never match is
/// indistinguishable from a correct one at the call site.
#[unsafe(no_mangle)]
pub extern "C" fn rules_finish(handle: u32) -> bool {
    let Some(mut builder) = BUILDING.with(|building| building.borrow_mut().remove(&handle)) else {
        return false;
    };

    // A `rules_next_rule` after the last rule leaves an empty one behind.
    if builder.rules.last().is_some_and(BuildingRule::is_empty) {
        builder.rules.pop();
    }
    if builder.rules.is_empty() || builder.rules.iter().any(|rule| !rule.is_complete()) {
        return false;
    }

    FINISHED.with(|finished| {
        finished.borrow_mut().insert(
            handle,
            RuleSet {
                rules: leak_rules(&builder.rules),
            },
        );
    });
    true
}

/// Turn built rules into the `&'static` shape `MagicCustom` requires.
///
/// The leaking is two levels deep and both are load-bearing. Each signature's
/// bytes are leaked first so a `&'static [u8]` exists to point at; then the
/// vector of those pointers is leaked, so a *group* of them — one rule's
/// signatures — can itself be a `&'static [&'static [u8]]` without borrowing
/// something local. Leaking only the inner allocations would leave the outer
/// slices borrowing a `Vec` that drops at the end of this function.
fn leak_rules(rules: &[BuildingRule]) -> &'static [MagicCustom<'static, u32>] {
    let mut pointers: Vec<&'static [u8]> = Vec::new();
    for rule in rules {
        for signature in &rule.signatures {
            pointers.push(Box::leak(signature.clone().into_boxed_slice()));
        }
    }
    let all_pointers: &'static [&'static [u8]] = Box::leak(pointers.into_boxed_slice());

    let mut built: Vec<MagicCustom<'static, u32>> = Vec::with_capacity(rules.len());
    let mut start = 0;
    for (index, rule) in rules.iter().enumerate() {
        let end = start + rule.signatures.len();
        let offsets: &'static [usize] = Box::leak(rule.offsets.clone().into_boxed_slice());
        built.push(MagicCustom {
            signatures: &all_pointers[start..end],
            offsets,
            // Unused by `match_types_custom`, which walks the rules in order and
            // returns the first match. Set to the rule's own read size so the
            // value is not a lie if a future version of the crate reads it.
            max_bytes_read: offsets.iter().copied().max().unwrap_or(0),
            // The index is the answer. A custom rule set names its own kinds in
            // JavaScript, and keeping this side ignorant of those names is what
            // lets one rule set serve any `kind` strings.
            kind: narrow(index),
            rules: CustomMatchRules::Default,
        });
        start = end;
    }

    Box::leak(built.into_boxed_slice())
}

/// Drop a finished rule set.
///
/// The allocations it was built from were leaked at `rules_finish`, so this
/// releases the handle and not the memory. The name says so rather than
/// pretending otherwise, because a `release` that does not release is the kind
/// of name that costs somebody an afternoon.
#[unsafe(no_mangle)]
pub extern "C" fn rules_release(handle: u32) {
    FINISHED.with(|finished| {
        finished.borrow_mut().remove(&handle);
    });
}

/// Match and return the index of the first rule that matched, or [`NO_MATCH`].
///
/// The answer is a rule index rather than a `FileKind`, because a custom rule set
/// names its own kinds. Handing back an index keeps this side ignorant of those
/// names, which is what lets the same rule set be reused under any `kind` strings
/// at all.
///
/// # Safety
///
/// `ptr` must describe `len` initialized bytes inside this module's linear
/// memory, and the caller must not mutate them until the call returns.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rules_match(handle: u32, ptr: *const u8, len: usize) -> u32 {
    // SAFETY: this function's own contract.
    let bytes = unsafe { borrow(ptr, len) };
    FINISHED.with(|finished| {
        let held = finished.borrow();
        let Some(set) = held.get(&handle) else {
            return NO_MATCH;
        };
        match_types_custom(bytes, set.rules, NO_MATCH)
    })
}

/// Match and return every rule that matched, writing indices to `out_ptr`.
///
/// Returns the number written, or [`NO_MATCH`] for a closed handle.
///
/// The crate has no "match all" of its own, so this calls the crate's matcher
/// once per rule over a one-rule slice. That is deliberate: a second
/// implementation of the same question is a second set of answers, and the point
/// of the level 2 API is that the crate's rule is the rule.
///
/// # Safety
///
/// `ptr` must describe `len` initialized bytes inside this module's linear
/// memory, and the caller must not mutate them until the call returns. `out_ptr`
/// must have room for `rules_len(handle)` `u32`s, which is the upper bound on how
/// many rules can match.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rules_match_all(
    handle: u32,
    ptr: *const u8,
    len: usize,
    out_ptr: *mut u32,
) -> u32 {
    // SAFETY: this function's own contract.
    let bytes = unsafe { borrow(ptr, len) };
    let matched = FINISHED.with(|finished| {
        let held = finished.borrow();
        let set = held.get(&handle)?;
        Some(
            set.rules
                .iter()
                .enumerate()
                .filter(|(_, rule)| {
                    match_types_custom(bytes, std::slice::from_ref(rule), NO_MATCH) != NO_MATCH
                })
                .map(|(index, _)| narrow(index))
                .collect::<Vec<u32>>(),
        )
    });
    let Some(matched) = matched else {
        return NO_MATCH;
    };
    // SAFETY: this function's own contract; the caller sized `out_ptr` for
    // `rules_len` entries, which is the upper bound on how many can match.
    let out = unsafe { std::slice::from_raw_parts_mut(out_ptr, matched.len()) };
    out.copy_from_slice(&matched);
    narrow(matched.len())
}

/// How many rules a handle holds, which is also the size of the buffer
/// `rules_match_all` writes into.
#[unsafe(no_mangle)]
pub extern "C" fn rules_len(handle: u32) -> u32 {
    FINISHED.with(|finished| {
        finished
            .borrow()
            .get(&handle)
            .map_or(0, |set| narrow(set.rules.len()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // The whole of this module's pointer contract is "these bytes are in linear
    // memory and stay put for the duration of the call". The shims below are where
    // that is asserted, once, instead of at twenty call sites where twenty copies
    // of the same reasoning would check nothing extra.
    //
    // They copy through `input_ptr` rather than handing over a `&'static` slice,
    // which is deliberate: that is exactly what `_wasm.js` does, so a test and the
    // loader run the same sequence of pointer and length. A shim that passed a
    // static slice would keep passing after a change to the buffer discipline,
    // which is the change most likely to be wrong.

    /// Copy `bytes` into the module's input buffer, as the loader does.
    fn with_input<R>(bytes: &[u8], call: impl FnOnce(*const u8) -> R) -> R {
        let ptr = input_ptr(narrow(bytes.len()));
        // SAFETY: `input_ptr` just grew the buffer to hold `bytes`, and the copy
        // is what makes the memory initialized.
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len()) };
        call(ptr)
    }

    fn add_signature(handle: u32, bytes: &[u8]) {
        with_input(bytes, |ptr| {
            // SAFETY: `ptr` and `bytes.len()` describe initialized bytes in this
            // module's memory, and the export copies before returning.
            unsafe { rules_add_signature(handle, ptr, bytes.len()) }
        });
    }

    fn match_rule(handle: u32, bytes: &[u8]) -> u32 {
        with_input(bytes, |ptr| {
            // SAFETY: as above; this one only reads.
            unsafe { rules_match(handle, ptr, bytes.len()) }
        })
    }

    fn kind_matches(kind: u32, bytes: &[u8]) -> bool {
        with_input(bytes, |ptr| {
            // SAFETY: as above.
            unsafe { kind_matches_at(kind, ptr, bytes.len()) }
        })
    }

    fn which_kind(bytes: &[u8]) -> u32 {
        with_input(bytes, |ptr| {
            // SAFETY: as above.
            unsafe { which_kind_at(ptr, bytes.len()) }
        })
    }

    fn which_kind_max(bytes: &[u8], max_bytes_read: usize) -> u32 {
        with_input(bytes, |ptr| {
            // SAFETY: as above.
            unsafe { which_kind_max_at(ptr, bytes.len(), max_bytes_read) }
        })
    }

    /// `rules_match_all` with the module's answer buffer, sized from `rules_len`.
    ///
    /// Sizing the buffer from the module's own rule count rather than from a
    /// hand-picked constant is the point: it is the same arithmetic the
    /// JavaScript loader does, so an off-by-one here is the same bug there.
    fn match_all(handle: u32, bytes: &[u8]) -> Vec<u32> {
        let answers = answer_ptr(rules_len(handle));
        with_input(bytes, |ptr| {
            // SAFETY: `answers` was just sized by `answer_ptr` from
            // `rules_len(handle)`, which is the upper bound on how many rules can
            // match, and `ptr` is live across the call.
            let count = unsafe { rules_match_all(handle, ptr, bytes.len(), answers) };
            if count == NO_MATCH {
                return Vec::new();
            }
            // A fresh read after the call: `rules_match_all` allocates, so the
            // answer buffer may have moved and the caller's view of it is stale.
            // SAFETY: `answers` still points at `count` initialized `u32`s.
            unsafe { std::slice::from_raw_parts(answers, count as usize) }.to_vec()
        })
    }

    /// A custom rule set, built through the same exports the loader uses.
    fn build_isolated_png() -> u32 {
        let handle = rules_new();
        add_signature(handle, b"\x89PNG\r\n\x1a\n");
        rules_add_offset(handle, 0);
        assert!(rules_finish(handle));
        handle
    }

    fn read_u32(bytes: &[u8], at: &mut usize) -> u32 {
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&bytes[*at..*at + 4]);
        *at += 4;
        u32::from_le_bytes(buf)
    }

    fn skip_offsets(bytes: &[u8], mut at: usize) -> usize {
        let count = read_u32(bytes, &mut at);
        at += count as usize * 4;
        at
    }

    #[test]
    fn the_crates_own_read_limits_survive_the_round_trip() {
        let bytes = blob();
        let mut at = 4;
        assert_eq!(&bytes[0..4], BLOB_MAGIC);
        assert_eq!(read_u32(bytes, &mut at) as usize, constant::COUNT);

        // Read positionally and compared against the crate's own values, which
        // pins the blob's order to the `constant` indices. If the two ever
        // disagree this fails on the slot number, rather than on a file that
        // mysteriously detects as the wrong type.
        for (slot, expected) in [
            DEFAULT_MAX_BYTES_READ,
            with_bytes_read(),
            DEFAULT_OFFSET,
            ISO_MAX_BYTES_READ,
            TAR_MAX_BYTES_READ,
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(
                read_u32(bytes, &mut at) as usize,
                expected,
                "constant slot {slot}"
            );
        }
    }

    #[test]
    fn the_blob_is_exactly_as_long_as_its_contents_claim() {
        // Walked to the very end. A decoder that reads one field too few looks
        // like a correct decoder until the last entry, and this is the assertion
        // that would have caught it.
        let bytes = blob();
        let mut at = 4;
        assert_eq!(read_u32(bytes, &mut at) as usize, constant::COUNT);
        at += constant::COUNT * 4;
        at = skip_offsets(bytes, at);
        at = skip_offsets(bytes, at);

        let count = read_u32(bytes, &mut at);
        assert_eq!(count as usize, SIGNATURE_KIND.len());
        for _ in SIGNATURE_KIND {
            at += 4; // kind discriminant
            at += 4; // flags
            at += 4; // max_bytes_read
            at = skip_offsets(bytes, at);
            let signatures = read_u32(bytes, &mut at);
            for _ in 0..signatures {
                let len = read_u32(bytes, &mut at) as usize;
                at += len;
            }
        }
        assert_eq!(at, bytes.len(), "the decoder and the encoder disagree");
    }

    /// What a predicate entry reports, measured rather than assumed.
    ///
    /// The first version of this test asserted that a predicate entry has no
    /// signatures, on the strength of how the Python binding presents them. That
    /// is wrong for one of the two: `ScriptExecute` carries `#!` as a prefilter
    /// *and* the predicate that a path separator follows on the same line, while
    /// `WEBP` has neither. So this asserts what is actually true, which is
    /// sharper than the assumption was: exactly one entry in the table has no
    /// signatures at all, and it is a predicate.
    #[test]
    fn a_predicate_entry_may_still_carry_a_signature() {
        let predicate: Vec<_> = SIGNATURE_KIND
            .iter()
            .filter(|magic| matches!(magic.rules, MatchRules::WithFn(_)))
            .collect();
        assert_eq!(
            predicate.len(),
            2,
            "a third predicate entry would change what `usesPredicate` means here"
        );
        assert_eq!(
            predicate
                .iter()
                .filter(|magic| !magic.signatures.is_empty())
                .count(),
            1,
            "exactly one of the two is a pure predicate with nothing to prefilter on"
        );

        let silent: Vec<_> = SIGNATURE_KIND
            .iter()
            .filter(|magic| magic.signatures.is_empty())
            .collect();
        assert_eq!(
            silent.len(),
            1,
            "a new entry with no signature changes what the blob can describe"
        );
        assert!(matches!(silent[0].rules, MatchRules::WithFn(_)));
    }

    /// `MatchRules` has exactly two variants, and `build_blob` reads the second
    /// one to set `usesPredicate`. A third variant would make that flag lie
    /// rather than fail, so it is asserted here instead of being left to a
    /// pattern that silently stops matching.
    #[test]
    fn a_predicate_rule_is_the_only_non_default_variant() {
        for magic in SIGNATURE_KIND {
            assert!(matches!(
                magic.rules,
                MatchRules::Default | MatchRules::WithFn(_)
            ));
        }
    }

    /// The constant block is a hand-maintained index into a flat array. A typo
    /// here would hand JavaScript a plausible wrong number, which is the failure
    /// mode worth a test rather than a clippy lint.
    #[test]
    fn every_constant_index_is_inside_the_block() {
        for index in [
            constant::DEFAULT_MAX_BYTES_READ,
            constant::BYTES_READ,
            constant::DEFAULT_OFFSET,
            constant::ISO_MAX_BYTES_READ,
            constant::TAR_MAX_BYTES_READ,
        ] {
            assert!(index < constant::COUNT);
        }
    }

    #[test]
    fn an_empty_buffer_is_a_no_match_rather_than_undefined_behaviour() {
        // An empty `Uint8Array` asks for a zero-length buffer, and `from_raw_parts`
        // rejects a null pointer even then. Two things make that defined: `borrow`
        // returns `&[]` for a null pointer, and `input_ptr` never hands one out.
        // Without both this is UB that happens to return NO_MATCH on the current
        // codegen and might not tomorrow.
        let handle = build_isolated_png();
        assert_eq!(match_rule(handle, b""), NO_MATCH);
        assert!(!kind_matches(0, b""));
        assert_eq!(which_kind(b""), NO_MATCH);
        assert_eq!(which_kind_max(b"", usize::MAX), NO_MATCH);
        // And a rule set with a zero-length signature is a legitimate one: it
        // matches everything, which is a real answer, not a crash.
        let catch_all = rules_new();
        add_signature(catch_all, b"");
        rules_add_offset(catch_all, 0);
        assert!(rules_finish(catch_all));
        assert_eq!(match_rule(catch_all, b"anything at all"), 0);
    }

    #[test]
    fn the_scratch_buffers_are_never_null() {
        // `from_raw_parts` requires a non-null, aligned pointer whatever the
        // length, so a zero-length request still has to return an address. On a
        // wasm build this is the only thing between a caller passing an empty
        // buffer and undefined behaviour; in a host test it is the invariant,
        // stated.
        assert!(!input_ptr(0).is_null());
        assert!(!answer_ptr(0).is_null());
        // Repeated calls of the same size do not move the buffer, so the address a
        // caller holds across two same-sized calls is still valid.
        let first = input_ptr(64);
        assert_eq!(first, input_ptr(64));
    }

    #[test]
    fn the_input_buffer_grows_rather_than_truncating() {
        // A short call followed by a long one, which is what a directory scan does
        // constantly. If the buffer were sized to the *first* request the second
        // would write past its end, into the module's own data — silently, because
        // the module would keep returning answers.
        let small = input_ptr(8);
        // SAFETY: `input_ptr` just allocated at least 8 bytes.
        unsafe { std::ptr::write_bytes(small, 0xAB, 8) };
        let large = input_ptr(4096);
        assert!(!large.is_null());
        // SAFETY: `input_ptr(4096)` just allocated at least 4096 bytes, and the
        // first eight of them were written above.
        let first_eight = unsafe { std::slice::from_raw_parts(large, 8) };
        assert_eq!(
            first_eight, &[0xAB; 8],
            "growing must preserve, not discard"
        );
    }

    #[test]
    fn the_input_and_answer_buffers_do_not_overlap() {
        // `rules_match_all` reads the input while writing the answer, so a shared
        // buffer would let a rule set match its own result list. The loader relies
        // on these being separate; so does this test.
        let input = input_ptr(64);
        let answers = answer_ptr(64);
        let input_len = 64 * size_of::<u8>();
        let answers_len = 64 * size_of::<u32>();
        assert!(
            input as usize + input_len <= answers as usize
                || answers as usize + answers_len <= input as usize,
            "the scratch buffers overlap: input {:#x}..{:#x}, answers {:#x}..{:#x}",
            input as usize,
            input as usize + input_len,
            answers as usize,
            answers as usize + answers_len,
        );
    }

    #[test]
    fn a_wrong_discriminant_matches_nothing_and_does_not_trap() {
        assert!(!kind_matches(9_999, b"\x89PNG\r\n\x1a\n"));
    }

    #[test]
    fn a_built_rule_set_matches_its_own_bytes() {
        let handle = build_isolated_png();
        assert_eq!(match_rule(handle, b"\x89PNG\r\n\x1a\n"), 0);
        assert_eq!(match_rule(handle, b"nope"), NO_MATCH);
        assert_eq!(rules_len(handle), 1);
    }

    #[test]
    fn match_all_reports_every_rule_that_matched() {
        let handle = rules_new();
        for magic in [&b"\x89PNG\r\n\x1a\n"[..], b"GIF89a", b"%PDF-"] {
            add_signature(handle, magic);
            rules_add_offset(handle, 0);
            rules_next_rule(handle);
        }
        // The trailing empty rule the last `rules_next_rule` left behind is
        // dropped rather than rejected, which is what this assertion pins.
        assert!(rules_finish(handle));
        assert_eq!(rules_len(handle), 3);
        assert_eq!(match_all(handle, b"\x89PNG\r\n\x1a\n"), vec![0]);
        assert_eq!(match_all(handle, b"GIF89a"), vec![1]);
        assert_eq!(match_all(handle, b"neither"), Vec::<u32>::new());

        // Two rules, both given the same signature. Both match, and `all` says
        // so, which is the only thing that separates it from returning the first
        // answer and calling it a day.
        let handle2 = rules_new();
        for _ in 0..2 {
            add_signature(handle2, b"\x89PNG\r\n\x1a\n");
            rules_add_offset(handle2, 0);
            rules_next_rule(handle2);
        }
        assert!(rules_finish(handle2));
        assert_eq!(match_all(handle2, b"\x89PNG\r\n\x1a\n"), vec![0, 1]);
    }

    #[test]
    fn a_rule_may_carry_several_signatures_and_several_offsets() {
        // This is the `Default` arm's actual question: does any signature appear
        // at any offset. Multiple of each is the normal case, not a mistake, and
        // the crate's `match_types_custom` is what answers it, so getting it
        // wrong here would be a port bug rather than a Rust bug.
        let handle = rules_new();
        for magic in [&b"\x89PNG"[..], b"PNG"] {
            add_signature(handle, magic);
        }
        for offset in [0, 4] {
            rules_add_offset(handle, offset);
        }
        assert!(rules_finish(handle));
        assert_eq!(rules_len(handle), 1);
        assert_eq!(
            match_rule(handle, b"junk\x89PNG\x0d\x0a\x1a\x0a"),
            0,
            "a signature at offset 4 must be found"
        );
    }

    #[test]
    fn an_incomplete_rule_set_is_rejected_rather_than_matching_nothing_forever() {
        // No rules at all.
        assert!(!rules_finish(rules_new()));

        // A rule with a signature and no offset.
        let handle = rules_new();
        add_signature(handle, b"x");
        assert!(!rules_finish(handle));

        // A rule with an offset and no signature.
        let handle = rules_new();
        rules_add_offset(handle, 0);
        assert!(!rules_finish(handle));

        // Two rules where the first is complete and the second is not: rejected
        // as a whole, because a half-built rule set is a caller bug and matching
        // the first rule anyway would hide it.
        let handle = rules_new();
        add_signature(handle, b"x");
        rules_add_offset(handle, 0);
        rules_next_rule(handle);
        add_signature(handle, b"x");
        assert!(!rules_finish(handle));
    }

    #[test]
    fn a_closed_handle_reports_no_match_rather_than_trapping() {
        assert_eq!(match_rule(9_999, b"\x89PNG\r\n\x1a\n"), NO_MATCH);
        assert_eq!(rules_len(9_999), 0);
        assert_eq!(match_all(9_999, b"\x89PNG\r\n\x1a\n"), Vec::<u32>::new());
    }

    #[test]
    fn a_rule_set_can_be_released_and_the_handle_stops_working() {
        let handle = build_isolated_png();
        assert_eq!(match_rule(handle, b"\x89PNG\r\n\x1a\n"), 0);
        rules_release(handle);
        assert_eq!(match_rule(handle, b"\x89PNG\r\n\x1a\n"), NO_MATCH);
    }

    #[test]
    fn handles_are_never_zero_because_zero_means_no_handle() {
        for _ in 0..64 {
            assert_ne!(rules_new(), 0);
        }
    }

    #[test]
    fn rules_are_leaked_not_rebuilt() {
        // The leak is the documented trade. This test exists so that replacing it
        // with a per-call rebuild shows up as a failing assertion about the
        // comment, rather than as a silent allocation on a hot path that nobody
        // notices until a directory scan gets slow.
        let handle = build_isolated_png();
        for _ in 0..1000 {
            assert_eq!(match_rule(handle, b"\x89PNG\r\n\x1a\n"), 0);
        }
        assert_eq!(rules_len(handle), 1);
    }

    /// `narrow` is the identity on wasm32 and unreachable on 64-bit for every
    /// value the blob actually carries, so the saturating arm is dead code in
    /// practice. It is tested anyway: it is the arm that would run if a read size
    /// ever exceeded `u32`, and an untested fallback is a fallback that does not
    /// do what it says.
    #[test]
    fn narrow_saturates_rather_than_wrapping() {
        assert_eq!(narrow(0), 0);
        assert_eq!(
            narrow(usize::try_from(u32::MAX).expect("fits in usize")),
            u32::MAX
        );
        if usize::BITS > 32 {
            assert_eq!(narrow(usize::MAX), u32::MAX);
            let past_the_edge = usize::try_from(u64::from(u32::MAX) + 1).expect("fits in usize");
            assert_eq!(narrow(past_the_edge), u32::MAX);
        }
    }
}
