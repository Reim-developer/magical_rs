/*
* The `unsafe_context` feature is very useful feature
* available in `magical_rs` from version `0.4.0` onwards.
* For security purposes, this feature is disabled by default.
* Don't use if you don't know what you're doing.
* Only use in case of critical performance.
* To use, run the following command with `Cargo`:
* cargo add magical_rs --features unsafe_context
*/

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

        let slice_len: usize = 100; /* Here you need to make sure that your slice length is correct. */

        let slice = slice::from_raw_parts(slice_ptr, slice_len);

        assert_ne!(slice.len(), slice_len + 1); /* Wrong slice len. */
        assert_eq!(slice.len(), 100);
        assert!(!slice.is_empty());
        assert!(slice.starts_with(b"MagicalGirl"));

        slice.starts_with(b"MagicalGirl")
    }
}

fn main() {
    let rules: &[MagicCustom<MagicKind>] = &[MagicCustom {
        signatures: &[],
        offsets: &[],
        max_bytes_read: 200,
        kind: MagicKind::MoeMoe,
        rules: CustomMatchRules::WithFnUnsafe {
            func: is_shoujo_girl,
        },
    }];

    let my_bytes = b"MagicalGirl";
    let result = match_types_custom(my_bytes, rules, MagicKind::UnknownFallback);

    assert_eq!(result, MagicKind::MoeMoe);
    assert_ne!(result, MagicKind::UnknownFallback);
    println!("{result:?}");
}
