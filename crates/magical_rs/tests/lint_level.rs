//! Every crate in the repository denies the same clippy groups.
//!
//! The groups are passed twice already -- once as `#![deny(...)]` at the top of
//! each crate root, and once on the clippy command line in every gate script --
//! and having them in both places is a duplication with a real failure mode. The
//! attribute was lost from `magical-file` when a doc comment was inserted above
//! it, and **nothing failed**: every gate still passes the groups on the command
//! line, so a crate that had stopped denying four lint groups built and tested
//! and linted green. The gate could not see it because the gate was not the
//! thing that had changed.
//!
//! This test reads each crate root and asserts the attribute is there. It is a
//! text check rather than a compile check on purpose: a `#![deny]` that is
//! present but names the wrong groups is exactly the case worth catching, and a
//! `#[test]` that compiled the crate would say nothing about what it denies.
//!
//! The crate list is written out rather than discovered by walking the tree,
//! for the same reason `ci_coverage.rs` writes out the workflow it reads: a list
//! found by searching is a list that finds whatever happens to be there, and a
//! new crate that is not in it would be silently unchecked.

use std::path::{Path, PathBuf};

/// A crate root, and the file it should be in.
///
/// Every crate that ships a library. The examples under `examples/` are `main.rs`
/// binaries and are not listed: they are demonstration code that is run rather
/// than published, `make linter` does not reach them, and holding a throwaway to
/// the same bar as the code being distributed would be a rule with nothing behind
/// it. If one of them grows a library it joins this list.
const CRATES: &[(&str, &str)] = &[
    ("crates/magical_rs", "src/lib.rs"),
    ("bindings/python", "src/lib.rs"),
    ("bindings/nodejs", "src/lib.rs"),
];

/// The four groups, in the order the library crate writes them.
const GROUPS: &[&str] = &[
    "clippy::pedantic",
    "clippy::all",
    "clippy::nursery",
    "clippy::perf",
];

/// Every crate denies all four groups.
#[test]
fn every_crate_denies_the_same_clippy_groups() {
    let root = repo_root();
    let mut problems: Vec<String> = Vec::new();

    for (crate_dir, file) in CRATES {
        let path = root.join(crate_dir).join(file);
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!("cannot read {}: {e}", path.display());
        });

        // The attribute has to be a crate-level `#![deny(..)]`, not a
        // `#[deny]` on an item and not the same word in a comment, so the line
        // is matched whole and the groups are then looked for inside it.
        let Some(deny) = text
            .lines()
            .find(|line| line.starts_with("#![deny(clippy::"))
        else {
            problems.push(format!(
                "{crate_dir}/{file} has no crate-level `#![deny(clippy::..)]` attribute"
            ));
            continue;
        };

        for group in GROUPS {
            if !deny.contains(group) {
                problems.push(format!("{crate_dir}/{file} does not deny {group}"));
            }
        }
    }

    assert!(
        problems.is_empty(),
        "{} of {} crate(s) are not held to the same bar:\n  {}",
        problems.len(),
        CRATES.len(),
        problems.join("\n  "),
    );
}

/// The attribute is present in every crate, so removing one is a visible change.
///
/// A second assertion, and it exists because the first one is satisfied by a
/// comment. `deny(clippy::pedantic` written inside a `//!` line would satisfy a
/// search and deny nothing, and the way to tell is that the attribute is on a
/// line that starts at column zero -- which is required of every inner
/// attribute, so the check costs nothing and cannot be passed by prose.
#[test]
fn the_deny_attribute_is_an_attribute_and_not_a_comment() {
    let root = repo_root();
    for (crate_dir, file) in CRATES {
        let path = root.join(crate_dir).join(file);
        let text = std::fs::read_to_string(&path).expect("the crate root is readable");
        let attributes: Vec<&str> = text
            .lines()
            .filter(|line| line.contains("clippy::pedantic"))
            .collect();
        assert!(
            !attributes.is_empty(),
            "{} mentions no clippy group at all",
            path.display()
        );
        for line in attributes {
            assert!(
                line.starts_with("#![") || line.starts_with("#[allow("),
                "{} has a line mentioning clippy::pedantic that is neither an attribute nor an \
                 allow: {line:?}\nA mention in a comment would satisfy a search and deny nothing.",
                path.display()
            );
        }
    }
}

/// The repository root, found by walking up from this crate.
///
/// Walking rather than counting `..`, for the reason `tests/workspace.rs` and
/// `tests/packaging.rs` both give.
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
