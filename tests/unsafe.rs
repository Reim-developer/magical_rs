#[test]
#[cfg(feature = "unsafe_context")]
fn test_unsafe_magic_custom() {
    use core::slice;
    use magical_rs::magical::magic_custom::match_types_custom;
    use magical_rs::magical::magic_custom::{CustomMatchRules, MagicCustom};

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum MagicKind {
        MoeMoe,
        UnknownFallback,
    }

    fn is_shoujo_girl(data: *const ()) -> bool {
        unsafe {
            let slice_ptr = data.cast::<u8>();
            /*
             * Here we assume that, we have the length of data as 100.
             * In reality, you need to make sure it is not too long or short.
             */
            let slice = slice::from_raw_parts(slice_ptr, 100);

            slice.starts_with(b"MagicalGirl")
        }
    }

    let rules: &[MagicCustom<MagicKind>] = &[MagicCustom {
        signatures: &[],
        offsets: &[],
        max_bytes_read: 200,
        kind: MagicKind::MoeMoe,
        rules: CustomMatchRules::WithFnUnsafe {
            func: is_shoujo_girl,
        },
    }];

    let result = match_types_custom(b"MagicalGirl", rules, MagicKind::UnknownFallback);

    assert_eq!(result, MagicKind::MoeMoe);
    assert_ne!(result, MagicKind::UnknownFallback);
}
