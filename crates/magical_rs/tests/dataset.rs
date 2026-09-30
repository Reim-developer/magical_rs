//! `formats.json` has to describe the crate's formats, not some other project's.
//!
//! The dataset at the repository root is the one place a format's name, MIME type
//! and extension are written down. `scripts/gen_kinds.ps1` generates
//! `kinds_meta.rs` and the Python enum from it, and `scripts/gen_formats.mjs`
//! generates the JavaScript tables, so a mistake in the dataset becomes a
//! mistake in three bindings at once and in all three it looks deliberate.
//!
//! Nothing in the library *reads* the dataset -- the crate has no `serde`, and
//! adding one to parse a build-time file would be a worse trade than checking the
//! answers the generated file already holds. So this test reads it with a parser
//! of its own, on purpose: a `serde_json` dependency here would let a schema
//! change in the file and a matching change in this test pass together, which is
//! the one thing this test exists to prevent.
//!
//! The checks are deliberately coarse. Parsing JSON here is already more
//! machinery than the job deserves, so this reads the few fields it needs with
//! string searching and refuses to proceed if the shape is not what it expects. A
//! shape it does not recognise is a failure, not a skip: a silently-skipped row is
//! a format that quietly stops being checked.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use magical_rs::magical::kinds_meta::ALL_KINDS;
use magical_rs::magical::magic::FileKind;
use magical_rs::magical::signatures::SIGNATURE_KIND;

/// The repository root, found by walking up from this crate.
///
/// Walking rather than counting `..` for the reason `tests/packaging.rs` gives: a
/// depth that is correct today is a second thing to update when the layout
/// changes, and the failure is a panic in a test whose name says nothing about
/// paths.
fn repo_root() -> PathBuf {
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

/// One row of the dataset.
///
/// `mime` and `extension` are `Option` because `null` is a real answer meaning
/// "not registered", and the distinction between that and a key that is missing
/// entirely is checked when the row is read rather than inferred from the value.
struct Format {
    variant: String,
    abi_order: Option<usize>,
    token: String,
    name: String,
    mime: Option<String>,
    extension: Option<String>,
}

/// One row's `"key": value` pairs, with both kept as text.
///
/// Owned rather than borrowed so the scanner can slice the file and hand back what
/// it found without leaking: the rows outlive the scanner, and a row's keys are
/// compared with `==` against a `&str` rather than used as a `HashMap` key, which
/// for six well-known names is cheaper than building a map per row.
type Row = Vec<(String, String)>;

/// The value a row gives for `key`, or `None` if the row has no such key.
fn value<'a>(row: &'a Row, key: &str) -> Option<&'a str> {
    row.iter()
        .find(|(name, _)| name == key)
        .map(|(_, value)| value.as_str())
}

/// The `formats` array, split into its rows.
///
/// A scanner rather than `serde_json`, and the only thing it has to get right is
/// strings: `RichTextFormat`'s documented signature is `7B 5C 72 74 66 31 ({\rtf1)`
/// and `SQLite`'s contains `\0`, so a reader that counts braces or stops at a
/// quote without noticing an escape reads that row as ending early and every
/// assertion after it as being about the wrong format. Not hypothetical -- it is
/// what the first version of this file did, and it failed on `RichTextFormat`.
///
/// So: one pass, a handful of pieces of state, and no attempt to be a JSON parser.
/// A key this does not recognise is a pair it records and nothing reads; a value
/// shape it does not recognise -- `abi_order` becoming an object, say -- is a row
/// whose `abi_order` will not parse, which is a failure with a message rather than
/// a skip.
fn rows(text: &str) -> Vec<Row> {
    const OPEN: &str = "\"formats\": [";
    let at = text
        .find(OPEN)
        .unwrap_or_else(|| panic!("formats.json has no `{OPEN}` array"))
        + OPEN.len();
    let body = &text[at..];
    let bytes = body.as_bytes();

    let mut out: Vec<Row> = Vec::new();
    let mut row: Row = Vec::new();
    // Brace depth inside the current row. `[` is not counted: the array being
    // walked is the only one there is, and a nested array inside a row would show
    // up as a row that will not parse rather than as silently wrong offsets.
    let mut depth = 0_i32;
    // A string being read: where its content starts, and whether it is a key.
    let mut string: Option<(usize, bool)> = None;
    let mut escaped = false;
    let mut key: Option<String> = None;
    // Where a value begins, once its key's colon has been seen.
    let mut value_at = 0_usize;
    let mut want_value = false;

    let mut at = 0_usize;
    while at < bytes.len() {
        let c = bytes[at];

        if let Some((start, is_key)) = string {
            if escaped {
                escaped = false;
            } else if c == b'\\' {
                escaped = true;
            } else if c == b'"' {
                let content = &body[start..at];
                string = None;
                if is_key {
                    key = Some(content.to_owned());
                } else {
                    let name = key
                        .take()
                        .expect("a value's key is recorded before its value");
                    row.push((name, content.to_owned()));
                    want_value = false;
                }
            }
            at += 1;
            continue;
        }

        match c {
            b'"' => string = Some((at + 1, !want_value)),
            b'{' => {
                depth += 1;
                row.clear();
                // A row's state starts empty. Resetting here rather than at the
                // closing brace is what stops a trailing `"group": null` -- whose
                // value is not a string, so nothing else resets it -- from leaving
                // the next row's first key looking like the tail of a pair.
                key = None;
                want_value = false;
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    out.push(std::mem::take(&mut row));
                }
            }
            // The array's own closing bracket: at depth 0, nothing is open.
            b']' if depth == 0 => break,
            b':' if depth == 1 => {
                want_value = true;
                value_at = at + 1;
            }
            b',' if depth == 1 => {
                want_value = false;
                key = None;
            }
            // An unquoted value: `null`, or the one number in the file.
            // `abi_order` is the number, and it is a position rather than a name --
            // quoting it would let `7` and `"7"` both be written for one
            // discriminant, which is the problem this file exists to catch.
            //
            // Kept as raw text for the caller to interpret rather than parsed here,
            // so `null` costs nothing and a shape this does not recognise becomes a
            // value the caller rejects with a message about the field.
            _ if want_value && !c.is_ascii_whitespace() => {
                let end = body[value_at..]
                    .find([',', '\n', '}'])
                    .map_or(bytes.len(), |offset| value_at + offset);
                let name = key
                    .take()
                    .expect("a value's key is recorded before its value");
                row.push((name, body[value_at..end].trim().to_owned()));
                want_value = false;
                at = end;
                continue;
            }
            _ => {}
        }
        at += 1;
    }
    out
}
/// Every row of the dataset, in the order the file lists them.
fn formats() -> Vec<Format> {
    let text = std::fs::read_to_string(repo_root().join("formats.json"))
        .expect("formats.json is at the repository root and is readable");

    rows(&text)
        .iter()
        .map(|row| {
            let text_of = |key: &str| {
                value(row, key)
                    .unwrap_or_else(|| panic!("a formats.json row has no `{key}`: {row:?}"))
                    .to_owned()
            };
            // `null` is a real answer meaning "not registered", and it is a
            // different mistake from the key being absent -- so the key has to be
            // there and the value has to be the four letters.
            let optional_of = |key: &str| match value(row, key) {
                Some("null") => None,
                Some(found) => Some(found.to_owned()),
                None => panic!("a formats.json row has no `{key}` key; use null for none: {row:?}"),
            };

            Format {
                variant: text_of("variant"),
                abi_order: Some(
                    text_of("abi_order")
                        .parse()
                        .expect("`abi_order` is a whole number"),
                ),
                token: text_of("token"),
                name: text_of("name"),
                mime: optional_of("mime"),
                extension: optional_of("extension"),
            }
        })
        .collect()
}

/// The dataset names exactly the formats the detection table returns.
///
/// Both directions, because either one alone is satisfiable by a table of the
/// right length: a dataset with a typo in one variant is missing a real format,
/// and one naming a format that was removed is promising one that cannot arrive.
#[test]
fn the_dataset_lists_exactly_what_the_table_detects() {
    let rows = formats();
    assert_eq!(
        rows.len(),
        114,
        "formats.json has {} rows, expected 114. The count is pinned in every generated file and \
         in every binding's tests, so changing it is a deliberate edit.",
        rows.len(),
    );

    let listed: BTreeSet<&str> = rows.iter().map(|row| row.variant.as_str()).collect();
    assert_eq!(
        listed.len(),
        rows.len(),
        "formats.json names the same variant more than once",
    );

    let detectable: BTreeSet<FileKind> = SIGNATURE_KIND.iter().map(|entry| entry.kind).collect();

    let missing: Vec<&str> = detectable
        .iter()
        .filter(|kind| !listed.contains(kind.variant_name()))
        .map(|kind| kind.variant_name())
        .collect();
    assert!(
        missing.is_empty(),
        "{} format(s) the detection table returns are absent from formats.json: {}",
        missing.len(),
        missing.join(", "),
    );

    let extra: Vec<&str> = listed
        .iter()
        .filter(|variant| FileKind::from_name(variant).is_none())
        .copied()
        .collect();
    assert!(
        extra.is_empty(),
        "{} format(s) in formats.json are not in the enum: {}. A format that cannot be detected is \
         a promise nothing keeps.",
        extra.len(),
        extra.join(", "),
    );

    assert_eq!(
        listed.len(),
        ALL_KINDS.len(),
        "ALL_KINDS and formats.json disagree on the count, so one of them was generated from \
         something other than the other",
    );
}

/// Every value the dataset holds is the value the generated Rust answers.
///
/// This is the test that justifies the dataset. `kinds_meta.rs` is *generated from*
/// `formats.json`, so comparing the two is comparing a file with its own output --
/// which is exactly the point: it catches a hand-edit to the generated file, which
/// is otherwise invisible, because the file compiles and every other test in this
/// suite reads it rather than comparing it to anything.
#[test]
fn the_generated_metadata_is_the_dataset() {
    let rows = formats();
    for row in &rows {
        let kind = FileKind::from_name(&row.variant).unwrap_or_else(|| {
            panic!("formats.json names {}, which is not a variant", row.variant)
        });
        let name = row.variant.as_str();

        assert_eq!(kind.display_name(), row.name, "{name}'s display name");
        assert_eq!(
            kind.mime().map(ToOwned::to_owned),
            row.mime,
            "{name}'s MIME type"
        );
        assert_eq!(
            kind.extension().map(ToOwned::to_owned),
            row.extension,
            "{name}'s extension",
        );
    }
}

/// The positions are a permutation of `0..114`.
///
/// `abi_order` is the JavaScript side's ABI: a kind crosses the wasm boundary as
/// its Rust discriminant, so entry N in the generated `FILE_KIND_INDICES` is what
/// the compiled module reports as N. A gap or a repeat would make one format
/// answer under another's name, and `test/kinds.test.js` catches that against the
/// compiled module -- but only after the dataset has been committed and the
/// generated table regenerated, so this is the check that runs first.
#[test]
fn abi_order_is_a_permutation_of_the_discriminants() {
    let rows = formats();
    let mut seen: BTreeMap<usize, &str> = BTreeMap::new();
    for row in &rows {
        let order = row
            .abi_order
            .unwrap_or_else(|| panic!("{} has no `abi_order`", row.variant));
        assert!(
            order < 114,
            "{} has abi_order {order}, which is not a discriminant",
            row.variant,
        );
        if let Some(other) = seen.insert(order, row.variant.as_str()) {
            panic!(
                "abi_order {order} is claimed by both {other} and {}",
                row.variant
            );
        }
    }
    assert_eq!(
        seen.len(),
        114,
        "abi_order is missing {} position(s); it has to be 0..113 with no gaps",
        114 - seen.len(),
    );
}

/// Every token is distinct, and every token is one lowercase word.
///
/// `token` is the Python enum's member *value*, so a duplicate would silently
/// alias one member to another and `FileKind("psd")` would answer with whichever
/// was declared second. Enum aliasing cannot be caught from inside Python -- the
/// class simply has fewer members than the table -- so the check belongs beside
/// the data.
#[test]
fn tokens_are_distinct_and_lowercase() {
    let rows = formats();

    let mut owners: BTreeMap<&str, usize> = BTreeMap::new();
    for row in &rows {
        *owners.entry(row.token.as_str()).or_insert(0) += 1;
    }
    let shared: Vec<&str> = owners
        .iter()
        .filter(|(_, count)| **count > 1)
        .map(|(token, _)| *token)
        .collect();
    assert!(
        shared.is_empty(),
        "these tokens are used by more than one format: {}. The Python enum would alias one \
         member to another and lose a format.",
        shared.join(", "),
    );

    for row in &rows {
        // Not "shorter than the variant", which is the rule that looks obvious and
        // is wrong: `Lua` is three letters and its token is `luac`, because the
        // token names the byte code rather than the interpreter's usual extension,
        // and nobody ships a `.lua` this table would recognise. What a token does
        // have to be is one lowercase word, because it is a value a caller types
        // and a serialised form that ends up in a URL.
        assert!(!row.token.is_empty(), "{} has an empty token", row.variant);
        assert!(
            row.token.bytes().all(|b| {
                b.is_ascii_lowercase()
                    || b.is_ascii_digit()
                    || b == b'+'
                    || b == b'.'
                    || b == b'_'
                    || b == b'-'
            }),
            "{} has token {:?}, which is not a lowercase token",
            row.variant,
            row.token,
        );
    }
}

/// A missing value is `null`, never a made-up one.
///
/// The shape is the assertion, and it is the one that matters: a MIME type is
/// served to a browser, so an invented one is indistinguishable from a real one at
/// the point it causes harm. `null` is visible to a caller and forces the decision
/// to be made where it is made.
#[test]
fn an_absent_value_is_null_rather_than_a_guess() {
    let rows = formats();

    for row in &rows {
        let name = row.variant.as_str();
        if let Some(mime) = &row.mime {
            assert!(
                mime.contains('/'),
                "{name} has MIME type {mime:?}, which has no `type/subtype` separator",
            );
            assert!(
                !mime.starts_with('.'),
                "{name} has MIME type {mime:?}, which starts with a dot",
            );
        }
        if let Some(extension) = &row.extension {
            assert!(
                !extension.starts_with('.'),
                "{name} has extension {extension:?}, which carries a leading dot",
            );
            assert!(
                !extension.contains('/') && !extension.contains('\\'),
                "{name} has extension {extension:?}, which carries a path separator",
            );
        }
    }

    // Measured rather than assumed, and stated so that a verified registration
    // arriving shows up here as a test to update rather than as a silent change.
    let without_mime: Vec<&str> = rows
        .iter()
        .filter(|row| row.mime.is_none())
        .map(|row| row.variant.as_str())
        .collect();
    assert_eq!(
        without_mime.len(),
        16,
        "{} formats have no MIME type, expected 16: {}",
        without_mime.len(),
        without_mime.join(", "),
    );
    assert!(
        without_mime.contains(&"ELF"),
        "ELF is absent from the list of formats with no MIME type. If its registration was \
         checked and rejected on purpose, say so here; if not, `formats.json` is wrong.",
    );
}

/// The readme's format rows and the dataset are the same list.
///
/// Not about the magic bytes -- `tests/readme_coverage.rs` checks those against
/// `SIGNATURE_KIND`, which is the only authority for them. This is about the
/// names: a readme row saying `PNG` beside a dataset saying `PNG image` is the
/// exact drift the dataset exists to prevent, and it is invisible to every other
/// test, because each side is checked against itself.
#[test]
fn the_readme_and_the_dataset_name_the_same_formats() {
    let rows = formats();
    let readme = std::fs::read_to_string(repo_root().join("readme.md"))
        .expect("the repository readme is readable");

    let mut in_readme: BTreeMap<&str, String> = BTreeMap::new();
    for line in readme.lines() {
        // `| PNG | \`Png\` | ... |` -- the first two cells of the generated table.
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        if cells.len() != 6 {
            continue;
        }
        // The header row is `| Format | \`FileKind\` | Magic | Offset |`, and
        // `FileKind` after the backticks is an identifier like any other. Skipping
        // it by its first cell rather than by position is what keeps a table with
        // a different number of columns from being half-read.
        //
        // An *empty* first cell is the other thing that looks like a row here, and
        // it cost this test a false positive before anything was broken: a table
        // whose header is `| | Crate | \`PyPI\` | \`npm\` |` has four columns and a
        // blank leading cell, so `Crate` lands in the second position and reads as
        // a variant name. That was the readme's table of the three bindings, and
        // the count came out 115. A row with nothing in its first cell is a header
        // or a separator, never a format.
        if cells[1].is_empty() || cells[1] == "Format" {
            continue;
        }
        let variant = cells[2].trim_matches('`');
        // The display name is compared with its markdown stripped, because the
        // readme writes `` `StarDict` binary `` and the dataset writes
        // `StarDict binary`. That difference is deliberate and one-directional:
        // markdown formatting is a property of the document, not of the format, so
        // it stays in the readme and the dataset holds the plain name that Rust,
        // Python and JavaScript hand to a caller.
        let shown = cells[1].replace('`', "");
        let mut chars = variant.chars();
        let is_identifier = chars
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && chars.all(|c| c.is_ascii_alphanumeric() || c == '_');
        if !is_identifier {
            continue;
        }
        in_readme.insert(variant, shown);
    }
    assert_eq!(
        in_readme.len(),
        114,
        "readme.md has {} identifiable format rows, expected 114",
        in_readme.len(),
    );

    for row in &rows {
        let shown = in_readme.get(row.variant.as_str()).unwrap_or_else(|| {
            panic!(
                "formats.json names {}, which has no row in readme.md. The table is \
                 `Format | FileKind | Magic | Offset`.",
                row.variant
            )
        });
        assert_eq!(
            shown.as_str(),
            row.name,
            "readme.md calls {} `{}` and formats.json calls it `{}`",
            row.variant,
            shown,
            row.name,
        );
    }
}
