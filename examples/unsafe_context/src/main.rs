/*
* The `unsafe_context` feature is very useful feature
* available in `magical_rs` from version `0.4.0` onwards.
* For security purposes, this feature is disabled by default.
* Don't use if you don't know what you're doing.
* Only use in case of critical performance.
* To use, run the following command with `Cargo`:
* cargo add magical_rs --features unsafe_context
*
* The feature hands your predicate a `*const ()` and no length. There is
* nothing on the predicate's side to check your promise against, so the promise
* is the whole safety argument: the buffer you pass has to be at least as long
* as the predicate reads. This example reads `READ_LEN` bytes and passes a
* buffer of exactly that many.
*/

use core::slice;
use magical_rs::magical::magic_custom::match_types_custom;
use magical_rs::magical::magic_custom::{CustomMatchRules, MagicCustom};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum MagicKind {
    MoeMoe,
    UnknownFallback,
}

/// How many bytes `is_shoujo_girl` reads.
const READ_LEN: usize = 100;

fn is_shoujo_girl(data: *const ()) -> bool {
    let slice_ptr = data.cast::<u8>();
    // SAFETY: `main` passes a `[u8; READ_LEN]`, and `match_types_custom` hands
    // the predicate a pointer into the buffer it was given, so `READ_LEN`
    // bytes are readable from it. A caller that passes a shorter buffer makes
    // this line a read out of bounds, which is why the function is `unsafe fn`
    // and why this comment is part of the contract rather than decoration.
    let slice = unsafe { slice::from_raw_parts(slice_ptr, READ_LEN) };

    assert_eq!(slice.len(), READ_LEN);
    assert!(!slice.is_empty());
    assert!(slice.starts_with(b"MagicalGirl"));

    slice.starts_with(b"MagicalGirl")
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

    /* The buffer has to be at least `READ_LEN` bytes. A `&[u8]` literal like
     * `b"MagicalGirl"` is 11, and reading 100 out of it is out of bounds even
     * when the allocator happens to leave the following bytes mapped. */
    let mut data = [0u8; READ_LEN];
    data[..b"MagicalGirl".len()].copy_from_slice(b"MagicalGirl");

    let result = match_types_custom(&data, rules, MagicKind::UnknownFallback);

    assert_eq!(result, MagicKind::MoeMoe);
    assert_ne!(result, MagicKind::UnknownFallback);
    println!("{result:?}");
}
