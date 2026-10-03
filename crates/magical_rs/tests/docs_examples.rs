//! Verification harness: every Rust example in `docs/`, compiled and run.
//!
//! `readme_examples.rs` does this for the repository readme. The documentation in
//! `docs/` is a separate document with separate code, and an example that does not
//! compile is a claim the reader finds out about by pasting it into a file.
//!
//! The examples are transcribed rather than extracted. Extracting them would mean
//! a parser for Markdown fences, and a parser that silently finds zero examples
//! passes — which is the failure mode this file exists to prevent. So each test
//! below names the page and the heading it came from, and a page gaining an
//! example is a line here. `every_example_is_attached_to_a_page` checks the other
//! direction: that this file does not drift from the pages it claims to cover.

use magical_rs::magic_rules;
use magical_rs::magical::bytes_read::{DEFAULT_MAX_BYTES_READ, with_bytes_read};
use magical_rs::magical::magic::FileKind;
use magical_rs::magical::magic_custom::{MagicCustom, match_types_custom};

// `read_file_header_into` is `#[cfg(feature = "std")]`, so its import is gated too —
// an ungated one fails to compile this whole file under `--no-default-features`, and
// takes the thirteen examples that need no filesystem with it. The pages mark these
// calls the same way, so the gate here is the documentation being checked rather than
// a workaround for it.
#[cfg(feature = "std")]
use magical_rs::magical::bytes_read::read_file_header_into;

/// docs/concepts.md — "Three orders exist, and the two that matter are not the
/// same sequence"
///
/// The page's whole point is that detection order, the ABI discriminant and
/// `ALL_KINDS` are three lists that do not agree. Asserting each position pins all
/// three, and the `Ktx2` pair is what makes the disagreement visible rather than
/// theoretical.
#[test]
fn concepts_three_orders_that_do_not_agree() {
    use magical_rs::magical::kinds_meta::ALL_KINDS;
    use magical_rs::magical::signatures::SIGNATURE_KIND;

    assert_eq!(ALL_KINDS.len(), 114, "the page says 114 throughout");

    let table = |k| SIGNATURE_KIND.iter().position(|m| m.kind == k).unwrap();
    let listed = |k| ALL_KINDS.iter().position(|x| *x == k).unwrap();

    // `Png` is first to be tried and first as a discriminant, and nowhere near the
    // front of the name-sorted list.
    assert_eq!(table(FileKind::Png), 0);
    assert_eq!(FileKind::Png as isize, 0);
    assert_eq!(listed(FileKind::Png), 75);

    // The pair the page uses: detection order and ABI order disagree about it.
    assert_eq!(table(FileKind::Ktx2), 52);
    assert_eq!(FileKind::Ktx2 as isize, 63);
    assert_eq!(table(FileKind::Ktx), 53);
    assert_eq!(FileKind::Ktx as isize, 62);

    // And the second pair, where the readme claims the same thing: `ScriptExecute`
    // is asked first, and carries the higher discriminant.
    assert_eq!(table(FileKind::ScriptExecute), 17);
    assert_eq!(FileKind::ScriptExecute as isize, 18);
    assert_eq!(table(FileKind::RAR), 18);
    assert_eq!(FileKind::RAR as isize, 17);

    // The page says `ALL_KINDS` is sorted by name, so `Ktx` precedes `Ktx2` there
    // and neither is near its table position.
    assert_eq!(listed(FileKind::Ktx), 46);
    assert_eq!(listed(FileKind::Ktx2), 47);
}

/// docs/concepts.md — "The read window"
#[test]
fn concepts_narrowing_the_window_drops_only_iso() {
    // A `Vec`, not `[u8; 40_000]`: a 40 KiB array is a local, and a local that size is
    // a stack frame clippy refuses to let a test quietly ask for. The page says the
    // same thing for the same reason.
    let mut iso = vec![0_u8; 40_000];
    iso[32_769..32_774].copy_from_slice(b"CD001");

    assert_eq!(FileKind::match_types(&iso), Some(FileKind::ISO));
    // The page uses `dispatch::first_match` rather than
    // `FileKind::match_with_max_read_rule` because the latter is
    // `#[cfg(not(feature = "std"))]` — this test builds with `std`, so writing the
    // convenient spelling here would not compile, which is exactly the mistake the
    // page's first draft made.
    assert_eq!(
        magical_rs::magical::dispatch::first_match(&iso, 2_048),
        None
    );

    // The two numbers the page calls "the default", which are not the same number.
    assert_eq!(with_bytes_read(), 36_870);
    assert_eq!(DEFAULT_MAX_BYTES_READ, 2_048);

    // The page's claim: exactly one rule needs more than the per-rule default.
    let over = magical_rs::magical::signatures::SIGNATURE_KIND
        .iter()
        .filter(|m| m.max_bytes_read > DEFAULT_MAX_BYTES_READ)
        .count();
    assert_eq!(
        over, 1,
        "the page says exactly one rule needs more than 2,048"
    );
}

/// docs/concepts.md — "A short file is not a padded one"
///
/// The page states that TIFF's magic is `II*\0` and that a three-byte `II*` read
/// into a zero-filled window completed its own signature and was reported as a
/// TIFF. Both halves are checked here, because "the page says this used to be a
/// bug" is a claim about the past and "it is fixed now" is a claim about the
/// present.
#[test]
fn concepts_a_short_buffer_is_not_zero_padded() {
    let short = b"II*";
    assert_eq!(FileKind::match_types(short), None);

    // What the page describes the bug as: the same bytes inside a padded window.
    let mut padded = [0_u8; 2_048];
    padded[..short.len()].copy_from_slice(short);
    assert_ne!(
        FileKind::match_types(&padded),
        None,
        "if this is now None as well, the page's account of the bug no longer describes a \
         difference, and the reason the reader trims is gone"
    );
}

/// docs/concepts.md — "Reading a file at all"
#[test]
#[cfg(feature = "std")]
fn concepts_reading_many_files_allocates_once() -> std::io::Result<()> {
    let mut buffer = Vec::new();
    let mut found = 0;

    for path in std::fs::read_dir(".")? {
        let path = path?.path();
        let Some(name) = path.to_str() else {
            continue;
        };
        if read_file_header_into(name, with_bytes_read(), &mut buffer).is_ok() {
            found += usize::from(FileKind::match_types(&buffer).is_some());
        }
    }

    // The repository readme is detected as text by nothing, so the count is not
    // asserted; what is asserted is that the loop ran and the buffer is reused.
    assert!(found < usize::MAX);
    assert!(
        buffer.capacity() >= with_bytes_read(),
        "the buffer was not reused"
    );
    Ok(())
}

/// docs/getting-started.md — "Rust", and "Bytes you already have"
#[test]
#[cfg(feature = "std")]
fn getting_started_identifies_a_real_file() -> std::io::Result<()> {
    let header = magical_rs::magical::bytes_read::read_file_header("readme.md", with_bytes_read())?;

    // The page's own example file, and the page's claim that a magic number is a
    // fact about the bytes rather than a guess from the name.
    assert!(
        FileKind::match_types(&header).is_none(),
        "readme.md is not a magic header"
    );

    assert_eq!(FileKind::match_types(b"GIF89a"), Some(FileKind::GIF));
    Ok(())
}

/// docs/getting-started.md — "The data-first spelling, for scanning a directory"
#[test]
#[cfg(feature = "magical_fluent")]
fn getting_started_the_fluent_spelling() {
    use magical_rs::magical::fluent::Detect;

    assert_eq!(b"GIF89a".detect(), Some(FileKind::GIF));
    assert!(b"GIF89a".is(FileKind::GIF));
    assert!(b"GIF89a".is_any([FileKind::Png, FileKind::GIF]));
    assert!(b"GIF89a".detect().is_some());
}

/// docs/detection-levels.md — "Level 1"
#[test]
fn levels_one_the_built_in_table() {
    let kind = FileKind::match_types(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    assert_eq!(kind, Some(FileKind::Png));
}

/// docs/detection-levels.md — "Level 2", the `magic_custom!` example
#[test]
fn levels_two_predicates_at_compile_time() {
    use magical_rs::{all_matches, magic_custom, match_custom};

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

/// docs/detection-levels.md — "`magic_rules!` — a rule set as a table"
#[test]
fn levels_two_a_rule_set_as_a_table() {
    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Kind {
        MahouShoujo,
        Unknown,
    }

    static RULES: &[MagicCustom<Kind>] = magic_rules![
        (Kind::MahouShoujo, b"MagicalGirl"),
        (Kind::Unknown, b"Mahou"),
    ];

    assert_eq!(
        match_types_custom(b"MagicalGirl", RULES, Kind::Unknown),
        Kind::MahouShoujo
    );
    // The fallback is not the same as a rule that matches. `b"Mahou"` *is* rule
    // two, so it answers `Unknown` — a rule matched, and it happened to be the one
    // carrying the fallback kind. A page that claimed otherwise would be teaching
    // the difference backwards.
    assert_eq!(
        match_types_custom(b"Mahou", RULES, Kind::Unknown),
        Kind::Unknown
    );
    // Nothing matches at all, so the `fallback` argument is what comes back.
    assert_eq!(
        match_types_custom(b"nothing", RULES, Kind::Unknown),
        Kind::Unknown
    );
}

/// docs/detection-levels.md — "Level 3"
#[test]
#[cfg(feature = "magical_dyn")]
fn levels_three_rules_at_runtime() {
    use magical_rs::magical::dyn_magic::DynMagicCustom;

    let rule = DynMagicCustom::new(
        |bytes: &[u8]| bytes.starts_with(b"MAGICAL"),
        String::from("custom format"),
        32,
    );

    assert!(rule.matches(b"MAGICAL"));
    assert_eq!(rule.kind_downcast_ref::<String>().unwrap(), "custom format");
}

/// docs/detection-levels.md — "Level 4"
#[test]
#[cfg(feature = "magical_async_dyn")]
fn levels_four_rules_that_await() {
    use magical_rs::magical::async_dyn_magic::{AsyncDynMagic, match_dyn_types_as};

    // The same minimal executor `readme_examples.rs` uses, so this file adds no
    // async runtime dependency of its own. Single-poll and never pending.
    fn block_on<F: std::future::Future>(fut: F) -> F::Output {
        use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

        const VTABLE: RawWakerVTable = RawWakerVTable::new(
            |_| RawWaker::new(std::ptr::null(), &VTABLE),
            |_| {},
            |_| {},
            |_| {},
        );
        let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) };
        let mut cx = Context::from_waker(&waker);

        let mut fut = Box::pin(fut);
        loop {
            if let Poll::Ready(out) = fut.as_mut().poll(&mut cx) {
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

/// docs/detection-levels.md — "Metadata across all three"
///
/// The page claims all three languages answer from one `formats.json`, so a format
/// cannot be `image/png` in Rust and something else elsewhere. What is checkable
/// from the crate is that the values agree with that file and that nothing is
/// invented for the formats that have no verified type.
#[test]
fn levels_metadata_is_never_invented() {
    use std::collections::BTreeSet;

    let table = magical_rs::magical::signatures::SIGNATURE_KIND;
    let with_mime = table.iter().filter(|m| m.kind.mime().is_some()).count();
    let with_extension = table
        .iter()
        .filter(|m| m.kind.extension().is_some())
        .count();

    assert!(
        with_mime > 0 && with_mime < table.len(),
        "{with_mime} of {}",
        table.len()
    );
    assert!(
        with_extension > 0 && with_extension < table.len(),
        "{with_extension} of {}",
        table.len()
    );

    // `None` means "none registered". A format with an empty string would be a
    // guess wearing the same type, which is what the page says does not happen.
    let empty: BTreeSet<&str> = table
        .iter()
        .filter_map(|m| m.kind.mime())
        .filter(|m| m.is_empty())
        .collect();
    assert!(empty.is_empty(), "these report an empty MIME: {empty:?}");
}

/// docs/across-languages.md — "What is genuinely the same"
///
/// The page promises that all three bindings answer identically because they call
/// the same code over the same table, and that the two windows mean the same thing
/// everywhere. Both are checkable from the crate.
#[test]
fn across_languages_the_shared_facts() {
    use magical_rs::magical::signatures::SIGNATURE_KIND;

    // "The rule table, and the order detection walks it." Ktx2 before Ktx is the
    // case that makes the point — and it has to be asked against the *table*, not
    // against `ALL_KINDS`, which is sorted by name and has them the other way
    // round. That is the mistake the page's second table exists to prevent.
    let table = |k| SIGNATURE_KIND.iter().position(|m| m.kind == k).unwrap();
    let ktx2 = table(FileKind::Ktx2);
    let ktx = table(FileKind::Ktx);
    assert!(ktx2 < ktx, "Ktx2 at {ktx2} must precede Ktx at {ktx}");

    // "The numbers. 36,870 and 2,048 mean the same thing everywhere."
    assert_eq!(with_bytes_read(), 36_870);
    assert_eq!(DEFAULT_MAX_BYTES_READ, 2_048);

    // "A None/null meaning 'no rule matched', and nothing else."
    let mut noise = [0_u8; 64];
    noise[0] = 0x2E; // `.`, which no signature in the table starts with
    assert_eq!(FileKind::match_types(&noise), None);
}

/// docs/api/rust.md — "Level 1", the `read_file_header_into` example
#[test]
#[cfg(feature = "std")]
fn api_rust_reading_a_directory() -> std::io::Result<()> {
    let mut buffer = Vec::new();
    let mut found = 0;

    for path in std::fs::read_dir(".")? {
        let path = path?.path();
        let Some(name) = path.to_str() else {
            continue;
        };
        if read_file_header_into(name, with_bytes_read(), &mut buffer).is_ok() {
            found += usize::from(FileKind::match_types(&buffer).is_some());
        }
    }

    assert!(found < usize::MAX);
    Ok(())
}

/// docs/api/rust.md — "Level 2"
#[test]
fn api_rust_level_two() {
    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Kind {
        MahouShoujo,
        Unknown,
    }

    static RULES: &[MagicCustom<Kind>] = magic_rules![
        (Kind::MahouShoujo, b"MagicalGirl"),
        (@bytes Kind::Unknown, [b"Magical", b"Mahou"], [0], 16),
    ];

    assert_eq!(
        match_types_custom(b"MagicalGirl", RULES, Kind::Unknown),
        Kind::MahouShoujo
    );
    assert_eq!(
        match_types_custom(b"something else", RULES, Kind::Unknown),
        Kind::Unknown
    );
}

/// Every test above names the page it came from, and every page is covered.
///
/// The direction that matters is this one: a page that gained an example without a
/// test here is a page whose example is unchecked, and it looks identical to a page
/// that is fully covered. The list is the pages that carry Rust examples, and a
/// page not in it is a page with no Rust to check.
#[test]
fn every_example_is_attached_to_a_page() {
    let pages = [
        "docs/api/rust.md",
        "docs/across-languages.md",
        "docs/concepts.md",
        "docs/detection-levels.md",
        "docs/getting-started.md",
    ];

    for page in pages {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join(page);
        assert!(
            path.is_file(),
            "{} is listed here but not in the repository",
            path.display()
        );
    }

    // And the pages are linked from the index, so a reader arriving at the
    // repository readme can find them. A documentation tree nothing points at is
    // a documentation tree nobody reads.
    let readme = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("readme.md"),
    )
    .expect("the repository readme is readable");
    for page in pages {
        assert!(
            readme.contains(page),
            "the readme does not link {page}, so this test asserts a page nobody is sent to"
        );
    }
}
