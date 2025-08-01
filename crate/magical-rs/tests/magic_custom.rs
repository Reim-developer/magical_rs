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
    use magical_rs::magical::magic_custom::{CustomMatchRules, MagicCustom, match_types_custom};

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
        rules: CustomMatchRules::WithFn(is_shoujo_girl),
    };

    let magical_girl = b"MagicalGirl";
    let result = match_types_custom(magical_girl, &[SHOUJO_RULE], ShoujuFile::Unknown);

    assert_eq!(result, ShoujuFile::MahouShouju);
    assert_ne!(result, ShoujuFile::Unknown);
}
