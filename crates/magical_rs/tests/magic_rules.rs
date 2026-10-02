//! What `magic_rules!` produces, checked against the struct it is sugar for.
//!
//! # Why the same rules are written twice
//!
//! Every test here builds the same rule set twice: once through the macro, once
//! as a `MagicCustom { .. }` literal, and asserts the two are equal. That is the
//! whole contract. The macro claims to be syntax, so the only thing worth testing
//! is whether the syntax and the struct agree — and a test that only checked
//! "the macro compiles" or "the macro matches some bytes" would pass while the
//! macro wrote a wrong `max_bytes_read` that nothing downstream reads.
//!
//! It would have passed on a real bug. The first version of the macro defaulted
//! `offsets` to `&[]`, which is a slice that matches no signature at all: every
//! rule compiled, every rule was unreachable, and the only thing that noticed was
//! a doctest asserting an answer. The equality tests below are what catch that
//! class, because `&[]` is not `&[0]`.
//!
//! # Why the struct's own fields are asserted
//!
//! `assert_eq!` on two `MagicCustom` values is not available — the struct derives
//! only `Clone, Copy` — so these tests compare field by field. That is more
//! verbose and it is the reason a test can name *which* field a change broke,
//! which is the difference between a test that tells you what to look at and one
//! that tells you something is wrong.

use magical_rs::magic_rules;
use magical_rs::magical::bytes_read::{DEFAULT_MAX_BYTES_READ, DEFAULT_OFFSET};
use magical_rs::magical::magic_custom::{CustomMatchRules, MagicCustom, match_types_custom};

/// The kind every rule in this file reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// A byte signature at offset 0.
    Shoujo,
    /// A second byte signature, so ordering can be tested.
    Magical,
    /// A rule at a non-zero offset.
    AtOffset,
    /// Decided by a function.
    Predicate,
    /// Never matches. Its only job is to be a later rule than the others.
    Never,
}

/// Which `CustomMatchRules` variant a rule carries, by name.
///
/// A `matches!` would do, and did: it failed on `AnyMatchesUnsafe(&[_])` for a
/// reason that was not the macro's — matching a slice pattern through an
/// enum's lifetime-carrying field needs the binding to outlive the match, and
/// the failure reads like a type error in the macro rather than a mistake in the
/// test. Naming the variant says what went wrong when it does.
const fn variant_of(rules: &CustomMatchRules<'_>) -> &'static str {
    match rules {
        CustomMatchRules::Default => "Default",
        CustomMatchRules::WithFn(_) => "WithFn",
        CustomMatchRules::AnyMatches(_) => "AnyMatches",
        CustomMatchRules::AllMatches(_) => "AllMatches",
        #[cfg(feature = "unsafe_context")]
        CustomMatchRules::WithFnUnsafe { .. } => "WithFnUnsafe",
        #[cfg(feature = "unsafe_context")]
        CustomMatchRules::AnyMatchesUnsafe(_) => "AnyMatchesUnsafe",
        #[cfg(feature = "unsafe_context")]
        CustomMatchRules::AllMatchesUnsafe(_) => "AllMatchesUnsafe",
        #[cfg(not(feature = "unsafe_context"))]
        _ => "unreachable",
    }
}

/// A byte predicate, so the `via` arms have something to point at.
fn is_shoujo(bytes: &[u8]) -> bool {
    bytes.starts_with(b"Shoujo")
}

/// A second byte predicate, for `via any` and `via all`.
fn is_magical(bytes: &[u8]) -> bool {
    bytes.starts_with(b"Magical")
}

/// The rules the macro builds.
///
/// Deliberately covers one arm of every shape the macro accepts, so a change to
/// an arm has to come here to be noticed.
static VIA_MACRO: &[MagicCustom<Kind>] = magic_rules![
    (Kind::Shoujo, b"Shoujo"),
    (Kind::Magical, b"Magical"),
    (Kind::AtOffset, b"OFFS", at 4, read 16),
    (Kind::Predicate, via is_shoujo),
];

/// The same four rules, written out.
///
/// The reference every other test measures against. If the two ever disagree,
/// this file is wrong or the macro is, and both are worth knowing.
static BY_HAND: &[MagicCustom<Kind>] = &[
    MagicCustom {
        signatures: &[b"Shoujo"],
        offsets: &[0],
        max_bytes_read: 2048,
        kind: Kind::Shoujo,
        rules: CustomMatchRules::Default,
    },
    MagicCustom {
        signatures: &[b"Magical"],
        offsets: &[0],
        max_bytes_read: 2048,
        kind: Kind::Magical,
        rules: CustomMatchRules::Default,
    },
    MagicCustom {
        signatures: &[b"OFFS"],
        offsets: &[4],
        max_bytes_read: 16,
        kind: Kind::AtOffset,
        rules: CustomMatchRules::Default,
    },
    MagicCustom {
        signatures: &[],
        offsets: &[],
        max_bytes_read: 2048,
        kind: Kind::Predicate,
        rules: CustomMatchRules::WithFn(is_shoujo),
    },
];

/// Two rules, one field at a time.
///
/// A helper rather than a macro of its own: the point is that a failure names the
/// field, and a `assert_eq!` over a tuple would name the tuple.
fn assert_rule_eq(
    label: &str,
    index: usize,
    got: &MagicCustom<'_, Kind>,
    want: &MagicCustom<'_, Kind>,
) {
    assert_eq!(got.kind, want.kind, "{label}[{index}]: kind");
    assert_eq!(
        got.signatures, want.signatures,
        "{label}[{index}]: signatures"
    );
    assert_eq!(got.offsets, want.offsets, "{label}[{index}]: offsets");
    assert_eq!(
        got.max_bytes_read, want.max_bytes_read,
        "{label}[{index}]: max_bytes_read"
    );
    assert_eq!(
        std::mem::discriminant(&got.rules),
        std::mem::discriminant(&want.rules),
        "{label}[{index}]: which CustomMatchRules variant",
    );
}

/// The macro and the struct agree, field by field.
#[test]
fn the_macro_builds_the_struct_it_claims_to() {
    assert_eq!(VIA_MACRO.len(), BY_HAND.len(), "rule count differs");
    for (index, (got, want)) in VIA_MACRO.iter().zip(BY_HAND).enumerate() {
        assert_rule_eq("macro", index, got, want);
    }
}

/// The rule count is what was written, not what the struct can hold.
///
/// A macro that silently dropped its last rule would still pass a "does it
/// compile" test and would answer every query the same way. This is the one
/// thing about a list that no per-field check can see.
#[test]
fn a_one_rule_list_is_still_a_list() {
    static ONE: &[MagicCustom<Kind>] = magic_rules![(Kind::Shoujo, b"Shoujo")];
    assert_eq!(ONE.len(), 1);
    assert_eq!(ONE[0].kind, Kind::Shoujo);
}

/// The defaulted `offsets` is `[0]`, which is what makes a rule reachable.
///
/// Named for the bug it exists to prevent. `offsets: &[]` compiles, runs, and
/// matches nothing, because matching is `signatures.any(|s| offsets.any(|o| ..))`
/// and an empty inner `any` is `false` for every signature.
#[test]
fn a_defaulted_offset_is_zero_and_not_empty() {
    static RULES: &[MagicCustom<Kind>] = magic_rules![(Kind::Shoujo, b"Shoujo")];

    assert_eq!(RULES[0].offsets, &[DEFAULT_OFFSET]);
    // The length is asserted separately from the contents above because "not
    // empty" and "equal to [0]" are different facts, and an empty slice matches no
    // signature at all.
    assert_eq!(
        RULES[0].offsets.len(),
        1,
        "an empty offsets slice matches no signature at all",
    );
    assert_eq!(
        match_types_custom(b"ShoujoFile", RULES, Kind::Never),
        Kind::Shoujo,
    );
}

/// A predicate rule carries no signatures, and needs none.
#[test]
fn a_predicate_rule_needs_no_signature() {
    static RULES: &[MagicCustom<Kind>] = magic_rules![(Kind::Predicate, via is_magical)];

    assert_eq!(RULES[0].signatures, &[] as &[&[u8]]);
    assert_eq!(RULES[0].offsets, &[] as &[usize]);
    assert_eq!(
        match_types_custom(b"MagicalGirl", RULES, Kind::Never),
        Kind::Predicate,
    );
    assert_eq!(match_types_custom(b"Nope", RULES, Kind::Never), Kind::Never);
}

/// `read` defaults to the crate's own default, not to something smaller.
///
/// Asserted against `DEFAULT_MAX_BYTES_READ` rather than against 2,048 as a
/// literal, so that if the crate's default ever moves, this says so instead of
/// quietly pinning a second, different number.
#[test]
fn a_defaulted_read_is_the_crates_default() {
    static RULES: &[MagicCustom<Kind>] = magic_rules![
        (Kind::Shoujo, b"Shoujo"),
        (Kind::Predicate, via is_shoujo),
    ];

    assert_eq!(RULES[0].max_bytes_read, DEFAULT_MAX_BYTES_READ);
    assert_eq!(RULES[1].max_bytes_read, DEFAULT_MAX_BYTES_READ);
}

/// Several signatures, and several offsets, and the two nest.
#[test]
fn several_signatures_and_several_offsets() {
    static BOTH: &[MagicCustom<Kind>] = magic_rules![
        (Kind::Shoujo, [b"Shoujo", b"Shojo"], at [0, 8], read 32),
    ];
    static SIGS_ONLY: &[MagicCustom<Kind>] = magic_rules![(Kind::Shoujo, [b"Shoujo", b"Shojo"])];

    // A `&[&[u8]]` cannot be compared against an array literal of byte-string
    // literals directly: `b"Shoujo"` is `&[u8; 6]` and `b"Shojo"` is `&[u8; 5]`, so
    // the array has no single element type to infer against the slice. Naming the
    // type is what makes the comparison mean anything.
    let want: &[&[u8]] = &[b"Shoujo", b"Shojo"];

    assert_eq!(BOTH[0].signatures, want);
    assert_eq!(BOTH[0].offsets, &[0, 8]);
    assert_eq!(BOTH[0].max_bytes_read, 32);

    assert_eq!(SIGS_ONLY[0].signatures, want);
    assert_eq!(SIGS_ONLY[0].offsets, &[0]);

    // The second signature matches, so the rule is reachable through it.
    assert_eq!(
        match_types_custom(b"Shojo", SIGS_ONLY, Kind::Never),
        Kind::Shoujo
    );
}

/// The second offset is a real offset, and a signature only there still matches.
#[test]
fn an_offset_alone_is_enough_to_reach_the_rule() {
    static RULES: &[MagicCustom<Kind>] = magic_rules![(Kind::AtOffset, b"OFFS", at 4)];

    let mut buffer = [b'.'; 16];
    buffer[4..8].copy_from_slice(b"OFFS");

    assert_eq!(RULES[0].offsets, &[4]);
    assert_eq!(
        match_types_custom(&buffer, RULES, Kind::Never),
        Kind::AtOffset,
    );
    // And nothing matches when the signature is not at that offset.
    assert_eq!(
        match_types_custom(b"OFFS........", RULES, Kind::Never),
        Kind::Never
    );
}

/// `via any` and `via all` reach the variants they name, with the right operands.
#[test]
fn via_any_and_via_all_reach_their_variants() {
    static ANY: &[MagicCustom<Kind>] =
        magic_rules![(Kind::Predicate, via any [is_shoujo, is_magical])];
    static ALL: &[MagicCustom<Kind>] =
        magic_rules![(Kind::Predicate, via all [is_shoujo, is_magical])];

    assert_eq!(variant_of(&ANY[0].rules), "AnyMatches");
    assert_eq!(variant_of(&ALL[0].rules), "AllMatches");

    // `any` accepts one of the two; `all` requires both, so a buffer that
    // satisfies one of them separates the two variants on behaviour alone.
    assert_eq!(
        match_types_custom(b"Shoujo", ANY, Kind::Never),
        Kind::Predicate
    );
    assert_eq!(
        match_types_custom(b"Magical", ANY, Kind::Never),
        Kind::Predicate
    );
    assert_eq!(match_types_custom(b"Shoujo", ALL, Kind::Never), Kind::Never);
    assert_eq!(
        match_types_custom(b"ShoujoMagical", ALL, Kind::Never),
        Kind::Never
    );
}

/// A trailing comma is optional, and one is not a rule of its own.
#[test]
fn a_trailing_comma_changes_nothing() {
    static WITH: &[MagicCustom<Kind>] = magic_rules![
        (Kind::Shoujo, b"Shoujo",),
        (Kind::Magical, b"Magical", read 64,),
    ];
    static WITHOUT: &[MagicCustom<Kind>] =
        magic_rules![(Kind::Shoujo, b"Shoujo"), (Kind::Magical, b"Magical", read 64)];

    assert_eq!(WITH.len(), WITHOUT.len());
    for (index, (got, want)) in WITH.iter().zip(WITHOUT).enumerate() {
        assert_rule_eq("with-comma", index, got, want);
    }
}

/// A multi-line list survives, which is the shape the doc shows.
///
/// A macro that only worked on one line would be a macro nobody uses, and the
/// failure would be a parse error at the first newline.
#[test]
fn a_rule_survives_being_written_across_lines() {
    static RULES: &[MagicCustom<Kind>] = magic_rules![
        (
            Kind::AtOffset,
            b"OFFS",
            at [
                4,
                8,
            ],
            read 64,
        ),
    ];

    assert_eq!(RULES.len(), 1);
    assert_eq!(RULES[0].offsets, &[4, 8]);
    assert_eq!(RULES[0].max_bytes_read, 64);
}

/// Order is the caller's, and the macro does not sort it.
///
/// `match_types_custom` returns the first match, so if this macro reordered — or
/// deduplicated, or reversed — the answer would change and nothing else would
/// notice. Both rules below match the same bytes.
#[test]
fn order_is_preserved_because_it_is_the_answer() {
    static SHUJO_FIRST: &[MagicCustom<Kind>] =
        magic_rules![(Kind::Shoujo, b"XX"), (Kind::Magical, b"XX"),];
    static MAGICAL_FIRST: &[MagicCustom<Kind>] =
        magic_rules![(Kind::Magical, b"XX"), (Kind::Shoujo, b"XX"),];

    assert_eq!(
        match_types_custom(b"XX", SHUJO_FIRST, Kind::Never),
        Kind::Shoujo
    );
    assert_eq!(
        match_types_custom(b"XX", MAGICAL_FIRST, Kind::Never),
        Kind::Magical,
    );
}

/// Two rules with the same kind and the same signature are two rules.
///
/// Not a hypothetical: deduplicating a rule list is an optimisation somebody
/// eventually adds, and it is only safe if the caller's order is the answer. This
/// is the test that says the count is preserved whatever the macro decides to do
/// with the values.
#[test]
fn duplicate_rules_are_kept() {
    static RULES: &[MagicCustom<Kind>] =
        magic_rules![(Kind::Shoujo, b"XX"), (Kind::Shoujo, b"XX"),];

    assert_eq!(RULES.len(), 2);
}

// ---------------------------------------------------------------------------
// The `unsafe via` arms.
//
// Gated on the same feature as the variants they expand to, so a build without
// `unsafe_context` does not fail to compile this file — the arms simply do not
// exist, exactly as the enum variants do not.
// ---------------------------------------------------------------------------

/// How many bytes the unsafe predicates below are allowed to read.
///
/// On the caller's side, because the callee is handed a `*const ()` and no length
/// and so has nothing to check a promise against.
#[cfg(feature = "unsafe_context")]
const READ_LEN: usize = 100;

/// A buffer of exactly [`READ_LEN`] bytes starting with the marker.
#[cfg(feature = "unsafe_context")]
fn buffer() -> [u8; READ_LEN] {
    let mut data = [0u8; READ_LEN];
    data[..b"MagicalGirl".len()].copy_from_slice(b"MagicalGirl");
    data
}

/// Matches the marker, over [`READ_LEN`] bytes.
#[cfg(feature = "unsafe_context")]
unsafe fn is_magical_girl(data: *const ()) -> bool {
    // SAFETY: every caller below passes `buffer()`, which is `READ_LEN` bytes, and
    // `match_types_custom` hands the predicate a pointer into the buffer it was
    // given. The caller being untrustworthy in general is what makes this
    // function `unsafe` at all; here the callers are the tests below.
    unsafe { core::slice::from_raw_parts(data.cast::<u8>(), READ_LEN).starts_with(b"MagicalGirl") }
}

/// The negation of [`is_magical_girl`], so `via all [a, b]` has two operands that
/// cannot both hold.
#[cfg(feature = "unsafe_context")]
unsafe fn is_not_magical_girl(data: *const ()) -> bool {
    // SAFETY: as `is_magical_girl`.
    unsafe { !core::slice::from_raw_parts(data.cast::<u8>(), READ_LEN).starts_with(b"MagicalGirl") }
}

/// `unsafe via` reaches the list variant, with a single operand.
///
/// One predicate is `AllMatchesUnsafe` of one rather than the struct variant
/// `WithFnUnsafe { func }`. The macro has one spelling for it, so this asserts
/// which variant that spelling actually produces — a macro that quietly built
/// `AnyMatchesUnsafe` would still answer correctly for one predicate and only
/// diverge from the documented shape.
#[test]
#[cfg(feature = "unsafe_context")]
fn unsafe_via_one_predicate_is_all_matches_of_one() {
    static RULES: &[MagicCustom<Kind>] =
        magic_rules![(Kind::Predicate, unsafe via is_magical_girl, read READ_LEN)];

    assert_eq!(
        variant_of(&RULES[0].rules),
        "AllMatchesUnsafe",
        "one unsafe predicate is `all` of one"
    );
    assert_eq!(RULES[0].max_bytes_read, READ_LEN);
    assert_eq!(
        match_types_custom(&buffer(), RULES, Kind::Never),
        Kind::Predicate,
    );
}

/// `unsafe via any` and `unsafe via all` differ on a buffer one of them accepts.
#[test]
#[cfg(feature = "unsafe_context")]
fn unsafe_via_any_and_all_differ() {
    static ANY: &[MagicCustom<Kind>] =
        magic_rules![(Kind::Predicate, unsafe via any [is_magical_girl, is_not_magical_girl])];
    static ALL: &[MagicCustom<Kind>] =
        magic_rules![(Kind::Predicate, unsafe via all [is_magical_girl, is_not_magical_girl])];

    assert_eq!(variant_of(&ANY[0].rules), "AnyMatchesUnsafe");
    assert_eq!(variant_of(&ALL[0].rules), "AllMatchesUnsafe");

    // One of the two is always true, so `any` always matches.
    assert_eq!(
        match_types_custom(&buffer(), ANY, Kind::Never),
        Kind::Predicate
    );
    // `all` needs both, and they contradict each other, so it never does.
    assert_eq!(match_types_custom(&buffer(), ALL, Kind::Never), Kind::Never);
}

/// An unsafe predicate rule still carries no signature, like the safe one.
#[test]
#[cfg(feature = "unsafe_context")]
fn an_unsafe_predicate_rule_needs_no_signature() {
    static RULES: &[MagicCustom<Kind>] =
        magic_rules![(Kind::Predicate, unsafe via is_magical_girl)];

    assert_eq!(RULES[0].signatures, &[] as &[&[u8]]);
    assert_eq!(RULES[0].offsets, &[] as &[usize]);
}
