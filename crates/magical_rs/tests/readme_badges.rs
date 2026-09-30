//! The readme's badges, checked.
//!
//! This file exists because a badge is the one thing in a readme that cannot be
//! wrong in a way a reader notices for a long time. A number goes stale, which is
//! what a dynamic badge is for; a *URL* goes stale silently, and a dead badge looks
//! like the project abandoned that line of the page. Nothing about a broken image
//! fails a build.
//!
//! So the URLs are not fetched. A test that needs the network is a test that fails
//! for reasons nobody is looking at, and this project has no network-dependent test
//! of any kind. What is checked is the part that can be checked offline and is the
//! part that actually breaks: the scheme, the host, and the package name inside the
//! URL. Every one of those is a place a rename leaves a stale badge, and every one
//! of them is answerable from this repository.
//!
//! The scanning is hand-written rather than a `regex` dependency, for the reason
//! `tests/dataset.rs` does its own: a test that pulls in a crate to read a file
//! introduces a version to keep current and a thing to compile, and the pattern here
//! is a scan for `![`, then `](`, then a URL -- which is a dozen lines.

use std::path::{Path, PathBuf};

/// Every badge image target in a markdown document, in order.
///
/// Markdown's image syntax: `![alt](target "title")`. The alt text and an optional
/// title are skipped rather than matched, so a badge with a title still counts once.
fn badges(text: &str) -> Vec<&str> {
    let mut found = Vec::new();
    let mut at = 0_usize;
    while let Some(bang) = text[at..].find("![") {
        let mut cursor = at + bang + 2;
        // The alt text runs to the closing `]`, which a URL cannot contain unescaped.
        let Some(alt) = text[cursor..].find(']') else {
            break;
        };
        cursor += alt + 1;
        if !text[cursor..].starts_with('(') {
            at = cursor;
            continue;
        }
        cursor += 1;
        let end = text[cursor..]
            .find([')', ' '])
            .unwrap_or(text.len() - cursor);
        found.push(&text[cursor..cursor + end]);
        at = cursor + end;
    }
    found
}

/// A registry a badge may point at, and the manifest that names it.
struct Registry {
    /// The path segment shields.io uses for it: `crates`, `pypi`, `npm`.
    segment: &'static str,
    /// The name the registry knows this project by. An npm one contains a slash of
    /// its own, which is why this is a substring test rather than a path segment
    /// comparison.
    package: &'static str,
    /// Where that name is declared.
    manifest: &'static str,
}

const REGISTRIES: &[Registry] = &[
    Registry {
        segment: "crates",
        package: "magical_rs",
        manifest: "crates/magical_rs/Cargo.toml",
    },
    Registry {
        segment: "pypi",
        package: "magical-py",
        manifest: "bindings/python/pyproject.toml",
    },
    Registry {
        segment: "npm",
        package: "@reim-developer/magical-js",
        manifest: "bindings/nodejs/package.json",
    },
];

/// The readmes that carry badges, and how many each is expected to have.
///
/// The counts are the anti-deletion guard. A badge quietly removed from one readme
/// is a failure a reader notices on the page they were sent to and nothing else
/// would mention -- but a *bulk* deletion cannot pass either, because this line
/// changes in the same diff and a reviewer can see it.
const FILES: &[(&str, usize)] = &[
    ("readme.md", 10),
    ("bindings/python/README.md", 5),
    ("bindings/nodejs/README.md", 5),
];

/// Hosts a badge may be served from.
///
/// https, and a host this project or a registry controls. A badge over http, or from
/// a host nobody here owns, is either a mistake or a tracking pixel; neither belongs
/// in a readme that renders for every visitor.
const HOSTS: &[&str] = &[
    "img.shields.io",
    "github.com",
    "www.npmjs.com",
    "pypi.org",
    "crates.io",
    "docs.rs",
    "nodejs.org",
];

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

fn read(rel: &str) -> String {
    let path = repo_root().join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// The `name = "..."` a Cargo or pyproject manifest declares.
fn manifest_name(text: &str) -> Option<&str> {
    let line = text.lines().find(|line| line.starts_with("name"))?;
    line.split('"').nth(1)
}

/// The `name` field of a `package.json`, read without a JSON parser.
///
/// The file is written by this repository and read by npm; a scan for the one key
/// this test needs is half a dozen lines, and pulling in `serde_json` for it would
/// make the crate's test suite depend on a crate the crate itself does not have.
fn package_json_name(text: &str) -> Option<&str> {
    let key = "\"name\":";
    let at = text.find(key)? + key.len();
    let rest = text[at..].trim_start();
    rest.strip_prefix('"')?.split('"').next()
}

/// Every badge is https, on a host this project or a registry controls.
#[test]
fn every_badge_is_https_on_a_known_host() {
    let mut problems = Vec::new();

    for (file, _) in FILES {
        let text = read(file);
        for url in badges(&text) {
            let Some(authority) = url.strip_prefix("https://") else {
                problems.push(format!("{file}: {url} is not https"));
                continue;
            };
            let host = authority.split('/').next().unwrap_or_default();
            if !HOSTS.contains(&host) {
                problems.push(format!("{file}: {url} is served from an unvetted host"));
            }
        }
    }

    assert!(
        problems.is_empty(),
        "{} badge(s) would not be trusted:\n  {}",
        problems.len(),
        problems.join("\n  "),
    );
}

/// Every badge aimed at a registry names the package that registry publishes.
///
/// Asserted as "a badge on crates.io contains `magical_rs`", which is the shape of
/// the failure a rename causes: the badge keeps working and keeps reporting a
/// number, for a crate that no longer exists. There is no test that catches that
/// without fetching the URL, and this is the offline half of the check.
#[test]
fn every_registry_badge_names_a_package_that_exists() {
    let mut problems = Vec::new();

    for (file, _) in FILES {
        let text = read(file);
        for url in badges(&text) {
            for registry in REGISTRIES {
                let marker = format!("/{}/", registry.segment);
                if !url.contains(&marker) {
                    continue;
                }
                if !url.contains(registry.package) {
                    problems.push(format!(
                        "{file}: {url} is aimed at {} but does not name {}",
                        registry.segment, registry.package
                    ));
                }
            }
        }
    }

    assert!(
        problems.is_empty(),
        "{} badge(s) name a package that does not exist:\n  {}",
        problems.len(),
        problems.join("\n  "),
    );
}

/// The name each registry badge uses is the one the manifest declares, right now.
///
/// This is what makes the file unable to go stale the way a hand-written number
/// does: every name is read out of a manifest, so a rename that misses a badge is a
/// failing test rather than a dead image.
#[test]
fn every_registry_name_matches_its_manifest() {
    for registry in REGISTRIES {
        let text = read(registry.manifest);
        let declared = if registry.manifest.ends_with("package.json") {
            package_json_name(&text)
        } else {
            manifest_name(&text)
        };

        assert_eq!(
            declared,
            Some(registry.package),
            "{} no longer publishes {}; update every badge URL",
            registry.manifest,
            registry.package,
        );
    }
}

/// Every readme has a download badge, and the root has one per registry.
#[test]
fn every_readme_has_a_download_badge() {
    for (file, _) in FILES {
        let text = read(file);
        let found = badges(&text);
        let downloads = found
            .iter()
            .filter(|url| url.contains("/d") || url.contains("/dm"))
            .count();
        assert!(
            downloads > 0,
            "{file} has no download badge. The numbers on it are meant to be live, and a badge \
             quietly deleted is a failure a reader notices on the page they were sent to and \
             nothing else would mention."
        );
    }

    let root = read("readme.md");
    let root_badges = badges(&root);
    for registry in REGISTRIES {
        let marker = format!("/{}/", registry.segment);
        assert!(
            root_badges.iter().any(|url| url.contains(&marker)),
            "readme.md has no {} badge, and the two binding readmes do. This is the page the \
             crate is read from.",
            registry.segment,
        );
    }
}

/// The badge count is unchanged, so this file cannot be the thing that removed them.
#[test]
fn the_badge_counts_are_stable() {
    for (file, expected) in FILES {
        let text = read(file);
        assert_eq!(
            badges(&text).len(),
            *expected,
            "the badge count in {file} changed. Adding or removing one is deliberate, so this \
             diff is where to notice it -- but check that it is what you meant.",
        );
    }
}
