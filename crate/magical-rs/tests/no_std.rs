#![no_std]
#[test]
#[cfg(feature = "no_std")]
fn test_no_std() {
    use magical_rs::magical::bytes_read::DEFAULT_MAX_BYTES_READ;
    use magical_rs::magical::magic::FileKind;

    const PNG_BYTES: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
    ];
    let result = FileKind::no_std_match_with_max_read_rule(PNG_BYTES, DEFAULT_MAX_BYTES_READ);

    assert_eq!(result, FileKind::Png);
    assert_ne!(result, FileKind::Unknown);
}
#[test]
#[cfg(feature = "no_std")]
fn test_no_std_with_custom_bytes_read() {
    use magical_rs::magical::magic::FileKind;

    const PNG_BYTES: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
    ];
    const MY_MAX_BYTES_READ: usize = 8;
    let result = FileKind::no_std_match_with_custom_max_read(PNG_BYTES, MY_MAX_BYTES_READ);

    assert_eq!(result, FileKind::Png);
    assert_ne!(result, FileKind::Unknown);
}

#[test]
fn test_no_std_custom() {
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
