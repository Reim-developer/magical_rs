//! Every relative link in `docs/` resolves, and to something that exists.
//!
//! This is not a style check. Documentation whose links are broken is worse than
//! no documentation, because it looks authoritative and sends a reader to a 404 at
//! the exact moment they are trying to believe it. `docs/` is cross-linked from
//! every page, which is the situation where a link rots fastest and nobody notices
//! until a reader hits it.
//!
//! Two failures are caught, and they are different failures:
//!
//! 1. the link points at a file that is not there — usually a page that was renamed,
//!    or a relative path written as if the file were one directory shallower;
//! 2. the link names an anchor no heading produces — usually a heading reworded.
//!
//! The second is the one that survives review by eye, because the link *looks*
//! right. Both were live when this file was written: several of the relative links
//! in `docs/api/` resolved one directory above where the target actually is, and
//! one anchor outlived the heading it was cut from.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

/// Collapses `.` and `..` lexically and returns one spelling of the path, so that
/// two ways of naming the same file compare equal.
///
/// This is done with `PathBuf::push` rather than by joining the component strings,
/// because on Windows the component list of `D:\a\b\..\c` is
/// `[Prefix("D:"), RootDir, "a", "b", ParentDir, "c"]` and re-joining those as
/// strings yields `D:\\a\moskov\c` — a UNC path, which resolves to nothing. Pushing
/// the components back onto a `PathBuf` keeps the prefix and root intact.
///
/// It is lexical rather than `canonicalize`d on purpose: `canonicalize` fails on a
/// path that does not exist, and "does not exist" is one of the two things this
/// test is looking for.
fn normalize(path: &Path) -> Result<PathBuf, String> {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    return Err(format!("`{}` climbs above the root", path.display()));
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    Ok(out)
}

/// The repository root, from this file's location rather than from the working
/// directory, so the test is right whether cargo runs it from the crate or from
/// above. Normalised, because every path this test compares against it is.
fn repo_root() -> PathBuf {
    normalize(&Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(".."))
        .expect("the crate is two directories below the repository root")
}

/// Every Markdown page this test holds links in: `docs/` recursively, plus the
/// repository readme, which is where a reader is sent first.
fn pages() -> Vec<PathBuf> {
    let root = repo_root();
    let mut found = vec![root.join("readme.md")];

    let mut stack = vec![root.join("docs")];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
            .flatten()
        {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "md") {
                found.push(path);
            }
        }
    }

    found.sort();
    assert!(
        found.len() > 1,
        "only the readme was found; docs/ is missing"
    );
    found
}

/// The anchors a page's headings produce, under GitHub's rule.
///
/// Lowercase, drop everything that is not a letter, digit, underscore, space or
/// hyphen, then spaces become hyphens. Repeated headings get `-1`, `-2` and so on,
/// which is why this returns a set rather than one slug per heading.
///
/// The underscore is kept because GitHub keeps it, and that is not a detail. The
/// readme has a heading that reads "Why `magical_fluent` is the one flag that is not
/// about a level", and its anchor is
/// `#why-magical_fluent-is-the-one-flag-that-is-not-about-a-level` — underscore and
/// all. A slugger that drops the underscore rejects a link GitHub resolves.
fn anchors_in(text: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();

    for line in text.lines() {
        let hashes = line.chars().take_while(|c| *c == '#').count();
        if hashes == 0 || hashes > 6 {
            continue;
        }
        let heading = line[hashes..].trim();

        let slug: String = heading
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_'))
            .map(|c| if c == ' ' { '-' } else { c })
            .collect();

        if slug.is_empty() {
            continue;
        }
        if !found.insert(slug.clone()) {
            let base = slug.clone();
            for n in 1.. {
                if found.insert(format!("{base}-{n}")) {
                    break;
                }
            }
        }
    }

    found
}

/// Resolves a link's file part against the directory holding the link.
///
/// An empty file part means the page itself, which is how `[x](#anchor)` and
/// `[x](path.md#anchor)` differ: the first names the page you are already on. That
/// case returns the input unchanged rather than normalising it, because the input
/// came from `pages()` and is already the spelling everything else is compared
/// against.
fn resolve(link_from: &Path, target: &str) -> Result<PathBuf, String> {
    if target.is_empty() {
        return Ok(link_from.to_path_buf());
    }
    normalize(
        &link_from
            .parent()
            .expect("every page has a parent")
            .join(target),
    )
    .map_err(|why| format!("`{target}` {why}"))
}

/// The relative links in a page's text, as `(as written, file part, anchor)`.
///
/// Deliberately not a Markdown parser: it looks for `](`, which is how every link
/// in `docs/` is written, and skips anything carrying a scheme or a space. A real
/// parser would be more correct, and would also be a parser that silently finds
/// zero links in a style it does not model — which is exactly the failure mode
/// this whole file exists to prevent. The count is asserted at the bottom for the
/// same reason.
fn links_in(text: &str) -> Vec<(String, String, String)> {
    let mut found = Vec::new();
    let mut at = 0;

    while let Some(start) = text[at..].find("](").map(|i| at + i) {
        let body_start = start + 2;
        let Some(rel_end) = text[body_start..].find(')').map(|i| body_start + i) else {
            break;
        };
        let body = &text[body_start..rel_end];
        at = rel_end + 1;

        if body.starts_with("http://")
            || body.starts_with("https://")
            || body.starts_with("mailto:")
            || body.contains(' ')
            || body.is_empty()
        {
            continue;
        }

        let (file, anchor) = match body.split_once('#') {
            Some((f, a)) => (f.to_owned(), a.to_owned()),
            None => (body.to_owned(), String::new()),
        };
        found.push((body.to_owned(), file, anchor));
    }

    found
}

/// The main check: every relative link in every page resolves to a file that
/// exists, and to an anchor that file actually produces.
#[test]
fn every_relative_link_in_docs_resolves() {
    let root = repo_root();
    let pages = pages();

    // Read every page once. The anchors of a page are needed by the pages that
    // link *to* it, so this cannot be done one page at a time.
    let mut text: BTreeMap<PathBuf, String> = BTreeMap::new();
    let mut anchors: BTreeMap<PathBuf, BTreeSet<String>> = BTreeMap::new();
    for page in &pages {
        let body = std::fs::read_to_string(page)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", page.display()));
        anchors.insert(page.clone(), anchors_in(&body));
        text.insert(page.clone(), body);
    }

    let mut checked = 0;
    let mut broken: Vec<String> = Vec::new();

    for page in &pages {
        let body = &text[page];
        let shown = page
            .strip_prefix(&root)
            .unwrap_or(page)
            .display()
            .to_string();

        for (raw, file, anchor) in links_in(body) {
            let resolved = match resolve(page, &file) {
                Ok(p) => p,
                Err(why) => {
                    broken.push(format!("{shown} -> {raw}: {why}"));
                    continue;
                }
            };

            // A link may point at a directory — `docs/`, `bindings/python`,
            // `examples/wasm_rust` — and GitHub renders those as a listing, so a
            // directory is a legitimate target. It may also point at something
            // that is not a page, such as `formats.json`. Either way the target
            // has to exist; only anchors are a Markdown question, and only a page
            // has headings.
            if !resolved.exists() {
                broken.push(format!("{shown} -> {raw}: no such file or directory"));
                continue;
            }

            if !anchor.is_empty() {
                match anchors.get(&resolved) {
                    None => broken.push(format!(
                        "{shown} -> {raw}: {file} carries no headings, so it has no anchors"
                    )),
                    Some(have) if !have.contains(&anchor.to_lowercase()) => {
                        broken.push(format!(
                            "{shown} -> {raw}: no heading makes the anchor #{anchor}"
                        ));
                    }
                    Some(_) => {}
                }
            }
            checked += 1;
        }
    }

    assert!(
        checked > 30,
        "only {checked} relative links were checked, so this test is not looking at \
         much; every page in docs/ is expected to cross-link the others"
    );
    assert!(
        broken.is_empty(),
        "{} of the relative links in docs/ do not resolve:\n  - {}",
        broken.len(),
        broken.join("\n  - ")
    );
}

/// Every page in `docs/` is reachable from its index, and the index is reachable
/// from the readme.
///
/// The direction that rots is the one this checks. A page that exists and is
/// correct is still documentation nobody finds, and adding a page is the point at
/// which the index is most likely to be left alone.
#[test]
fn every_page_is_reachable_from_the_index() {
    let root = repo_root();
    let index = root.join("docs").join("README.md");
    let index_text = std::fs::read_to_string(&index)
        .unwrap_or_else(|e| panic!("docs/README.md is the index and must exist: {e}"));

    let mut unlinked: Vec<String> = Vec::new();
    for page in pages() {
        if page == root.join("readme.md") || page == index {
            continue;
        }
        let name = page
            .strip_prefix(root.join("docs"))
            .expect("a docs page is under docs/")
            .to_string_lossy()
            .replace('\\', "/");

        // The index may link `api/rust.md` or `rust.md`; both name the same page to
        // a reader, and insisting on one spelling would be a style rule.
        let bare = name.rsplit('/').next().unwrap_or(&name);
        if index_text.contains(name.as_str()) || index_text.contains(bare) {
            continue;
        }
        unlinked.push(name);
    }

    assert!(
        unlinked.is_empty(),
        "these pages are in docs/ but the index never mentions them, so a reader \
         arriving at docs/README.md cannot get to them:\n  - {}",
        unlinked.join("\n  - ")
    );

    let readme =
        std::fs::read_to_string(root.join("readme.md")).expect("the repository readme is readable");
    assert!(
        readme.contains("docs/README.md"),
        "the readme does not link docs/README.md, so the index is reachable only by \
         guessing the path"
    );
}
