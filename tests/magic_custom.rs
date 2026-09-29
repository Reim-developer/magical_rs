#[test]
fn test_match_types_custom() {
    use magical_rs::magical::magic_custom::{CustomMatchRules, MagicCustom, match_types_custom};
    #[derive(Debug, Clone, Copy, PartialEq)]
    enum FileKind {
        Png,
        Unknown,
    }

    const PNG_SIGNATURE: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    static PNG_RULE: MagicCustom<FileKind> = MagicCustom {
        signatures: &[PNG_SIGNATURE],
        offsets: &[0],
        max_bytes_read: 2048,
        kind: FileKind::Png,
        rules: CustomMatchRules::Default,
    };

    const PNG_BYTES: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
    ];

    let result = match_types_custom(PNG_BYTES, &[PNG_RULE], FileKind::Unknown);

    assert_eq!(result, FileKind::Png);
    assert_ne!(result, FileKind::Unknown);
}

#[test]
fn test_with_custom_rules() {
    use magical_rs::magical::magic_custom::{MagicCustom, match_types_custom};
    use magical_rs::with_fn_matches;

    #[derive(Debug, Clone, Copy, PartialEq)]
    enum ShoujuFile {
        MahouShouju,
        Unknown,
    }

    fn is_shoujo_girl(bytes: &[u8]) -> bool {
        bytes.starts_with(b"MagicalGirl")
    }

    static SHOUJO_RULE: MagicCustom<ShoujuFile> = MagicCustom {
        signatures: &[],
        offsets: &[],
        max_bytes_read: 2048,
        kind: ShoujuFile::MahouShouju,
        rules: with_fn_matches!(is_shoujo_girl),
    };

    let magical_girl = b"MagicalGirl";
    let result = match_types_custom(magical_girl, &[SHOUJO_RULE], ShoujuFile::Unknown);

    assert_eq!(result, ShoujuFile::MahouShouju);
    assert_ne!(result, ShoujuFile::Unknown);
}

#[test]
fn test_with_any_matches() {
    use magical_rs::any_matches;
    use magical_rs::magic_custom;
    use magical_rs::match_custom;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum CuteGirlKind {
        ShoujoFile,
        UnknownFallback,
    }

    fn find_shoujo_girl(bytes: &[u8]) -> bool {
        bytes.starts_with(b"MagicalGirl")
    }

    fn wrong_shoujo_girl(bytes: &[u8]) -> bool {
        !bytes.starts_with(b"MagicalGirl")
    }

    let rule = magic_custom! (
        signatures: [],
        offsets: [],
        max_bytes_read: 69,
        kind: CuteGirlKind::ShoujoFile,
        rules: any_matches!(find_shoujo_girl, wrong_shoujo_girl)
    );

    let result = match_custom! {
        bytes: b"MagicalGirl",
        rules: [rule],
        fallback: CuteGirlKind::UnknownFallback
    };

    assert_eq!(result, CuteGirlKind::ShoujoFile);
    assert_ne!(result, CuteGirlKind::UnknownFallback);
}

#[test]
fn test_with_all_matches() {
    use magical_rs::all_matches;
    use magical_rs::magic_custom;
    use magical_rs::match_custom;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum CuteGirlKind {
        ShoujoFile,
        UnknownFallback,
    }

    fn find_shoujo_girl(bytes: &[u8]) -> bool {
        bytes.starts_with(b"MagicalGirl")
    }

    fn wrong_shoujo_girl(bytes: &[u8]) -> bool {
        !bytes.starts_with(b"MagicalGirl")
    }

    let rule = magic_custom! (
        signatures: [],
        offsets: [],
        max_bytes_read: 69,
        kind: CuteGirlKind::ShoujoFile,
        rules: all_matches!(find_shoujo_girl, wrong_shoujo_girl)
    );

    let result = match_custom! {
        bytes: b"MagicalGirl",
        rules: [rule],
        fallback: CuteGirlKind::UnknownFallback
    };

    assert_eq!(result, CuteGirlKind::UnknownFallback);
    assert_ne!(result, CuteGirlKind::ShoujoFile);
}

/// The `CustomMatchRules::Default` arm of `matches_custom`, branch by branch.
///
/// The Python binding cannot call this code. `MagicCustom` holds `&'static`
/// slices, because a level 2 rule is meant to be a `static`, so a rule
/// assembled from Python data at run time would have to be `Box::leak`ed and a
/// process that builds rules in a loop would leak without bound. The binding
/// therefore reproduces the eight-line comparison in `signatures_match`.
///
/// A copy that nothing defines is just a second thing to believe. This module
/// is the definition: every branch below is pinned against the crate's own
/// `match_types_custom`, and `tests/test_levels.py` in the bindings crate pins
/// the copy against the same branches. A change to either side is then a test
/// failure rather than a detection that quietly stops working.
#[cfg(feature = "std")]
mod default_arm {
    use magical_rs::magical::magic_custom::{CustomMatchRules, MagicCustom, match_types_custom};

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Kind {
        Matched,
        Fallback,
    }

    const ACAD: &[u8] = b"ACAD";
    const MAGIC: &[u8] = b"MAGIC";

    /// Builds a real `MagicCustom`, which needs `&'static`.
    ///
    /// Leaking is acceptable in a test and nowhere else, which is the whole
    /// reason the binding cannot do this.
    fn rule(signatures: &[&'static [u8]], offsets: &[usize]) -> MagicCustom<'static, Kind> {
        let signatures: &'static [&'static [u8]] =
            Box::leak(signatures.to_vec().into_boxed_slice());
        let offsets: &'static [usize] = Box::leak(offsets.to_vec().into_boxed_slice());
        MagicCustom {
            signatures,
            offsets,
            max_bytes_read: 2048,
            kind: Kind::Matched,
            rules: CustomMatchRules::Default,
        }
    }

    /// Whether `data` satisfies the rule, told apart from the fallback.
    fn matches(data: &[u8], signatures: &[&'static [u8]], offsets: &[usize]) -> bool {
        let rule = rule(signatures, offsets);
        match_types_custom(data, &[rule], Kind::Fallback) == Kind::Matched
    }

    #[test]
    fn matches_a_signature_at_offset_zero() {
        assert!(matches(b"ACAD\0\0", &[ACAD], &[0]));
    }

    #[test]
    fn matches_a_signature_at_a_positive_offset() {
        assert!(matches(b"xxxxACAD", &[ACAD], &[4]));
    }

    #[test]
    fn matches_when_the_signature_ends_exactly_at_the_end() {
        // The bound is `data.len() >= offset + signature.len()`, so a signature
        // finishing on the final byte matches.
        assert!(matches(b"xxxxACAD", &[ACAD], &[4]));
    }

    #[test]
    fn rejects_an_offset_past_the_end() {
        assert!(!matches(b"xxxxACAD", &[ACAD], &[5]));
        assert!(!matches(b"xxxxACAD", &[ACAD], &[99]));
    }

    #[test]
    fn rejects_a_signature_one_byte_short() {
        // The bound is on the signature, not on the offset: three bytes of
        // `ACAD` at offset 0 is not a match, and neither is four bytes when
        // only three remain.
        assert!(!matches(b"ACA", &[ACAD], &[0]));
        assert!(!matches(b"xxxACA", &[ACAD], &[3]));
    }

    #[test]
    fn rejects_empty_input() {
        assert!(!matches(b"", &[ACAD], &[0]));
    }

    #[test]
    fn accepts_the_second_of_several_signatures() {
        assert!(matches(b"MAGIC", &[ACAD, MAGIC], &[0]));
    }

    #[test]
    fn accepts_the_second_of_several_offsets() {
        assert!(matches(b"MAGIC", &[MAGIC], &[99, 0]));
    }

    #[test]
    fn accepts_the_right_pair_of_several_of_each() {
        assert!(matches(b"__ACAD", &[ACAD, MAGIC], &[0, 2]));
    }

    #[test]
    fn rejects_when_no_combination_matches() {
        assert!(!matches(b"MAGIC", &[ACAD], &[0, 1]));
    }

    #[test]
    fn rejects_when_either_side_is_empty() {
        // The arm is `signatures.any(offsets.any(...))`, so an empty side can
        // never match, whatever the other side holds.
        assert!(!matches(b"ACAD", &[], &[0]));
        assert!(!matches(b"ACAD", &[ACAD], &[]));
    }

    #[test]
    fn accepts_an_empty_signature_at_offset_zero() {
        // A consequence of the comparison rather than a rule anyone wants, but
        // the binding reproduces it, so it is pinned here too.
        assert!(matches(b"anything", &[b""], &[0]));
    }
}
