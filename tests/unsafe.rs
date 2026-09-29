//! The `unsafe_context` predicates, and the one obligation they impose.
//!
//! `WithFnUnsafe`, `AnyMatchesUnsafe` and `AllMatchesUnsafe` hand the callee a
//! `*const ()` and no length. There is nothing on the callee's side to check
//! the caller's promise against, so the promise has to be true: the buffer
//! passed to `match_types_custom` has to be at least as long as the predicate
//! reads.
//!
//! These tests used to pass `b"MagicalGirl"`, 11 bytes, to predicates that read
//! 100. A normal build passes that, because the allocator usually hands out a
//! larger block and the 90 bytes past the end happen to be mapped. Miri does
//! not, and reports what it is:
//!
//! ```text
//! error: Undefined Behavior: constructing invalid value of type &[u8]:
//!        encountered a dangling reference (going beyond the bounds of its
//!        allocation)
//!     --> tests/unsafe.rs:109:25
//! ```
//!
//! So `make test-unsafe` exiting 0 was never evidence this was sound. The
//! buffer below is [`READ_LEN`] bytes, which is exactly what the predicates ask
//! for, and that is the whole fix.

/// How many bytes each predicate below reads.
///
/// A constant on the caller's side rather than inside the predicate, because
/// the caller is the only side that can promise the bytes are there.
#[cfg(feature = "unsafe_context")]
const READ_LEN: usize = 100;

/// A buffer of exactly [`READ_LEN`] bytes, starting with the marker.
///
/// An array rather than a `Vec`, so the length the predicate is allowed to read
/// is a type rather than a number in a comment.
#[cfg(feature = "unsafe_context")]
fn buffer() -> [u8; READ_LEN] {
    let mut data = [0u8; READ_LEN];
    let marker = b"MagicalGirl";
    data[..marker.len()].copy_from_slice(marker);
    data
}

/// Matches the marker, over [`READ_LEN`] bytes.
#[cfg(feature = "unsafe_context")]
fn is_shoujo_girl(data: *const ()) -> bool {
    let ptr = data.cast::<u8>();
    // SAFETY: `buffer` is `READ_LEN` bytes and `match_types_custom` passes a
    // pointer into the buffer it was given, so `READ_LEN` bytes are readable
    // from it. The caller is untrustworthy in general, which is what makes
    // this `unsafe fn` at all; here the caller is the test below.
    unsafe { core::slice::from_raw_parts(ptr, READ_LEN).starts_with(b"MagicalGirl") }
}

/// The exact negation of [`is_shoujo_girl`], so that the `AllMatchesUnsafe`
/// test below has two predicates that cannot both be true.
#[cfg(feature = "unsafe_context")]
fn is_not_shoujo_girl(data: *const ()) -> bool {
    let ptr = data.cast::<u8>();
    // SAFETY: as `is_shoujo_girl`.
    unsafe { !core::slice::from_raw_parts(ptr, READ_LEN).starts_with(b"MagicalGirl") }
}

#[test]
#[cfg(feature = "unsafe_context")]
fn test_unsafe_magic_custom() {
    use magical_rs::magical::magic_custom::match_types_custom;
    use magical_rs::magical::magic_custom::{CustomMatchRules, MagicCustom};

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum MagicKind {
        MoeMoe,
        UnknownFallback,
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

    let data = buffer();
    let result = match_types_custom(&data, rules, MagicKind::UnknownFallback);

    assert_eq!(result, MagicKind::MoeMoe);
    assert_ne!(result, MagicKind::UnknownFallback);
}

#[test]
#[cfg(feature = "unsafe_context")]
fn test_any_unsafe_fn() {
    use magical_rs::magical::magic_custom::match_types_custom;
    use magical_rs::magical::magic_custom::{CustomMatchRules, MagicCustom};

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum MagicKind {
        MoeMoe,
        UnknownFallback,
    }

    let rules: &[MagicCustom<MagicKind>] = &[MagicCustom {
        signatures: &[],
        offsets: &[],
        max_bytes_read: 200,
        kind: MagicKind::MoeMoe,
        /*
         * The two functions are logically contradictory, and `Any` still
         * crosses them: either one matching is enough.
         */
        rules: CustomMatchRules::AnyMatchesUnsafe(&[is_shoujo_girl, is_not_shoujo_girl]),
    }];

    let data = buffer();
    let result = match_types_custom(&data, rules, MagicKind::UnknownFallback);

    assert_eq!(result, MagicKind::MoeMoe);
    assert_ne!(result, MagicKind::UnknownFallback);
}

#[test]
#[cfg(feature = "unsafe_context")]
fn test_all_unsafe_fn() {
    use magical_rs::magical::magic_custom::match_types_custom;
    use magical_rs::magical::magic_custom::{CustomMatchRules, MagicCustom};

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum MagicKind {
        MoeMoe,
        UnknownFallback,
    }

    let rules: &[MagicCustom<MagicKind>] = &[MagicCustom {
        signatures: &[],
        offsets: &[],
        max_bytes_read: 200,
        kind: MagicKind::MoeMoe,
        /*
         * The same contradictory pair, and `All` cannot cross it: both would
         * have to hold at once, so the rule declines and the fallback wins.
         */
        rules: CustomMatchRules::AllMatchesUnsafe(&[is_shoujo_girl, is_not_shoujo_girl]),
    }];

    let data = buffer();
    let result = match_types_custom(&data, rules, MagicKind::UnknownFallback);

    assert_ne!(result, MagicKind::MoeMoe);
    assert_eq!(result, MagicKind::UnknownFallback);
}
