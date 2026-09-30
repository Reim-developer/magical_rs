//! The generated metadata has to describe the formats the table can actually
//! return.
//!
//! `magical::kinds_meta` is written by `scripts/gen_kinds.ps1` from
//! `formats.json` and checked in, which is the same arrangement the Python
//! binding's `_kinds.py` uses and the same hazard: a generated file nobody can
//! diff is a generated file nobody can review, and a format added to
//! `SIGNATURE_KIND` without a row in the dataset produces an accessor with no arm
//! -- which is a compile error, and only if the new variant is reached by an
//! exhaustive `match`. It is, so that case is covered.
//!
//! The case that is *not* covered by the compiler is the other direction. The
//! generator's own check is against the dataset's declared count, so a format
//! with a dataset row and no `SIGNATURE_KIND` entry fails there, but that check
//! does not know what `SIGNATURE_KIND` contains. This one does, and
//! `tests/dataset.rs` covers the third side: the dataset against the generated
//! file, so a hand-edit to this one is caught as well.

use std::collections::BTreeSet;

use magical_rs::magical::kinds_meta::ALL_KINDS;
use magical_rs::magical::magic::FileKind;
use magical_rs::magical::signatures::SIGNATURE_KIND;

/// Distinct kinds the detection table can return.
fn detectable() -> BTreeSet<FileKind> {
    SIGNATURE_KIND.iter().map(|entry| entry.kind).collect()
}

/// `ALL_KINDS` is exactly the set of kinds the table produces, in both
/// directions.
#[test]
fn all_kinds_is_exactly_what_the_table_detects() {
    let listed: BTreeSet<FileKind> = ALL_KINDS.iter().copied().collect();
    let detectable = detectable();

    // A repeat in `ALL_KINDS` would make `len` disagree with the set, which is
    // how a format gets listed twice in a catalogue and a lookup finds the
    // first of them.
    assert_eq!(
        listed.len(),
        ALL_KINDS.len(),
        "ALL_KINDS lists the same format more than once",
    );
    assert_eq!(
        ALL_KINDS.len(),
        114,
        "the format count is pinned at 114 in readme.md and in every binding's tests",
    );

    let missing: Vec<&FileKind> = detectable.difference(&listed).collect();
    assert!(
        missing.is_empty(),
        "{} format(s) the detection table returns are missing from ALL_KINDS: {}. Add them to \
         formats.json and re-run `scripts/gen_kinds.ps1`.",
        missing.len(),
        missing
            .iter()
            .map(|k| k.variant_name())
            .collect::<Vec<_>>()
            .join(", "),
    );

    let extra: Vec<&FileKind> = listed.difference(&detectable).collect();
    assert!(
        extra.is_empty(),
        "{} format(s) in ALL_KINDS are not in the detection table: {}. A format that cannot be \
         detected is a promise nothing keeps.",
        extra.len(),
        extra
            .iter()
            .map(|k| k.variant_name())
            .collect::<Vec<_>>()
            .join(", "),
    );
}

/// Every format round-trips through its own name.
///
/// A `from_name` arm that names the wrong variant, or a `variant_name` arm that
/// disagrees with the declaration, both make a lookup that "works" and answers
/// with a different format. Neither is visible without this.
#[test]
fn every_format_round_trips_through_its_variant_name() {
    for &kind in &ALL_KINDS {
        let name = kind.variant_name();
        assert_eq!(
            FileKind::from_name(name),
            Some(kind),
            "{name} does not look itself up",
        );
    }
}

/// Names are exact, and an unknown name is a `None` rather than a guess.
#[test]
fn from_name_is_case_sensitive_and_answers_none_for_nonsense() {
    assert_eq!(FileKind::from_name("Png"), Some(FileKind::Png));
    // Case matters, on purpose: `Png` is the identifier this crate uses and
    // `png` is not. A loose lookup belongs in a command line parser.
    assert_eq!(FileKind::from_name("png"), None);
    assert_eq!(FileKind::from_name("PNG"), None);
    assert_eq!(FileKind::from_name("Png "), None);
    assert_eq!(FileKind::from_name(""), None);
    assert_eq!(FileKind::from_name("Nonsense"), None);
}

/// Every format has a display name, and it is not the variant name in every
/// case.
///
/// The second half is the interesting one: `Png` and `PNG` are the same format
/// and a metadata table that made them identical would mean the readme's own
/// "Format" column was never checked against the enum.
#[test]
fn every_format_has_a_display_name_and_at_least_one_differs() {
    for &kind in &ALL_KINDS {
        let display = kind.display_name();
        assert!(
            !display.is_empty(),
            "{} has an empty display name",
            kind.variant_name()
        );
    }
    assert_eq!(FileKind::Png.display_name(), "PNG");
    assert_eq!(FileKind::Jpg.display_name(), "JPEG");
    assert_ne!(
        FileKind::Png.variant_name(),
        FileKind::Png.display_name(),
        "if every display name equalled its variant name, the readme's Format column would be \
         unchecked",
    );
}

/// A MIME type looks like a MIME type, and an extension looks like an extension.
///
/// The checks are on shape rather than on which value is right, because a wrong
/// value is a documentation bug that no test can detect -- there is no authority
/// to compare against inside a test. What can be checked is that the field is
/// not something else entirely: a MIME type with a leading dot, an extension
/// with a slash, an empty string in place of "no value".
#[test]
fn mime_and_extension_look_like_what_they_are() {
    for &kind in &ALL_KINDS {
        let name = kind.variant_name();

        if let Some(mime) = kind.mime() {
            assert!(!mime.is_empty(), "{name} has an empty MIME type");
            assert!(
                !mime.starts_with('.'),
                "{name} has MIME type {mime:?}, which starts with a dot",
            );
            assert!(
                mime.contains('/'),
                "{name} has MIME type {mime:?}, which has no `type/subtype` separator",
            );
            assert_eq!(
                mime,
                mime.trim(),
                "{name} has MIME type {mime:?} with surrounding whitespace",
            );
        }

        if let Some(extension) = kind.extension() {
            assert!(!extension.is_empty(), "{name} has an empty extension");
            assert!(
                !extension.starts_with('.'),
                "{name} has extension {extension:?}, which carries a leading dot",
            );
            assert!(
                !extension.contains('/'),
                "{name} has extension {extension:?}, which carries a path separator",
            );
            assert!(
                extension
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'+' || b == b'_'),
                "{name} has extension {extension:?}, which is not a bare extension",
            );
        }
    }
}

/// Two formats do not claim the same extension, with the one real exception.
///
/// A shared extension is a bug in this table: the extension is what a caller
/// writes a file with, and two formats answering the same one means the caller's
/// `.sit` is ambiguous in a way the detection table -- which reads bytes, not
/// names -- cannot resolve. The MIME types are deliberately *not* checked the
/// same way, because sharing a MIME type is legitimate and common: a container
/// format and the media inside it answer the same type on purpose.
///
/// The exception is listed rather than the rule being weakened, so a second
/// duplicate fails this test and the list grows by one line with a reason.
#[test]
fn extensions_are_unique() {
    /// Extensions two formats both claim, and why.
    const SHARED: &[(&str, &[&str])] = &[
        // `Stuffit` is a `.sit` archive and `StuffitSit` is a self-extracting one.
        // They are distinct formats -- different magic -- and the convention
        // gives both the same suffix, so the duplicate is in the world and not
        // in the table.
        ("sit", &["Stuffit", "StuffitSit"]),
    ];

    let mut owner: std::collections::BTreeMap<&str, Vec<&str>> = std::collections::BTreeMap::new();
    for &kind in &ALL_KINDS {
        if let Some(extension) = kind.extension() {
            owner
                .entry(extension)
                .or_default()
                .push(kind.variant_name());
        }
    }

    for (extension, claimants) in &owner {
        if claimants.len() < 2 {
            continue;
        }
        // Sorted on both sides, because the order carries no meaning: the
        // claimants arrive in `ALL_KINDS` order, which is sorted by the short
        // name the Python binding uses as its enum value, and `StuffitSit` has a
        // different one from `Stuffit`. Asserting on order would be asserting
        // on a sort key that has nothing to do with this test.
        let mut actual: Vec<&str> = claimants.clone();
        actual.sort_unstable();
        let mut expected: Vec<&str> = SHARED
            .iter()
            .find(|(ext, _)| ext == extension)
            .map(|(_, names)| names.to_vec())
            .unwrap_or_default();
        expected.sort_unstable();
        assert_eq!(
            actual, expected,
            "extension {extension:?} is claimed by more formats than the table records. Two \
             formats writing the same suffix is ambiguous for anything that has to name the file, \
             so either the metadata is wrong or the pair belongs in the exception list above with \
             a reason.",
        );
    }

    // And the exception list itself is not stale: an entry naming formats that
    // no longer share the extension is a line nobody re-reads.
    for (extension, names) in SHARED {
        let mut expected: Vec<&str> = names.to_vec();
        expected.sort_unstable();
        let actual = owner.get(extension).map(|claims| {
            let mut claims: Vec<&str> = claims.clone();
            claims.sort_unstable();
            claims
        });
        assert_eq!(
            actual.as_deref(),
            Some(expected.as_slice()),
            "the exception list claims {names:?} share {extension:?}, and the table says otherwise",
        );
    }
}

/// The generated file is written without a byte-order mark.
///
/// `gen_kinds.ps1` writes with `UTF8Encoding($false)` on purpose. PowerShell's
/// own `Set-Content -Encoding utf8` prepends one, and adding it back would be a
/// silent generator regression: rustc accepts a BOM, the file diffs as changed
/// in a way nobody can see, and the first byte of the first line of a generated
/// source file is a reasonable thing to assert. The Python side asserts the same
/// thing about `_kinds.py`, which ships inside a wheel.
#[test]
fn the_generated_file_has_no_byte_order_mark() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("magical")
        .join("kinds_meta.rs");
    let bytes =
        std::fs::read(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));

    assert!(
        !bytes.starts_with(&[0xEF, 0xBB, 0xBF]),
        "{} starts with a UTF-8 BOM; gen_kinds.ps1 must write it without one",
        path.display(),
    );
}

/// The specific values this project is confident about.
///
/// Not a substitute for the generator's cross-check against the readme; this
/// is the handful that a reader would notice being wrong, and each one is a
/// name a person types rather than a value derived from bytes.
#[test]
fn the_values_anyone_would_type_are_right() {
    assert_eq!(FileKind::Png.mime(), Some("image/png"));
    assert_eq!(FileKind::Jpg.mime(), Some("image/jpeg"));
    assert_eq!(FileKind::GIF.mime(), Some("image/gif"));
    assert_eq!(FileKind::PDF.mime(), Some("application/pdf"));
    assert_eq!(FileKind::WASM.mime(), Some("application/wasm"));
    assert_eq!(FileKind::PkgZip.mime(), Some("application/zip"));
    assert_eq!(FileKind::ISO.mime(), Some("application/x-iso9660-image"));
    // Formats with no registered value say so rather than guessing. `ELF` has
    // an IANA registration now and `zlib` does not, but a format this table has
    // no verified value for must not acquire one by accident.
    assert_eq!(FileKind::ELF.mime(), None);
    assert_eq!(FileKind::Zlib.mime(), None);
    assert_eq!(FileKind::Zlib.extension(), None);

    // A shebang is text, and reporting it as such is the whole reason the
    // extension exists.
    assert_eq!(FileKind::ScriptExecute.display_name(), "Script / shebang");
}
