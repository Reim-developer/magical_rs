//! Verification harness: every code example in readme.md, compiled and run.
//! If this file compiles and passes, the README examples are correct.
//!
//! Gated on `std` because `read_file_header` is, and these examples read real
//! files from the checkout. Without the gate this file fails to compile rather
//! than skipping, so `cargo test --no-default-features` was never actually
//! runnable and a failure there would have looked like a broken build.

// Each example is a `std` example: they open files, so they cannot be checked
// against the `no_std` build. The `no_std` guarantee is covered by building
// for `thumbv7em-none-eabi`, not by these tests.
#![cfg(feature = "std")]

use magical_rs::magical::bytes_read::{read_file_header, with_bytes_read};
use magical_rs::magical::magic::FileKind;
use magical_rs::{all_matches, magic_custom, match_custom};

#[cfg(feature = "unsafe_context")]
use core::slice;
#[cfg(feature = "unsafe_context")]
use magical_rs::magical::magic_custom::{CustomMatchRules, MagicCustom, match_types_custom};

/// README: Quick start
#[test]
fn readme_quick_start() {
    let header = read_file_header("Cargo.toml", with_bytes_read()).unwrap();
    assert!(FileKind::match_types(&header).is_none());
}

/// README: "`with_bytes_read()` returns 36,870 bytes" — this is a factual
/// claim in the README, so it gets a test.
#[test]
fn readme_documented_read_size_is_accurate() {
    assert_eq!(with_bytes_read(), 36_870);
}

/// README: Level 1
#[test]
fn readme_level_1() {
    let kind = FileKind::match_types(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    assert_eq!(kind, Some(FileKind::Png));
}

/// README: Level 2
#[test]
fn readme_level_2() {
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum Kind {
        CadFile,
        Fallback,
    }

    fn is_cad(bytes: &[u8]) -> bool {
        bytes.starts_with(b"ACAD")
    }

    fn is_short(bytes: &[u8]) -> bool {
        bytes.len() <= 4
    }

    let rule = magic_custom!(
        signatures: [b"ACAD"],
        offsets: [0],
        max_bytes_read: 2048,
        kind: Kind::CadFile,
        rules: all_matches!(is_cad, is_short)
    );

    let result = match_custom!(bytes: b"ACAD", rules: [rule], fallback: Kind::Fallback);

    assert_eq!(result, Kind::CadFile);
}

/// README: Level 5
#[cfg(feature = "unsafe_context")]
#[test]
fn readme_level_5() {
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum Kind {
        Matched,
        Fallback,
    }

    const MAGIC_LEN: usize = 8;

    unsafe fn starts_with_marker(ptr_data: *const ()) -> bool {
        // SAFETY: caller guarantees `ptr_data` is readable for `MAGIC_LEN` bytes.
        let ptr = ptr_data.cast::<u8>();
        unsafe { slice::from_raw_parts(ptr, MAGIC_LEN).starts_with(b"MAGICALG") }
    }

    let rules: &[MagicCustom<Kind>] = &[MagicCustom {
        signatures: &[],
        offsets: &[],
        max_bytes_read: 200,
        kind: Kind::Matched,
        rules: CustomMatchRules::AllMatchesUnsafe(&[starts_with_marker]),
    }];

    let result = match_types_custom(b"MAGICALG", rules, Kind::Fallback);
    assert_eq!(result, Kind::Matched);
}

/// README: feature-flag table claims levels 3 and 4 need their flags.
#[cfg(feature = "magical_dyn")]
#[test]
fn readme_level_3() {
    use magical_rs::magical::dyn_magic::DynMagicCustom;

    let rule = DynMagicCustom::new(
        |bytes: &[u8]| bytes.starts_with(b"MAGICAL"),
        String::from("custom format"),
        32,
    );

    assert!(rule.matches(b"MAGICAL"));
    assert_eq!(rule.kind_downcast_ref::<String>().unwrap(), "custom format");
}

#[cfg(feature = "magical_async_dyn")]
#[test]
fn readme_level_4() {
    use magical_rs::magical::async_dyn_magic::{AsyncDynMagic, match_dyn_types_as};

    // Minimal executor, so this test needs no async runtime dependency.
    fn block_on<F: std::future::Future>(mut fut: F) -> F::Output {
        use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

        const VTABLE: RawWakerVTable = RawWakerVTable::new(
            |_| RawWaker::new(std::ptr::null(), &VTABLE),
            |_| {},
            |_| {},
            |_| {},
        );
        let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) };
        let mut cx = Context::from_waker(&waker);

        // Safety-relevant: single-poll, never-pending future.
        loop {
            if let Poll::Ready(out) =
                unsafe { std::pin::Pin::new_unchecked(&mut fut) }.poll(&mut cx)
            {
                return out;
            }
        }
    }

    let rule = AsyncDynMagic::new(
        |bytes: &[u8]| {
            // The future must be 'static, so it cannot borrow `bytes`.
            let owned = bytes.to_vec();
            async move { owned.starts_with(b"MAGICAL") }
        },
        String::from("custom format"),
        128,
    );

    let rules = [rule];
    let result = block_on(match_dyn_types_as::<String>(b"MAGICAL", &rules));
    assert_eq!(result.map(String::as_str), Some("custom format"));
}

/// README: the `magic_rules!` table, and the `magical_fluent` methods.
///
/// Two in one test because both are the same kind of claim — an example in the
/// readme that runs — and the difference is only which feature has to be on.
/// `magic_rules!` needs none, so it is here unconditionally.
#[test]
fn readme_magic_rules_and_fluent() {
    use magical_rs::magic_rules;
    use magical_rs::magical::magic_custom::{MagicCustom, match_types_custom};

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum Kind {
        CadFile,
        ShortFile,
        Fallback,
    }

    fn is_cad(bytes: &[u8]) -> bool {
        bytes.starts_with(b"ACAD")
    }

    fn is_short(bytes: &[u8]) -> bool {
        bytes.len() <= 4
    }

    // The readme's table, rule for rule.
    static RULES: &[MagicCustom<Kind>] = magic_rules![
        (Kind::CadFile, b"ACAD", read 2048),
        (Kind::ShortFile, [b"<<", b">>"], read 4),
        (Kind::Fallback, via all [is_cad, is_short]),
    ];

    assert_eq!(
        match_types_custom(b"ACAD", RULES, Kind::Fallback),
        Kind::CadFile
    );
    assert_eq!(
        match_types_custom(b">>", RULES, Kind::Fallback),
        Kind::ShortFile
    );
    assert_eq!(
        match_types_custom(b"nope", RULES, Kind::Fallback),
        Kind::Fallback
    );
}

/// README: the `magical_fluent` block, gated the way the readme gates it.
#[cfg(feature = "magical_fluent")]
#[test]
fn readme_fluent() {
    use magical_rs::magical::fluent::Detect;
    use magical_rs::magical::magic::FileKind;

    let gif = b"GIF89a";

    assert_eq!(gif.detect(), Some(FileKind::GIF));
    assert_eq!(gif.detect_within(2_048), Some(FileKind::GIF));
    assert!(gif.is(FileKind::GIF));
    assert!(!gif.is(FileKind::Png));
    assert!(gif.is_any([FileKind::Png, FileKind::GIF]));
}
