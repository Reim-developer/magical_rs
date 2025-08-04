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
        signatures: [b"MagicalGirl"],
        offsets: [0],
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
}
