//! The workspace manifest has to describe the tree it is the root of.
//!
//! `Cargo.toml` at the repository root is a virtual manifest now: no package,
//! just `members` and `exclude`. That moved two failure modes into it, and
//! both of them are silent until something is run that is not the workspace.
//!
//! **An unexcluded crate under the root.** A package that lives under a
//! workspace root and is neither a member nor excluded is an error -- "current
//! package believes it's in a workspace when it's not". It does not fail
//! `cargo test` at the root, which never looks at it. It fails `maturin
//! develop` in `bindings/python`, `cargo build` in `bindings/nodejs`, and
//! `cargo run` in whichever `examples/` directory was added last, each of which
//! is run by a different command on a different schedule. So a new example is a
//! broken example until someone happens to run it.
//!
//! **A `..` in `readme` that cargo quietly drops.** Not this file's business,
//! and mentioned only because it is the same shape: cargo copies a `readme`
//! from outside the package into the published archive and ignores the
//! same-looking `..` in `include`, so the `CHANGELOG.md` the old root manifest
//! listed would have stopped shipping with no warning at all. `tests/packaging.rs`
//! is where that is checked.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Directories whose crates are excluded from the workspace, and which
/// therefore have to be listed in `Cargo.toml` one crate at a time.
const EXCLUDED_TREES: [&str; 3] = ["benchmarks", "bindings", "examples"];

/// The repository root.
///
/// Found by walking up from this crate rather than by counting `..`, because a
/// depth that is correct today is a second thing to update when the layout
/// changes again, and the failure is a panic in a test whose name says nothing
/// about paths.
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

/// Every path listed in `workspace.exclude`.
///
/// Read out of the manifest as text rather than through `cargo metadata`,
/// because the question is what the manifest *says*, and `cargo metadata`
/// would answer it for the crates cargo did resolve -- which is exactly the
/// set that is wrong when this test is failing.
///
/// The list is read between the key and the closing bracket. TOML arrays may
/// span lines, which this one does, and reading only one line would silently
/// see an empty array and pass forever.
fn declared_excludes(root: &Path) -> BTreeSet<String> {
    let manifest = std::fs::read_to_string(root.join("Cargo.toml"))
        .unwrap_or_else(|e| panic!("cannot read Cargo.toml: {e}"));

    let start = manifest.find("exclude").unwrap_or_else(|| {
        panic!(
            "Cargo.toml has no `exclude` key. Every crate under {EXCLUDED_TREES:?} needs one, or \
             cargo refuses to build it as its own package."
        )
    });
    let after = &manifest[start + "exclude".len()..];
    let open = after
        .find('[')
        .unwrap_or_else(|| panic!("`exclude` is not an array: {after:?}"));
    let close = after[open..]
        .find(']')
        .unwrap_or_else(|| panic!("`exclude` has no closing bracket: {after:?}"));

    after[open + 1..open + close]
        .split(',')
        .filter_map(|entry| {
            let quoted = entry.trim().strip_prefix('"')?.strip_suffix('"')?;
            // A comment after a comma-separated entry is a line the parser
            // above is not going to understand, and one is written in this
            // manifest, so a bare `//` line is dropped rather than treated as
            // a path that happens to be missing.
            (!quoted.is_empty() && !quoted.contains("//")).then(|| quoted.replace('\\', "/"))
        })
        .collect()
}

/// Every crate under the excluded trees, as a path relative to the root.
///
/// Two shapes, because there are two shapes. `bindings/` and `examples/` are
/// directories *of* crates; `benchmarks/` **is** one crate, and looking only one
/// level down missed it -- which showed up as the reverse failure, the `exclude`
/// entry being reported as stale. So each tree is checked for its own manifest
/// and for one on each of its children.
///
/// One level below a tree, and no recursion. A crate two levels down would be
/// missed, and the fix would be a recursion here rather than a change to any
/// manifest.
fn crates_under(root: &Path) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let add = |found: &mut BTreeSet<String>, path: &Path| {
        if path.join("Cargo.toml").is_file() {
            found.insert(
                path.strip_prefix(root)
                    .expect("a tree is under the root")
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    };

    for tree in EXCLUDED_TREES {
        let dir = root.join(tree);
        let entries = std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
        add(&mut found, &dir);
        for entry in entries.flatten() {
            add(&mut found, &entry.path());
        }
    }
    found
}

/// Every crate outside the workspace is named in `exclude`.
#[test]
fn every_outside_crate_is_excluded_from_the_workspace() {
    let root = repo_root();
    let declared = declared_excludes(&root);
    let present = crates_under(&root);

    assert!(
        !present.is_empty(),
        "found no crate under {EXCLUDED_TREES:?}, so this test is not looking at anything",
    );

    let missing: Vec<&String> = present.difference(&declared).collect();
    assert!(
        missing.is_empty(),
        "{} crate(s) exist but are not in `workspace.exclude`: {}. Each one fails to build on its \
         own with \"current package believes it's in a workspace when it's not\", and not from \
         `cargo test` at the root, which never looks at them. Add each to the list in Cargo.toml.",
        missing.len(),
        missing
            .iter()
            .map(|p| p.as_str())
            .collect::<Vec<_>>()
            .join(", "),
    );

    // And the other direction, which is the one that costs a stranger. A path
    // in `exclude` that no longer exists is harmless to cargo, so it is exactly
    // the kind of line that stays behind a deleted crate and turns the list
    // into a list of history.
    let stale: Vec<&String> = declared.difference(&present).collect();
    assert!(
        stale.is_empty(),
        "`workspace.exclude` names {} that no longer exist: {}. cargo ignores a stale entry \
         silently, so drop them.",
        stale.len(),
        stale
            .iter()
            .map(|p| p.as_str())
            .collect::<Vec<_>>()
            .join(", "),
    );
}

/// The path dependencies in the excluded crates point at the library's new home.
///
/// Every crate outside the workspace reaches the library by a relative path, and
/// the library moved from the repository root into `crates/magical_rs`. A
/// dependency that still says `../..` resolves to the virtual manifest, which
/// has no package, and the error is "no matching package named `magical_rs`
/// found" -- which reads like a version problem and is a path problem.
///
/// This reads the manifests as text rather than resolving them, so a broken
/// path is reported here instead of by whichever tool happened to run.
#[test]
fn every_outside_crate_reaches_the_library_at_its_new_home() {
    let root = repo_root();
    let expected = "crates/magical_rs";
    let mut offenders: Vec<String> = Vec::new();

    for relative in crates_under(&root) {
        let manifest = root.join(&relative).join("Cargo.toml");
        let text = std::fs::read_to_string(&manifest)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", manifest.display()));

        // Only the dependency line, so a path in a comment or a `[patch]` for
        // something else is not what this is about.
        for line in text
            .lines()
            .filter(|l| l.trim_start().starts_with("magical_rs ="))
        {
            let resolved = line
                .split("path")
                .nth(1)
                .and_then(|rest| rest.split('"').nth(1))
                .unwrap_or_else(|| {
                    panic!("{relative} has a `magical_rs` dependency with no path: {line:?}")
                });
            if !resolved.ends_with(expected) {
                offenders.push(format!("{relative}: path = \"{resolved}\""));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "{} dependency path(s) do not reach {expected}: {}. The library moved into that directory; \
         a dependency still saying `../..` resolves to the virtual manifest, which has no package.",
        offenders.len(),
        offenders.join("; "),
    );
}
