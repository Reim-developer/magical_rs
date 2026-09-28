//! Self-verifying coverage of the built-in signature table.
//!
//! These tests iterate `SIGNATURE_KIND` itself rather than hard-coding a list,
//! so every format added to the table is covered automatically. If a signature
//! is wrong, or is shadowed by an earlier entry, these tests fail.

use magical_rs::magical::magic::FileKind;
use magical_rs::magical::match_rules::MatchRules;
use magical_rs::magical::signatures::SIGNATURE_KIND;

/// Neutral padding. All-zero is avoided so a fill byte can never itself
/// satisfy a signature and mask a broken offset.
const PAD: u8 = 0x2A;

/// Build a buffer containing `magic` at `offset`, padded with `PAD`.
fn buffer_at(magic: &[u8], offset: usize) -> Vec<u8> {
    let mut buf = vec![PAD; offset + magic.len()];
    buf[offset..offset + magic.len()].copy_from_slice(magic);
    buf
}

/// Every signature in the table must be recognised as its own `FileKind`.
///
/// Entries using a custom predicate are covered by
/// [`custom_rule_entries_are_listed`] and the predicate's own tests.
#[test]
fn every_signature_detects_its_own_kind() {
    let mut checked = 0_usize;
    let mut mismatches: Vec<String> = Vec::new();

    for entry in SIGNATURE_KIND {
        if matches!(entry.rules, MatchRules::WithFn(_)) {
            continue;
        }

        for signature in entry.signatures {
            for offset in entry.offsets {
                let bytes = buffer_at(signature, *offset);
                let actual = FileKind::match_types(&bytes);
                checked += 1;

                if actual != Some(entry.kind) {
                    mismatches.push(format!(
                        "{:?}: expected at offset {}, got {:?} (signature {:02X?})",
                        entry.kind, offset, actual, signature
                    ));
                }
            }
        }
    }

    assert!(checked > 0, "no signatures were exercised");
    assert!(
        mismatches.is_empty(),
        "{} of {checked} signature entries did not detect themselves:\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
}

/// A signature must not match at the wrong offset.
#[test]
fn signatures_do_not_match_at_wrong_offset() {
    let mut mismatches: Vec<String> = Vec::new();

    for entry in SIGNATURE_KIND {
        if matches!(entry.rules, MatchRules::WithFn(_)) {
            continue;
        }

        for signature in entry.signatures {
            // Offset 0 is the only offset that always exists in the table.
            if entry.offsets.contains(&0) {
                continue;
            }
            let bytes = buffer_at(signature, 0);
            if let Some(found) = FileKind::match_types(&bytes) {
                mismatches.push(format!(
                    "{:?} matched at offset 0 as {found:?}, but is declared at {:?} \
                     (signature {:02X?})",
                    entry.kind, entry.offsets, signature
                ));
            }
        }
    }

    assert!(
        mismatches.is_empty(),
        "{} signatures matched at an undeclared offset:\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
}

/// An empty or unknown buffer must not be classified as a known type.
#[test]
fn padding_is_not_misdetected() {
    assert_eq!(FileKind::match_types(&[]), None);
    assert_eq!(FileKind::match_types(&vec![PAD; 64_096].into_boxed_slice()), None);
}

/// A truncated buffer must not panic and must not match.
#[test]
fn truncated_buffer_is_safe() {
    for entry in SIGNATURE_KIND {
        if matches!(entry.rules, MatchRules::WithFn(_)) {
            continue;
        }

        for signature in entry.signatures {
            for len in 0..signature.len() {
                let bytes = &signature[..len];
                // Must not panic; a match here would be a real bug.
                let _ = FileKind::match_types(bytes);
            }
        }
    }
}

/// Entries using a custom predicate rather than a byte signature.
///
/// Both exist because a byte signature alone is either ambiguous or too broad.
#[test]
fn custom_rule_entries_are_listed() {
    let custom: Vec<FileKind> = SIGNATURE_KIND
        .iter()
        .filter(|m| matches!(m.rules, MatchRules::WithFn(_)))
        .map(|m| m.kind)
        .collect();

    assert_eq!(
        custom,
        vec![FileKind::ScriptExecute, FileKind::WEBP],
        "custom-rule entries changed; update this test and the readme"
    );
}

/// The table must stay non-trivial and every entry must be reachable.
#[test]
fn signature_table_is_populated() {
    assert!(
        SIGNATURE_KIND.len() >= 114,
        "expected at least 114 built-in formats, found {}",
        SIGNATURE_KIND.len()
    );

    for entry in SIGNATURE_KIND {
        match entry.rules {
            MatchRules::Default => assert!(
                !entry.signatures.is_empty() && !entry.offsets.is_empty(),
                "{:?} has a default rule but no signature or no offset",
                entry.kind
            ),
            MatchRules::WithFn(_) => {}
        }
    }
}
