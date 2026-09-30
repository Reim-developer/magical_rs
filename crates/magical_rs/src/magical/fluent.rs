//! `Detect` — `bytes.detect()` instead of `FileKind::match_types(&bytes)`.
//!
//! # Why this is behind a feature flag
//!
//! Every other addition to this crate is either always compiled or a `macro_rules!`
//! that costs nothing until invoked. This one is different, and the reason is that
//! it puts a name on a type the caller does not own. `bytes.detect()` is a method on
//! `[u8]`, so a caller who writes it has taken a dependency on this crate for a
//! slice of bytes they could have passed to anything. That is a decision to make,
//! not one to make for them, and it cannot be undone by the caller short of not
//! writing it. So it is opt-in.
//!
//! Turning the feature off must leave *nothing* behind — not a trait, not a type,
//! not a re-export. `tests/magical_fluent.rs` asserts that by reading `lib.rs` and
//! checking the module is mentioned only on a `#[cfg(feature = "magical_fluent")]`
//! line, and the `make test-nostd` target builds the crate with the feature absent.
//! A `#[cfg]` on the module body but not on the `pub mod` line would compile with
//! the feature off and still be visible in the docs, which is the version of this
//! that a reader would have to notice for themselves.
//!
//! # What it does not do
//!
//! It adds no matching. Every method here is a named call to
//! [`FileKind::match_types`], [`dispatch::first_match`] or a `const fn` on
//! [`FileKind`], and the answers are the ones the ungated API gives. `detect()` is
//! `match_types`, spelled the other way round.

use crate::magical::dispatch;
use crate::magical::magic::FileKind;
use crate::magical::magic_custom::{MagicCustom, match_types_custom};
use crate::magical::signatures::SIGNATURE_KIND;

/// Detection, as methods on the bytes themselves.
///
/// Implemented for `[u8]`, and reached through `&self`, so it works for a `Vec`,
/// a `[u8; N]`, a `str`'s bytes, or a borrowed slice without any of them knowing.
///
/// ```
/// use magical_rs::magical::fluent::Detect;
///
/// let png = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
/// assert!(png.detect().is_some());
///
/// // A file this table does not recognise, rather than a guess.
/// assert!([b'.'; 64].detect().is_none());
/// ```
pub trait Detect {
    /// The format these bytes are, or `None` if the table does not recognise them.
    ///
    /// The same answer as [`FileKind::match_types`]: first match in
    /// `SIGNATURE_KIND` order, over the whole table, with no `max_bytes_read`
    /// filter. That filter is what [`Detect::detect_within`] is for.
    ///
    /// ```
    /// use magical_rs::magical::fluent::Detect;
    /// use magical_rs::magical::magic::FileKind;
    ///
    /// assert_eq!(b"GIF89a".detect(), Some(FileKind::GIF));
    /// ```
    fn detect(&self) -> Option<FileKind>;

    /// The format these bytes are, using only rules that fit inside `max_bytes_read`.
    ///
    /// `max_bytes_read` is a window the caller claims to hold, not a length `bytes`
    /// is checked against. A 100-byte file classified with a 2,048-byte window is
    /// ordinary.
    ///
    /// ```
    /// use magical_rs::magical::fluent::Detect;
    /// use magical_rs::magical::magic::FileKind;
    ///
    /// // ISO 9660 stores its magic at 32,769, and declares a read size of 36,870.
    /// let mut iso = [0u8; 40_000];
    /// iso[32_769..32_774].copy_from_slice(b"CD001");
    ///
    /// assert_eq!(iso.detect_within(36_870), Some(FileKind::ISO));
    /// assert_eq!(iso.detect_within(2_048), None);
    /// ```
    fn detect_within(&self, max_bytes_read: usize) -> Option<FileKind>;

    /// Whether these bytes are this one format, ignoring everything else.
    ///
    /// Not the same question as `detect() == Some(kind)`, and the difference is
    /// worth naming: this asks whether *that format's own rule* matches, which
    /// ignores every rule that would have been tried first. A file that is both
    /// CBOR and, by its first bytes, some earlier format in the table is the
    /// earlier format as far as `detect()` is concerned and is still "yes, it is
    /// CBOR" as far as this is concerned.
    ///
    /// ```
    /// use magical_rs::magical::fluent::Detect;
    /// use magical_rs::magical::magic::FileKind;
    ///
    /// assert!(b"GIF89a".is(FileKind::GIF));
    /// assert!(!b"GIF89a".is(FileKind::Png));
    /// ```
    fn is(&self, kind: FileKind) -> bool;

    /// Whether these bytes are any of the named formats, in one lookup each.
    ///
    /// Takes an iterator rather than a slice so a caller can pass a `&[FileKind]`
    /// or a computed set without collecting one first. The formats are tried in
    /// the order given, and the answer is the first that matches.
    ///
    /// ```
    /// use magical_rs::magical::fluent::Detect;
    /// use magical_rs::magical::magic::FileKind;
    ///
    /// assert!(b"GIF89a".is_any([FileKind::Png, FileKind::GIF]));
    /// assert!(!b"GIF89a".is_any([FileKind::Png, FileKind::Jpg]));
    /// ```
    fn is_any<I>(&self, kinds: I) -> bool
    where
        I: IntoIterator<Item = FileKind>,
    {
        kinds.into_iter().any(|kind| self.is(kind))
    }
}

// There is deliberately no `detect_or`.
//
// It was here first, documented as "a `map_or` that does not evaluate the
// fallback unless it is needed", and that was false: a by-value argument is
// evaluated before the call whatever the callee does with it, so a caller passing
// an expensive fallback paid for it on every call, with nothing in the signature
// to say so. The method was also a spelling of `self.detect().unwrap_or(fallback)`
// and saved nothing. It is gone rather than kept with a corrected comment.
//
// The version that *is* lazy is `detect()` plus `unwrap_or_else` at the call site,
// and a caller who wants that can write it themselves. What makes it worth not
// having a wrapper is that `FileKind` has no "unknown" variant — all 114 of them
// are real formats — so there is no honest value for a fallback to be, and an enum
// carrying one would be claiming a knowledge it does not have.

/// The same trait for a level 2 rule set, so a caller's own rules read the same
/// way as the built-in table.
///
/// Separate from [`Detect`] because the element type differs: the built-in table
/// decides what `detect()` returns, and this one lets the caller decide. Merging
/// them would mean every `detect()` call paying for a generic parameter it does
/// not use.
pub trait DetectRules<K: Clone> {
    /// The kind of the first rule that matches, or `fallback`.
    ///
    /// This is [`match_types_custom`] with the rule list left to the caller, which
    /// is the thing a level 2 caller actually writes. Order is the answer, so the
    /// rules are tried in the order given.
    ///
    /// ```
    /// use magical_rs::magic_rules;
    /// use magical_rs::magical::fluent::DetectRules;
    /// use magical_rs::magical::magic_custom::MagicCustom;
    ///
    /// #[derive(Clone, Copy, Debug, PartialEq)]
    /// enum Kind {
    ///     MahouShoujo,
    ///     Unknown,
    /// }
    ///
    /// static RULES: &[MagicCustom<Kind>] = magic_rules![(Kind::MahouShoujo, b"MagicalGirl")];
    ///
    /// assert_eq!(b"MagicalGirl".detect_in(RULES, Kind::Unknown), Kind::MahouShoujo);
    /// assert_eq!(b"something else".detect_in(RULES, Kind::Unknown), Kind::Unknown);
    /// ```
    fn detect_in(&self, rules: &[MagicCustom<'_, K>], fallback: K) -> K;
}

impl Detect for [u8] {
    #[inline]
    fn detect(&self) -> Option<FileKind> {
        FileKind::match_types(self)
    }

    #[inline]
    fn detect_within(&self, max_bytes_read: usize) -> Option<FileKind> {
        dispatch::first_match(self, max_bytes_read).map(|index| SIGNATURE_KIND[index].kind)
    }

    #[inline]
    fn is(&self, kind: FileKind) -> bool {
        SIGNATURE_KIND
            .iter()
            .find(|magic| magic.kind == kind)
            .is_some_and(|magic| magic.matches(self))
    }
}

impl<K: Clone> DetectRules<K> for [u8] {
    #[inline]
    fn detect_in(&self, rules: &[MagicCustom<'_, K>], fallback: K) -> K {
        match_types_custom(self, rules, fallback)
    }
}
