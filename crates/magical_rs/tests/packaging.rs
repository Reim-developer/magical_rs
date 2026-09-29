//! What actually ships, checked against what the manifest says ships.
//!
//! `Cargo.toml` used to be the repository root, so `LICENSE` and
//! `CHANGELOG.md` were inside the package. The library moved into
//! `crates/magical_rs/`, and both are now outside it.
//!
//! What cargo does with a file outside the package depends on which key names
//! it, and the two keys disagree in a way that is silent in both cases:
//!
//! * `readme = "../../readme.md"` works. Cargo copies the file into the archive
//!   as `readme.md`.
//! * `include = ["../../CHANGELOG.md"]` does not. A `..` segment is ignored, so
//!   the entry matches nothing, no warning is printed, and the published crate
//!   is missing the file.
//!
//! So `CHANGELOG.md` and `LICENSE` are copied into the crate rather than
//! referenced, and this test is what makes the copies safe. The files at the
//! repository root are the ones a human edits; if a copy ever differs, this
//! fails and names both paths.
//!
//! `readme` is listed here too even though it is a reference rather than a
//! copy, because it is the one that works and it stops looking like the other
//! two.

use std::path::{Path, PathBuf};

/// Files that live at the repository root and are copied into the package
/// because cargo will not reach outside it for them.
const COPIED: [&str; 2] = ["LICENSE", "CHANGELOG.md"];

/// The repository root, found by walking up from this crate.
///
/// Walking rather than counting `..` for the reason `tests/workspace.rs` gives:
/// a depth that is correct today is a second thing to update when the layout
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

/// The manifest of this crate.
fn manifest() -> &'static str {
    static TEXT: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    TEXT.get_or_init(|| {
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
            .expect("the crate's own Cargo.toml is readable")
    })
}

/// Every copy is byte-identical to the file it was copied from.
#[test]
fn every_copied_file_matches_the_one_at_the_repository_root() {
    let root = repo_root();
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    for name in COPIED {
        let source = root.join(name);
        let copy = crate_dir.join(name);

        let source_bytes = std::fs::read(&source)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", source.display()));
        let copy_bytes =
            std::fs::read(&copy).unwrap_or_else(|e| panic!("cannot read {}: {e}", copy.display()));

        assert_eq!(
            source_bytes,
            copy_bytes,
            "{} and {} differ. The root file is the one to edit; copy it over the other, or the \
             published crate ships one text and the repository shows another.",
            source.display(),
            copy.display(),
        );
    }
}

/// The license is MIT, and the copy is the MIT text rather than a file of the
/// right size with the wrong words.
///
/// A manifest can say `license = "MIT"` while shipping something else, and
/// `cargo publish` will not notice.
#[test]
fn the_packaged_license_is_the_mit_text() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("LICENSE");
    let bytes =
        std::fs::read(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let text = String::from_utf8(bytes).expect("a license file is text");

    assert!(
        text.contains("MIT License") && text.contains("WITHOUT WARRANTY OF ANY KIND"),
        "{} does not look like the MIT text",
        path.display(),
    );
    assert!(
        manifest().contains("license = \"MIT\""),
        "the manifest no longer declares `license = \"MIT\"`, and the packaged text is the only \
         thing left to go by",
    );
}

/// Everything that has to ship is named in `include`, or is a file cargo ships
/// on its own.
///
/// `include` is not a filter over what cargo already found, it is the list of
/// what to package. A file that is not named does not ship even when it sits in
/// the package directory, unless cargo special-cases its name -- which it does
/// for `LICENSE*` today, and might not tomorrow.
///
/// The assertion is on the manifest rather than on the archive because
/// building the archive means running `cargo package`, which means running
/// cargo inside a test. The manifest is the thing that decides.
#[test]
fn every_file_that_has_to_ship_is_listed_in_include() {
    let text = manifest();
    let start = text
        .find("include")
        .expect("the manifest has an `include` key");
    let listed = text[start..]
        .split(']')
        .next()
        .unwrap_or_else(|| panic!("`include` has no closing bracket: {}", &text[start..]));

    for name in COPIED {
        assert!(
            listed.contains(&format!("\"{name}\"")),
            "`include` does not name {name}: {listed:?}",
        );
    }
    // And the readme, which is reached the other way but has to be reached.
    assert!(
        text.contains("readme = \"../../readme.md\""),
        "the manifest no longer points `readme` at the repository readme, so the published crate \
         has no documentation of its own",
    );
}
