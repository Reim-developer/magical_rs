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
//! So `CHANGELOG.md`, `LICENSE` and `readme.md` are copied into the crate rather
//! than referenced, and this test is what makes the copies safe. The files at
//! the repository root are the ones a human edits; if a copy ever differs, this
//! fails and names both paths.
//!
//! `readme.md` was the odd one out, and it is now a copy for the same reason as
//! the other two rather than for a different reason. It used to be a reference:
//! `readme = "../../readme.md"`, which cargo honours by copying the file into
//! the archive as `readme.md` at the package root. That produces the right
//! *archive* and the wrong *tree* — `src/lib.rs` reaches it as
//! `include_str!("../../../readme.md")`, which resolves in this repository and
//! not in the package. Every build and every test was green, because `cargo
//! build` compiles the source tree; `cargo publish` compiles the archive, and
//! failed at its verify step on 0.6.5 with
//!
//! ```text
//! error: couldn't read `src/../../../readme.md`: No such file or directory
//! ```
//!
//! That failure was after the version was tagged and the tag pushed, which is
//! the worst moment for it. `the_documented_readme_path_exists_inside_the_package`
//! is the test that would have caught it.

use std::path::{Path, PathBuf};

/// Files that live at the repository root and are copied into the package
/// because cargo will not reach outside it for them.
const COPIED: [&str; 3] = ["LICENSE", "CHANGELOG.md", "readme.md"];

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
    // And the readme key has to name the *copy*, not the file at the repository
    // root. Cargo honours a `readme` outside the package by copying it in under
    // the name `readme.md`, so both spellings put a `readme.md` in the archive and
    // only one of them puts it where `src/lib.rs` can reach it — the one inside the
    // package. Pointing outside is what made `cargo publish` fail on 0.6.5 with
    // `couldn't read src/../../../readme.md`, and it failed *after* the crate had
    // been versioned and tagged, so the tag was already pushed when this surfaced.
    assert!(
        text.contains("readme = \"readme.md\""),
        "the manifest points `readme` somewhere other than the checked-in copy. If it points at \
         `../../readme.md`, cargo copies that file in as `readme.md` and `src/lib.rs` -- which \
         reaches it as `../readme.md` -- is fine; but the copy is what makes that path true, so \
         pointing outside while `include` also names the copy is two sources of truth for one file.",
    );
    assert!(
        !text.contains("readme = \"../../readme.md\""),
        "the manifest points `readme` at the repository root again. Cargo copies it into the \
         archive as `readme.md` and the published crate still builds, but `src/lib.rs` reaching \
         `../readme.md` now resolves to whatever is checked in -- so the two files have to be \
         kept in sync by hand for no benefit. Name the copy.",
    );
}

/// The path in `src/lib.rs` reaches the readme **inside the package**.
///
/// This is the test that would have caught the 0.6.5 publish failure, and it is
/// here because nothing else did. Every build in CI compiles the source tree, where
/// three levels up is the repository readme and the path works; `cargo publish`
/// compiles the copied archive instead, where the readme is at the package root and
/// the same path does not exist. So the source could be wrong for a whole release
/// with a green build and a green test suite, and would only speak up at the last
/// step before an upload.
///
/// Read rather than assumed, because the answer is a property of what `readme` in
/// the manifest points at, and that property changed while the path did not.
#[test]
fn the_documented_readme_path_stays_inside_the_package() {
    let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs");
    let text = std::fs::read_to_string(&lib)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", lib.display()));

    // The one `include_str!` that names a markdown file. There is only one, and
    // this test says so rather than assuming it.
    let needle = "include_str!(\"";
    let start = text
        .find(needle)
        .unwrap_or_else(|| panic!("{} has no include_str!", lib.display()))
        + needle.len();
    let rest = &text[start..];
    let end = rest.find('"').unwrap_or_else(|| {
        panic!(
            "the include_str! path in {} has no closing quote",
            lib.display()
        )
    });
    let declared = Path::new(&rest[..end]);

    // **`..` is the whole question, and it is asked by counting them, not by
    // resolving the path.** The first version of this test resolved `declared`
    // against `CARGO_MANIFEST_DIR` and asserted the result existed -- which is
    // the source tree, the one place the broken path *does* resolve. Three levels
    // up from `crates/magical_rs/src` is the repository readme, it is right there,
    // and the test passed on the exact code that could not be published.
    //
    // What `cargo publish` compiles is the archive, whose root is the package
    // directory and nothing above it. A `..` segment inside the archive leaves the
    // archive: `src/..` is the package root, `src/../..` is already outside it, and
    // the file the compiler is looking for is not on disk anywhere. So the count
    // is the property, and checking that the file exists in *this* tree is
    // checking the one thing that was never broken.
    let ups = declared
        .components()
        .filter(|c| matches!(c, std::path::Component::ParentDir))
        .count();

    assert_eq!(
        ups,
        1,
        "`src/lib.rs` reaches the readme as `{}`, which climbs {ups} level(s) out of `src/`. \
         Inside the published archive `src/..` is the package root and `src/../..` is outside the \
         archive, where nothing exists -- so `cargo publish` fails at its verify step with \
         `couldn't read src/{}` while every build in CI passes, because CI compiles the source \
         tree where the repository readme really is three levels up. Exactly one `..` is correct.",
        declared.display(),
        declared.display(),
    );

    // And the one level it climbs to has to be a readme that is actually shipped,
    // which is a separate question from the path being well-formed: a correct
    // path to a file absent from `include` is still a publish failure.
    let lands_on = declared
        .components()
        .rev()
        .find(|c| !matches!(c, std::path::Component::CurDir))
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .unwrap_or_default();
    assert_eq!(
        lands_on,
        "readme.md",
        "`src/lib.rs` reaches `{}` one level up, which is `src/..` -- the package root. Cargo puts \
         the readme there under the name `readme.md`, whatever the key pointed at, so this names a \
         file that will not be there.",
        declared.display(),
    );
}
