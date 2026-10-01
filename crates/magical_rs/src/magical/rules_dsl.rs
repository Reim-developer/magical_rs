//! `magic_rules!` — level 2 rule lists as a table rather than as struct literals.
//!
//! This lives in its own file rather than at the bottom of `magic_custom.rs` because
//! it is sugar *for* that module, and a macro that documents the module it wraps
//! reads better next to it than 200 lines below it.
//!
//! What it replaces, in full:
//!
//! ```ignore
//! static RULES: &[MagicCustom<ShoujoKind>] = &[
//!     MagicCustom {
//!         signatures: &[b"MagicalGirl"],
//!         offsets: &[0],
//!         max_bytes_read: 69,
//!         kind: ShoujoKind::MahouShoujo,
//!         rules: CustomMatchRules::Default,
//!     },
//! ];
//! ```
//!
//! What it becomes:
//!
//! ```
//! use magical_rs::magic_rules;
//! use magical_rs::magical::magic_custom::MagicCustom;
//!
//! #[derive(Clone, Copy, Debug, PartialEq)]
//! enum ShoujoKind {
//!     MahouShoujo,
//!     Unknown,
//! }
//!
//! static RULES: &[MagicCustom<ShoujoKind>] = magic_rules![
//!     (ShoujoKind::MahouShoujo, b"MagicalGirl", read 69),
//! ];
//! ```
//!
//! # What it does not do
//!
//! **It is not a new matching engine.** It expands to exactly the
//! `MagicCustom { .. }` literals above and nothing else. Every rule the macro
//! writes, a caller could have written by hand, and the crate's existing tests
//! cover the struct it produces rather than this sugar over it. A rule the macro
//! cannot express is a rule you write the long way, and the long way keeps
//! working — `magic_custom!` and the bare struct are both still public.
//!
//! **It is not gated behind a feature flag.** It adds no dependency, allocates
//! nothing, and emits no code unless you invoke it, so there is nothing to gate.
//! The fluent API in `fluent.rs` is the one that earns a flag, because a trait
//! method on `[u8]` is something a caller has to opt out of, not out of nothing.
//!
//! # The syntax
//!
//! One rule is a parenthesised tuple. Its first element is the kind, and what
//! follows is either bytes or a predicate, never both:
//!
//! ```text
//! (Kind, BYTES, offset N, read N)
//! (Kind, [BYTES, ..], at [N, ..], read N)
//! (Kind, via PREDICATE, read N)
//! (Kind, via any [PREDICATE, ..], read N)
//! (Kind, via all [PREDICATE, ..], read N)
//! (Kind, unsafe via PREDICATE, read N)
//! ```
//!
//! `offset` and `read` are optional and default to `DEFAULT_OFFSET` (0) and
//! `DEFAULT_MAX_BYTES_READ` (2,048). The `unsafe` forms are the crate's
//! `unsafe_context` predicates and need that feature, exactly as the enum
//! variants they expand to do.
//!
//! ## Why the defaults are the crate's, and not smaller
//!
//! `read` defaults to 2,048 because that is `DEFAULT_MAX_BYTES_READ`, the same
//! number the built-in table uses. A smaller default would be a trap: `read` is
//! what a caller reads that many bytes before matching, and a rule that needs
//! 32,769 bytes because its magic sits at that offset is *not* a rule that can
//! be satisfied by 2,048. The macro cannot infer which rules need more, so it
//! does not guess smaller than the crate already does — and the [Examples]
//! section shows what an offset past 2,048 costs you: you have to write it.

/// A list of [level 2 rules](crate::magical::magic_custom::MagicCustom), as a table
/// rather than as struct literals.
///
/// A list, so it expands to a `&'static [MagicCustom<K>]` — the shape
/// [`match_types_custom`](crate::magical::magic_custom::match_types_custom)
/// takes, and the shape a `static` holds:
///
/// ```
/// use magical_rs::magic_rules;
/// use magical_rs::magical::magic_custom::{MagicCustom, match_types_custom};
///
/// #[derive(Clone, Copy, Debug, PartialEq)]
/// enum Kind {
///     MahouShoujo,
///     Mikoto,
///     Unknown,
/// }
///
/// fn is_mikoto(bytes: &[u8]) -> bool {
///     bytes.starts_with(b"MikotoChan")
/// }
///
/// static RULES: &[MagicCustom<Kind>] = magic_rules![
///     // A byte signature at offset 0. `read` and `offset` are both defaulted.
///     (Kind::MahouShoujo, b"MagicalGirl"),
///
///     // A predicate instead of bytes. The rule needs no signature to match on,
///     // so the macro leaves `signatures` and `offsets` empty for you — which is
///     // the step the struct literal makes easy to get wrong, since leaving a
///     // signature in place does not make the predicate conditional, it just
///     // leaves bytes nobody reads.
///     (Kind::Mikoto, via is_mikoto),
/// ];
///
/// let found = match_types_custom(b"MagicalGirl", RULES, Kind::Unknown);
/// assert_eq!(found, Kind::MahouShoujo);
///
/// let found = match_types_custom(b"MikotoChan", RULES, Kind::Unknown);
/// assert_eq!(found, Kind::Mikoto);
/// ```
///
/// # Order is the answer
///
/// [`match_types_custom`](crate::magical::magic_custom::match_types_custom)
/// returns the first rule that matches, so the order below is the precedence.
/// A predicate that matches everything placed first means nothing after it can
/// ever run:
///
/// ```
/// use magical_rs::magic_rules;
/// use magical_rs::magical::magic_custom::{MagicCustom, match_types_custom};
///
/// #[derive(Clone, Copy, Debug, PartialEq)]
/// enum Kind {
///     Text,
///     Cbor,
///     Unknown,
/// }
///
/// fn starts_with_magic(bytes: &[u8]) -> bool {
///     bytes.starts_with(b"\x93")
/// }
///
/// static RULES: &[MagicCustom<Kind>] = magic_rules![
///     (Kind::Cbor, via starts_with_magic),
///     (Kind::Text, b"\x1b["),          // an escape sequence, easy to get wrong alone
/// ];
///
/// // CBOR first, so a `\x93`-prefixed file is CBOR rather than text.
/// assert_eq!(match_types_custom(b"\x93abc", RULES, Kind::Unknown), Kind::Cbor);
/// assert_eq!(match_types_custom(b"\x1b[0m", RULES, Kind::Unknown), Kind::Text);
/// ```
///
/// # Several signatures, several offsets
///
/// More than one signature means the rule matches on any of them; more than one
/// offset means any signature at any of them. Both nest, and a rule with a
/// non-zero offset cannot be satisfied by a buffer shorter than
/// `offset + longest signature` — the macro cannot check that, because the
/// signature lengths are whatever the caller wrote:
///
/// ```
/// use magical_rs::magic_rules;
/// use magical_rs::magical::magic_custom::MagicCustom;
///
/// #[derive(Clone, Copy, Debug, PartialEq)]
/// enum Kind {
///     Iso,
///     Unknown,
/// }
///
/// // ISO 9660 stores its magic at three of the 16 KiB boundaries the standard
/// // allows, and 36,870 is what reading one costs. The default `read` of 2,048
/// // would not reach any of them, which is why the rule says so out loud.
/// static RULES: &[MagicCustom<Kind>] = magic_rules![
///     (Kind::Iso, b"CD001", at [32769, 34817, 36865], read 36865),
/// ];
/// ```
///
/// # Empty
///
/// A list with no rules is a `&'static []` of an unnameable element type, which
/// is the one thing the macro refuses rather than paper over:
///
/// ```compile_fail
/// use magical_rs::magic_rules;
/// use magical_rs::magical::magic_custom::MagicCustom;
///
/// #[derive(Clone, Copy)]
/// enum Kind { A }
/// static RULES: &[MagicCustom<Kind>] = magic_rules![];
/// ```
///
/// Use `&[]` for that. The macro's error names the fix, because "type
/// annotations needed" is not a fix.
#[macro_export]
macro_rules! magic_rules {
    // Entry point. Nothing here builds a value; it only decides how many rules
    // there are, so that a one-rule list is still a list rather than a bare
    // element the caller has to wrap.
    () => {
        compile_error!(
            "`magic_rules![]` has no element type, so it cannot name one. Write `&[]`, \
             or add a rule."
        )
    };
    // The brackets are re-added here rather than being seen by this macro at all:
    // `magic_rules![ .. ]` invokes with `[]` as the delimiter, so the macro is
    // handed the *contents* and the two are indistinguishable from an empty
    // invocation. Putting them back is what lets the inner macro tell an empty
    // list from a list of rules, which is the whole reason it is a separate macro.
    ($($rule:tt)*) => {
        $crate::__magic_rules_inner!([$($rule)*])
    };
}

/// The list body, once the empty case above has had its say.
///
/// This is a separate macro rather than a second arm of `magic_rules!` because
/// `magic_rules!` and `__magic_rules_inner!` would otherwise both be able to
/// match a bare token list, and the compiler picks the first one it finds rather
/// than the most specific — so the empty-list error would never fire.
///
/// Each rule is matched as `tt`, not `expr`, and that is load-bearing rather than
/// incidental. A rule is `(Kind, b"x", read 69)`, and the `read 69` in it is not an
/// expression, so a list arm matching `$rule:expr` would fail to *parse* a rule
/// that is perfectly valid — with an error pointing at the `read` and saying
/// nothing about rules at all. Matching the parenthesised group as one token tree
/// and handing its contents to [`__magic_rule_inner`] is what keeps the outer
/// macro ignorant of the inner syntax.
#[doc(hidden)]
#[macro_export]
macro_rules! __magic_rules_inner {
    // Each `$rule` here is one whole parenthesised group, captured as a single
    // `tt`. It has to be a single `tt` rather than `$( $rule:tt )*` *inside* the
    // group, because a repetition that matches across a group's contents loses
    // the separators: `(Kind, b"x")` captured that way replays as `Kind b"x"`,
    // which is not a rule and fails with an error pointing at nothing useful.
    ([$( $rule:tt ),* $(,)?]) => {
        &[$($crate::__magic_rule_group!($rule)),*]
    };
    ([$($other:tt)*]) => {
        compile_error!(
            "each rule is parenthesised: `(Kind, BYTES, ..)` or `(Kind, via PREDICATE, ..)`. \
             See `magic_rules!`."
        )
    };
    ($($other:tt)*) => {
        compile_error!("`magic_rules!` takes a bracketed list, `[ .. ]`. See `magic_rules!`.")
    };
}

/// Strips one rule's parentheses and hands the contents to the clause matcher.
///
/// Its own macro because the two jobs need two different shapes: this one takes a
/// single group, the next takes the tokens inside it. A single macro cannot do
/// both, because the arm that strips a group would also match an unparenthesised
/// rule and then recurse on itself forever.
#[doc(hidden)]
#[macro_export]
macro_rules! __magic_rule_group {
    (( $($rule:tt)* )) => {
        $crate::__magic_rule_inner!($($rule)*)
    };
    ($other:tt) => {
        compile_error!(
            "a rule is parenthesised: `(Kind, BYTES, ..)` or `(Kind, via PREDICATE, ..)`. \
             See `magic_rules!`."
        )
    };
}

/// One rule, from the tokens inside its parentheses.
///
/// The order of these arms is the whole design. A rule is a *sequence* of
/// clauses, and this reads that sequence left to right, which is why the arms
/// are ordered `Kind`, then the byte clauses, then the predicate clauses, then
/// the modifiers. `via` and `unsafe via` are distinguished before `via`'s
/// payload is looked at, so `unsafe` can never be swallowed as a function name.
#[doc(hidden)]
#[macro_export]
macro_rules! __magic_rule_inner {
    // ---- the two builders, first -----------------------------------------
    // These come before everything else, including the catch-all below, and the
    // reason is the catch-all. `macro_rules!` tries arms in order and does not
    // backtrack once an arm matches, so `($($other:tt)*)` placed above these
    // would swallow every internal call and turn a working expansion into
    // `compile_error!`. Internal calls are the reason this macro recurses at all.
    //
    // Separated from the clause-matching arms so that every byte rule and every
    // predicate rule assembles its struct in exactly one place. A change to the
    // struct's shape is then a two-line edit rather than a twenty-arm sweep, and
    // the arms below stay about syntax.
    //
    // `@bytes` and `@predicate` differ in what they leave in `offsets`, and the
    // difference is not cosmetic. A byte rule needs at least one offset because
    // matching iterates `offsets` and an empty slice answers `false` for every
    // signature — a rule written as `offsets: &[]` compiles, runs, and never
    // matches, which is the worst of the three outcomes. A predicate rule needs
    // none, because it never reads them. That asymmetry is why the two builders
    // exist at all rather than one builder with a flag.
    (@bytes $kind:expr, [$($sig:expr),*], [$($offset:expr),*], $read:expr) => {
        $crate::magical::magic_custom::MagicCustom {
            signatures: &[$($sig),*],
            offsets: &[$($offset),*],
            max_bytes_read: $read,
            kind: $kind,
            rules: $crate::magical::magic_custom::CustomMatchRules::Default,
        }
    };
    (@predicate $kind:expr, $rules:expr, $read:expr) => {
        $crate::magical::magic_custom::MagicCustom {
            signatures: &[],
            offsets: &[],
            max_bytes_read: $read,
            kind: $kind,
            rules: $rules,
        }
    };

    // ---- kind, then a predicate -------------------------------------------
    // **The order of the `any` and `all` arms is load-bearing, and putting them
    // after the bare `via` arm breaks `via any [..]` entirely.** A `$predicate:expr`
    // against the tokens `any [is_a, is_b]` does not decline and let the next arm
    // try: `expr` is a *parsed fragment*, so the parser commits, reads `any [` as
    // an index expression, and stops at the first `,` inside the brackets with a
    // hard parse error. `macro_rules!` backtracks between arms but not inside a
    // fragment, so there is no arm order that recovers from this — `any` and `all`
    // have to be matched as literal tokens before any `expr` sees them.
    //
    // The same applies to the unsafe block below, and to `unsafe` being matched
    // before `via` is looked at, so `unsafe` cannot be read as a function name.
    //
    // Every predicate arm leaves `signatures` and `offsets` empty. A rule that
    // matches by calling a function must not also carry bytes: the struct allows
    // both, and the bytes would then be dead, which reads as "and also" and means
    // nothing.

    // `via any` and `via all`, longest form first for the same reason as the byte
    // arms above: a shorter arm placed first would reject the longer input.
    ($kind:expr, via any [$($predicate:expr),* $(,)?], read $read:expr $(,)?) => {
        $crate::__magic_rule_inner!(@predicate $kind, $crate::magical::magic_custom::CustomMatchRules::AnyMatches(&[$($predicate),*]), $read)
    };
    ($kind:expr, via any [$($predicate:expr),* $(,)?] $(,)?) => {
        $crate::__magic_rule_inner!(@predicate $kind, $crate::magical::magic_custom::CustomMatchRules::AnyMatches(&[$($predicate),*]), $crate::magical::bytes_read::DEFAULT_MAX_BYTES_READ)
    };
    ($kind:expr, via all [$($predicate:expr),* $(,)?], read $read:expr $(,)?) => {
        $crate::__magic_rule_inner!(@predicate $kind, $crate::magical::magic_custom::CustomMatchRules::AllMatches(&[$($predicate),*]), $read)
    };
    ($kind:expr, via all [$($predicate:expr),* $(,)?] $(,)?) => {
        $crate::__magic_rule_inner!(@predicate $kind, $crate::magical::magic_custom::CustomMatchRules::AllMatches(&[$($predicate),*]), $crate::magical::bytes_read::DEFAULT_MAX_BYTES_READ)
    };

    // The bare `via`, which is the only arm an `expr` reaches.
    ($kind:expr, via $predicate:expr, read $read:expr $(,)?) => {
        $crate::__magic_rule_inner!(@predicate $kind, $crate::magical::magic_custom::CustomMatchRules::WithFn($predicate), $read)
    };
    ($kind:expr, via $predicate:expr $(,)?) => {
        $crate::__magic_rule_inner!(@predicate $kind, $crate::magical::magic_custom::CustomMatchRules::WithFn($predicate), $crate::magical::bytes_read::DEFAULT_MAX_BYTES_READ)
    };

    // ---- the unsafe predicates -------------------------------------------
    // These expand to variants that exist only under `unsafe_context`, so using
    // them without that feature is a compile error naming the variant — which is
    // the right error, and a better one than this macro could invent.
    ($kind:expr, unsafe via any [$($predicate:expr),* $(,)?], read $read:expr $(,)?) => {
        $crate::__magic_rule_inner!(@predicate $kind, $crate::magical::magic_custom::CustomMatchRules::AnyMatchesUnsafe(&[$($predicate),*]), $read)
    };
    ($kind:expr, unsafe via any [$($predicate:expr),* $(,)?] $(,)?) => {
        $crate::__magic_rule_inner!(@predicate $kind, $crate::magical::magic_custom::CustomMatchRules::AnyMatchesUnsafe(&[$($predicate),*]), $crate::magical::bytes_read::DEFAULT_MAX_BYTES_READ)
    };
    ($kind:expr, unsafe via all [$($predicate:expr),* $(,)?], read $read:expr $(,)?) => {
        $crate::__magic_rule_inner!(@predicate $kind, $crate::magical::magic_custom::CustomMatchRules::AllMatchesUnsafe(&[$($predicate),*]), $read)
    };
    ($kind:expr, unsafe via all [$($predicate:expr),* $(,)?] $(,)?) => {
        $crate::__magic_rule_inner!(@predicate $kind, $crate::magical::magic_custom::CustomMatchRules::AllMatchesUnsafe(&[$($predicate),*]), $crate::magical::bytes_read::DEFAULT_MAX_BYTES_READ)
    };
    // A single unsafe predicate is `AllMatchesUnsafe` of one. The crate's
    // `WithFnUnsafe { func }` is a struct variant holding a bare function, and
    // there is no way to name it through a list-shaped arm without a second
    // spelling of the same thing; one function is one function either way, and
    // `all` of one says so.
    ($kind:expr, unsafe via $predicate:expr, read $read:expr $(,)?) => {
        $crate::__magic_rule_inner!(@predicate $kind, $crate::magical::magic_custom::CustomMatchRules::AllMatchesUnsafe(&[$predicate]), $read)
    };
    ($kind:expr, unsafe via $predicate:expr $(,)?) => {
        $crate::__magic_rule_inner!(@predicate $kind, $crate::magical::magic_custom::CustomMatchRules::AllMatchesUnsafe(&[$predicate]), $crate::magical::bytes_read::DEFAULT_MAX_BYTES_READ)
    };

    // ---- kind, then bytes ------------------------------------------------
    // **The list arm comes first, and moving it below the single-signature arm
    // breaks every rule with more than one signature.** `expr` matches an array
    // literal — `[b"XXX", b"YY"]` is a perfectly good expression — so an arm
    // written `($kind:expr, $sig:expr)` accepts a *list* of signatures and binds
    // the whole list to one `$sig`. The expansion then has a `[&[u8]]` where a
    // `&[u8]` belongs, and the error is a type mismatch on the two byte strings'
    // lengths rather than anything about rules.
    //
    // This was mistaken for a coercion problem at first, and the fix that was
    // tried — `as &[u8]` on each element — appeared to work until it was removed
    // and the tests stayed green. It was the ordering all along. The comment is
    // here because the fix looks plausible: `expr` fragments are opaque AST
    // nodes, so a coercion site not reaching them is a real thing to suspect, and
    // this time it was not the cause.
    ($kind:expr, [$($sig:expr),* $(,)?] $(,)?) => {
        $crate::__magic_rule_inner!(@bytes $kind, [$($sig),*], [$crate::magical::bytes_read::DEFAULT_OFFSET], $crate::magical::bytes_read::DEFAULT_MAX_BYTES_READ)
    };
    ($kind:expr, $sig:expr $(,)?) => {
        $crate::__magic_rule_inner!(@bytes $kind, [$sig], [$crate::magical::bytes_read::DEFAULT_OFFSET], $crate::magical::bytes_read::DEFAULT_MAX_BYTES_READ)
    };

    // ---- kind, then bytes, then `at` and/or `read` ------------------------
    // Two ordering rules hold across this whole block, and both were found by
    // writing a rule the macro refused to accept rather than by reading it.
    //
    // 1. **List arms before single arms**, for the reason given above: `$sig:expr`
    //    matches an array literal, so a single arm placed first would capture a
    //    list as one signature. That applies to every pair below, not just the
    //    bare one.
    // 2. **Longer forms before shorter ones within each group.** `macro_rules!`
    //    commits to the first arm matching the *whole* sequence and does not
    //    backtrack, so `at 4` placed above `at 4, read 69` would accept the short
    //    form's tokens, fail on the trailing `read`, and never reach the arm that
    //    wanted them.
    //
    // The order below is: list-with-`read`, list-with-`at`-list, list-with-`at`,
    // list-with-`read`-only; then the same four for a single signature.
    ($kind:expr, [$($sig:expr),* $(,)?], at [$($offset:expr),* $(,)?], read $read:expr $(,)?) => {
        $crate::__magic_rule_inner!(@bytes $kind, [$($sig),*], [$($offset),*], $read)
    };
    ($kind:expr, [$($sig:expr),* $(,)?], at [$($offset:expr),* $(,)?] $(,)?) => {
        $crate::__magic_rule_inner!(@bytes $kind, [$($sig),*], [$($offset),*], $crate::magical::bytes_read::DEFAULT_MAX_BYTES_READ)
    };
    ($kind:expr, [$($sig:expr),* $(,)?], at $offset:expr, read $read:expr $(,)?) => {
        $crate::__magic_rule_inner!(@bytes $kind, [$($sig),*], [$offset], $read)
    };
    ($kind:expr, [$($sig:expr),* $(,)?], at $offset:expr $(,)?) => {
        $crate::__magic_rule_inner!(@bytes $kind, [$($sig),*], [$offset], $crate::magical::bytes_read::DEFAULT_MAX_BYTES_READ)
    };
    ($kind:expr, [$($sig:expr),* $(,)?], read $read:expr $(,)?) => {
        $crate::__magic_rule_inner!(@bytes $kind, [$($sig),*], [$crate::magical::bytes_read::DEFAULT_OFFSET], $read)
    };
    ($kind:expr, $sig:expr, at [$($offset:expr),* $(,)?], read $read:expr $(,)?) => {
        $crate::__magic_rule_inner!(@bytes $kind, [$sig], [$($offset),*], $read)
    };
    ($kind:expr, $sig:expr, at [$($offset:expr),* $(,)?] $(,)?) => {
        $crate::__magic_rule_inner!(@bytes $kind, [$sig], [$($offset),*], $crate::magical::bytes_read::DEFAULT_MAX_BYTES_READ)
    };
    ($kind:expr, $sig:expr, at $offset:expr, read $read:expr $(,)?) => {
        $crate::__magic_rule_inner!(@bytes $kind, [$sig], [$offset], $read)
    };
    ($kind:expr, $sig:expr, at $offset:expr $(,)?) => {
        $crate::__magic_rule_inner!(@bytes $kind, [$sig], [$offset], $crate::magical::bytes_read::DEFAULT_MAX_BYTES_READ)
    };
    ($kind:expr, $sig:expr, read $read:expr $(,)?) => {
        $crate::__magic_rule_inner!(@bytes $kind, [$sig], [$crate::magical::bytes_read::DEFAULT_OFFSET], $read)
    };

    // ---- anything else ----------------------------------------------------
    // A rule whose last clause is `at` or `read` in an order this does not accept,
    // or a rule that mixes bytes and a predicate. Both are a mistake a first
    // reading of the syntax would not make, and both are caught here rather than
    // silently taking a default.
    ($kind:expr, via $predicate:expr, at $($rest:tt)*) => {
        compile_error!(
            "a rule matches by bytes or by predicate, not both, and `at` only applies to \
             bytes. Write `(Kind, [$sig, ..], at [$offset, ..])` for a rule at an offset."
        )
    };
    ($kind:expr, $sig:expr, via $($rest:tt)*) => {
        compile_error!(
            "a rule matches by bytes or by predicate, not both. Drop the `via` clause, or \
             drop the signature before it."
        )
    };
    ($kind:expr, $sig:expr, read $read:expr, $($rest:tt)*) => {
        compile_error!("`read` comes after `at`, and nothing comes after `read`.")
    };
    ($($other:tt)*) => {
        compile_error!(
            "a rule is `(Kind, BYTES, at [..], read N)` or `(Kind, via PREDICATE, read N)`. \
             This one is neither. See `magic_rules!`."
        )
    };

}
