//! `Detect` answers the same questions as the ungated API, and the feature leaves
//! nothing behind when it is off.
//!
//! # Two halves, for two different failures
//!
//! The first half is behaviour: every method here is checked against the function
//! it wraps, on the same inputs, and the two must agree. A `Detect` that disagreed
//! with `match_types` would be a second detector to keep correct, which is the
//! opposite of what sugar is for.
//!
//! The second half is the feature flag itself. A `#[cfg]` on the module body but
//! not on the `pub mod` line compiles perfectly with the feature off and still
//! shows a module in the documentation that is empty. That is a failure nothing in
//! the build would report, so it is reported here instead.
//!
//! # Why the halves are gated differently
//!
//! The behavioural tests need the methods, so they need the feature. The flag
//! tests need the feature's *absence*, and would be checking the wrong thing if
//! they only ran when it was present — so they are ungated and run in the ordinary
//! `cargo test`, which is the build where the feature is off.

use std::path::Path;

/// The repository root, found by walking up from this crate.
///
/// Walking rather than counting `..`, for the reason `tests/packaging.rs` and
/// `tests/lint_level.rs` both give.
fn repo_root() -> std::path::PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .ancestors()
        .find(|dir| dir.join("Makefile").is_file())
        .unwrap_or_else(|| {
            panic!(
                "no ancestor of {} holds a Makefile, so the repository root cannot be located",
                manifest.display()
            )
        })
        .to_path_buf()
}

/// A PNG header, the table's first entry.
#[cfg(feature = "magical_fluent")]
const PNG: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

/// `detect` and `match_types` agree on every one of the table's own fixtures.
///
/// The comparison is against the ungated API rather than against a list of
/// expected answers, because the contract is that they are the same function
/// spelled differently — not that both happen to be right about PNG.
#[test]
#[cfg(feature = "magical_fluent")]
fn detect_agrees_with_match_types_on_every_fixture() {
    use magical_rs::magical::fluent::Detect;
    use magical_rs::magical::magic::FileKind;
    use magical_rs::magical::signatures::SIGNATURE_KIND;

    let mut checked = 0;
    for (position, magic) in SIGNATURE_KIND.iter().enumerate() {
        for signature in magic.signatures {
            let mut buffer = vec![0_u8; signature.len() + 32];
            buffer[..signature.len()].copy_from_slice(signature);
            assert_eq!(
                buffer.detect(),
                FileKind::match_types(&buffer),
                "entry {position} ({:?})",
                magic.kind
            );
            checked += 1;
        }
    }
    assert!(checked > 100, "only {checked} fixtures were compared");
}

/// `detect_within` and the dispatch index agree, on every window worth trying.
///
/// The window is a filter rather than a length, so the interesting values are the
/// ones either side of a rule's own `max_bytes_read`.
#[test]
#[cfg(feature = "magical_fluent")]
fn detect_within_agrees_with_the_index_on_every_window() {
    use magical_rs::magical::dispatch;
    use magical_rs::magical::fluent::Detect;
    use magical_rs::magical::signatures::SIGNATURE_KIND;

    // A `Vec` rather than a 40,000-byte array: clippy's `large_stack_arrays` is
    // right that this does not belong on the stack, and the point of the fixture
    // is the offset it puts a signature at, not where the bytes live.
    let mut iso = vec![0_u8; 40_000];
    iso[32_769..32_774].copy_from_slice(b"CD001");

    for bytes in [PNG, b"GIF89a", iso.as_slice(), &[], &[0x2E; 2_048]] {
        for window in [usize::MAX, 0, 8, 64, 2_048, 36_870] {
            assert_eq!(
                bytes.detect_within(window),
                dispatch::first_match(bytes, window).map(|i| SIGNATURE_KIND[i].kind),
                "{} bytes, window {window}",
                bytes.len()
            );
        }
    }
}

/// `is` asks about one format's own rule, which is a different question from
/// `detect`.
///
/// Asserted as a difference and not only as agreement, because that is the whole
/// reason the method exists: a file two formats both claim is where the two
/// answers part company, and a test that only checked they matched would never
/// find the difference.
#[test]
#[cfg(feature = "magical_fluent")]
fn is_is_not_detect() {
    use magical_rs::magical::fluent::Detect;
    use magical_rs::magical::magic::FileKind;

    let svg = b"<?xml version=\"1.0\"?><svg></svg>";

    let detected = svg.detect();
    assert!(
        detected.is_some(),
        "the fixture is not recognised at all, so the test proves nothing",
    );
    assert!(
        svg.is(FileKind::XML),
        "an XML document must be reported as XML by its own rule, whatever the table's \
         order decides for `detect`",
    );

    if let Some(found) = detected {
        assert!(
            svg.is(found),
            "detect returned {found:?} but is() denies it"
        );
    }
    assert!(
        !svg.is(FileKind::PkgZip),
        "an XML document is not a ZIP by either question"
    );
}

/// `is_any` is `is` over an iterator, and it takes an iterator.
#[test]
#[cfg(feature = "magical_fluent")]
fn is_any_is_a_loop_over_is() {
    use magical_rs::magical::fluent::Detect;
    use magical_rs::magical::magic::FileKind;

    assert!(PNG.is_any([FileKind::PkgZip, FileKind::GIF, FileKind::Png]));
    assert!(!PNG.is_any([FileKind::PkgZip, FileKind::GIF]));
    assert!(!PNG.is_any([]), "an empty set of formats contains nothing");

    // An iterator rather than a slice: this is the shape that lets a caller pass
    // a generated set without collecting one first.
    assert!(PNG.is_any((0..2).map(|i| if i == 0 {
        FileKind::Png
    } else {
        FileKind::PkgZip
    })));
}

/// `FileKind` has no "unknown" arm, which is why there is no `detect_or`.
///
/// The method was removed rather than corrected. A by-value fallback is evaluated
/// before the call whatever the callee does with it, so a method documented as
/// lazy was not, and a caller paying for an expensive fallback on every call had
/// no way to see that from the signature. It was also a spelling of
/// `self.detect().unwrap_or(fallback)`, which saves nothing.
///
/// This asserts the reason it is not worth a wrapper. Every variant is a real
/// format — that is what `detect()` returning `Option` is for, and an `Unknown`
/// arm would mean a caller had to remember that it means "I gave up" rather than
/// "the file is unknown".
#[test]
#[cfg(feature = "magical_fluent")]
fn every_file_kind_is_a_real_format_so_there_is_no_fallback_to_offer() {
    use magical_rs::magical::kinds_meta::ALL_KINDS;
    use magical_rs::magical::magic::FileKind;

    for kind in ALL_KINDS {
        let name = kind.display_name();
        assert!(
            !name.is_empty(),
            "{kind:?} has an empty display name, so it is standing in for something rather \
             than naming a format",
        );
        assert!(
            !kind.variant_name().is_empty(),
            "{kind:?} has an empty variant name",
        );
    }

    // And the count is the table's, so adding a format cannot quietly add a
    // variant that means something other than a format.
    assert_eq!(ALL_KINDS.len(), 114);
    assert_eq!(FileKind::match_types(PNG), Some(FileKind::Png));
}

/// The level 2 half reads the same way and answers the same way.
#[test]
#[cfg(feature = "magical_fluent")]
fn detect_in_agrees_with_match_types_custom() {
    use magical_rs::magic_rules;
    use magical_rs::magical::fluent::DetectRules;
    use magical_rs::magical::magic_custom::{MagicCustom, match_types_custom};

    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Kind {
        Shoujo,
        Other,
        Unknown,
    }

    fn is_other(bytes: &[u8]) -> bool {
        bytes.starts_with(b"Other")
    }

    static RULES: &[MagicCustom<Kind>] = magic_rules![
        (Kind::Shoujo, b"Shoujo"),
        (Kind::Other, via is_other),
    ];

    for bytes in [b"ShoujoFile".as_slice(), b"OtherFile", b"nope", &[]] {
        assert_eq!(
            bytes.detect_in(RULES, Kind::Unknown),
            match_types_custom(bytes, RULES, Kind::Unknown),
            "{} bytes",
            bytes.len()
        );
    }

    assert_eq!(b"ShoujoFile".detect_in(RULES, Kind::Unknown), Kind::Shoujo);
    assert_eq!(b"nope".detect_in(RULES, Kind::Unknown), Kind::Unknown);
}

// ---------------------------------------------------------------------------
// The feature flag itself.
//
// Ungated on purpose: these are about the feature being *off*, and the ordinary
// `cargo test` is the build where it is.
// ---------------------------------------------------------------------------

/// The crate root, for the checks below.
const CRATE_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs");

/// `fluent` appears in `lib.rs` only on a line gated on the feature.
///
/// The other half of "off means nothing" is that the module does not exist, which
/// this cannot see — `make test-nostd` builds the crate with the feature off, and
/// a stray `use` of it anywhere else would fail that build. What this sees is
/// whether the declaration is gated at all, which is the difference between "there
/// is no module when the feature is off" and "there is a module that is empty".
#[test]
fn the_fluent_module_is_gated_on_its_own_feature() {
    let text = std::fs::read_to_string(CRATE_ROOT).expect("the crate root is readable");

    let lines: Vec<&str> = text.lines().collect();
    let at: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| {
            let code = line.trim_start();
            !code.starts_with("//") && code.contains("pub mod fluent")
        })
        .map(|(i, _)| i)
        .collect();

    assert_eq!(
        at.len(),
        1,
        "expected exactly one `pub mod fluent` in lib.rs, found {at:?}",
    );

    // The `cfg` is on the line *above* the declaration, which is where an attribute
    // gating a `mod` has to be — so this reads the previous line rather than
    // looking for both on one. A `#[cfg]` on the same line as `pub mod` would be a
    // syntax error, so a test that looked for it there would pass vacuously.
    let declaration = at[0];
    let attribute = declaration
        .checked_sub(1)
        .map(|i| lines[i].trim())
        .unwrap_or_default();

    assert!(
        attribute.starts_with("#[cfg("),
        "`pub mod fluent` is not behind a cfg; the line above it is {attribute:?}",
    );
    assert!(
        attribute.contains("feature = \"magical_fluent\""),
        "`pub mod fluent` is behind the wrong cfg: {attribute}",
    );
}

/// Nothing outside the gated module mentions it.
///
/// A `use magical_rs::magical::fluent::...` in another module would make the crate
/// need the feature to build at all, which is the opposite of what the feature is
/// for. Only this test and the doc examples are allowed to name it.
#[test]
fn no_other_module_depends_on_the_feature() {
    let root = repo_root().join("crates/magical_rs/src");
    let mut problems = Vec::new();

    for path in walk(&root) {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if name == "lib.rs" || name == "fluent.rs" {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("a source file is readable");
        for (number, line) in text.lines().enumerate() {
            // A comment naming the module is not a dependency on it, and lib.rs's
            // own comment explains why the `cfg` is where it is. Only code counts.
            let code = line.trim_start();
            if (code.starts_with("//") || code.starts_with("///")) || !code.contains("fluent") {
                continue;
            }
            problems.push(format!(
                "{}:{}: {line}",
                path.strip_prefix(&root).unwrap_or(&path).display(),
                number + 1
            ));
        }
    }

    assert!(
        problems.is_empty(),
        "these lines mention the gated module outside it:\n  {}",
        problems.join("\n  "),
    );
}

/// Every `.rs` under `src`, recursively.
///
/// Hand-rolled rather than a dependency: the crate has none, and this is a test.
fn walk(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(walk(&path));
        } else if path.extension().is_some_and(|e| e == "rs") {
            found.push(path);
        }
    }
    found
}
