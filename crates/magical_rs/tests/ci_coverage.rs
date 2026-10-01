//! The `Makefile` and the workflows under `.github/workflows/` have to agree,
//! checked rather than kept in step by hand.
//!
//! `test-unsafe` has been in the `Makefile` since `8e09005` on 2025-08-07 and was
//! never in `crate_dev.yml`. Nothing was wrong with the target and nothing was
//! wrong with the workflow. The two files listed the same targets, by hand, and
//! the last addition to the first was never copied to the second. Three days
//! earlier, in `ea52943`, the workflow's step list had mirrored the `Makefile`
//! exactly, which is what makes this drift rather than a decision.
//!
//! So these tests read both files and check the relationship between them. A
//! target added to the `Makefile` with no CI step fails here, in this job, on
//! the push that added it, rather than at the next release.
//!
//! Two assertions, both about the same drift:
//!
//! * every target with a rule is reachable from a step, and
//! * every name in `.PHONY` has a rule.
//!
//! The second one is not the same check twice. A `.PHONY` name with no rule is
//! not a broken target, it is a target make considers already done, because a
//! phony name with no prerequisites and no recipe is satisfied by existing. This
//! file carried `target` in `.PHONY` since `b8fb23e` with no rule anywhere, and
//! `make target` printed `Nothing to be done` and exited 0, which reads like a
//! run that passed and is not one.
//!
//! The rule this states is that a target belongs in the `Makefile` only if CI
//! should be running it. A convenience nobody wants CI to run does not need a
//! target, and a helper that a step needs is reached with `$(MAKE)` from the
//! target that needs it, which is what `test-nostd` does with `build-nostd` and
//! what `bench` does with `bench-report`.
//! Reachability rather than membership, so a helper counts as covered without
//! needing a step of its own.

use std::collections::{BTreeMap, BTreeSet};

/// The workflows that between them have to cover the `Makefile`.
///
/// Was one name, `crate_dev.yml`, and became a list when `benchmarks.yml` was
/// added. The test's subject has not changed: what it is looking for is a
/// `Makefile` target that nothing runs, and `bench` and `bench-report` are run
/// — by a workflow, in a file that exists next to the one this used to read. The
/// thing that would have been wrong is adding a third target to the `Makefile`
/// and putting its step in a workflow nobody reads, and that is still caught.
///
/// Every file in the directory, not a list maintained here. A second name in
/// this array would be a third place for a workflow to be forgotten in, which is
/// the failure this file exists to prevent.
const WORKFLOWS: &str = ".github/workflows";

/// What the `Makefile` declares.
struct Makefile {
    /// Names that have a rule, in the order they appear.
    targets: Vec<String>,

    /// Names listed in `.PHONY`, which is a declaration and not a rule.
    phony: BTreeSet<String>,

    /// For each target, the targets its recipe invokes.
    invokes: BTreeMap<String, BTreeSet<String>>,
}

/// The repository root, found by walking up from this crate.
///
/// Not a fixed number of `..` segments. The crate sits in `crates/magical_rs`
/// and the root is two levels above it, but a depth that is correct today is a
/// second thing to update when the layout changes again, and the failure is a
/// panic in a test whose name says nothing about paths. Walking up until the
/// marker is found is correct wherever the crate ends up, and the marker is
/// unambiguous: only the repository root has a `Makefile` that a workflow runs.
fn repo_root() -> std::path::PathBuf {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
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

/// Reads a file from the repository root, naming the path when it is not there.
///
/// This is a failure and not a skip. `Cargo.toml`'s `include` is
/// `["src/**/*.rs", "LICENSE", "CHANGELOG.md"]`, so neither this file nor the
/// `Makefile` it reads ships to crates.io and it never runs for someone who
/// installed the package. A missing file here means a checkout that was pruned,
/// and returning quietly would make a broken tree look like a passing one,
/// which is the failure this whole file exists to make impossible.
fn read(rel: &str) -> String {
    let path = repo_root().join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

impl Makefile {
    /// Every name in `.PHONY`, as a set, for set difference against the rules.
    fn phony_set(&self) -> BTreeSet<&str> {
        self.phony.iter().map(String::as_str).collect()
    }

    /// Every name that has a rule, as a set.
    fn target_set(&self) -> BTreeSet<&str> {
        self.targets.iter().map(String::as_str).collect()
    }
}

/// The target name a line declares, if it declares one.
///
/// A rule is a name at column zero followed by a colon, optionally with
/// prerequisites after it. `.PHONY` begins with a dot and is handled by the
/// caller, and a colon followed by `=` is an immediate variable assignment
/// rather than a rule, so `CHECK := cargo` is not read as a target named
/// `CHECK`.
fn rule_name(line: &str) -> Option<&str> {
    if line.starts_with([' ', '\t']) {
        return None;
    }
    let (name, after) = line.split_once(':')?;
    if after.trim_start().starts_with('=') {
        return None;
    }
    let name = name.trim();
    let plausible = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'));
    plausible.then_some(name)
}

/// The targets one line invokes, through `make` or `$(MAKE)`.
///
/// `$(MAKE)` is rewritten to `make` first, because the recursive form is
/// otherwise invisible: the literal text `$(MAKE) build-nostd` does not contain
/// `make `.
fn invocations(line: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let mut rest = line.replace("$(MAKE)", "make ");
    while let Some(at) = rest.find("make ") {
        // `trim_start` is not optional. The recipe is `@$(MAKE) build-nostd`,
        // and the rewrite above turns that into `@make  build-nostd` with two
        // spaces, so without this the name comes back empty and the edge is
        // silently dropped.
        let after = rest[at + "make ".len()..].trim_start();
        let name: String = after
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
            .collect();
        if !name.is_empty() {
            found.insert(name);
        }
        rest = after.to_owned();
    }
    found
}

fn parse_makefile(text: &str) -> Makefile {
    let mut out = Makefile {
        targets: Vec::new(),
        phony: BTreeSet::new(),
        invokes: BTreeMap::new(),
    };

    // A recipe is a run of lines that begin with a tab, and the continuation
    // lines of a recipe that a trailing backslash produced, which indent with
    // spaces rather than a tab. Either way the line starts with whitespace, and
    // that is what tells it apart from the next rule.
    let mut open: Option<String> = None;

    for line in text.lines() {
        if let Some(rest) = line.trim_start().strip_prefix(".PHONY:") {
            out.phony.extend(rest.split_whitespace().map(str::to_owned));
            open = None;
            continue;
        }

        if let Some(name) = rule_name(line) {
            out.targets.push(name.to_owned());
            out.invokes.entry(name.to_owned()).or_default();
            open = Some(name.to_owned());
            continue;
        }

        match &open {
            Some(name) if line.starts_with([' ', '\t']) => {
                if let Some(children) = out.invokes.get_mut(name) {
                    children.extend(invocations(line));
                }
            }
            _ => open = None,
        }
    }

    out
}

/// The targets a workflow's `run:` block scalars invoke.
///
/// Only the bodies of `run:` blocks are read, for the reason the recipes are
/// read rather than the whole file: a step's comment that names a target is
/// prose. `crate_dev.yml` has one that says a contributor fixes formatting with
/// `make fmt` locally, and counting it would let a target pass this test
/// without any step ever running it.
fn workflow_targets(text: &str) -> BTreeSet<String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut found = BTreeSet::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let indent = line.len() - line.trim_start().len();
        let is_block_scalar = line
            .trim_start()
            .strip_prefix("run:")
            .is_some_and(|rest| rest.trim_start().starts_with(['|', '>']));

        if !is_block_scalar {
            i += 1;
            continue;
        }

        // Everything more indented than the `run:` key is body. A blank line is
        // not enough to end it, which is why it is skipped rather than compared.
        let mut j = i + 1;
        while j < lines.len() {
            let body = lines[j];
            if body.trim().is_empty() {
                j += 1;
                continue;
            }
            if body.len() - body.trim_start().len() <= indent {
                break;
            }
            found.extend(invocations(body));
            j += 1;
        }
        i = j;
    }

    found
}

/// Every target in the `Makefile` is run by a step in some workflow, or by a
/// target that a step runs.
#[test]
fn every_makefile_target_is_reachable_from_a_ci_step() {
    let makefile = parse_makefile(&read("Makefile"));

    // Every workflow, and every step in each. Sorted so the failure message names
    // the workflows in a fixed order rather than in whatever order the
    // filesystem returned them.
    let mut steps: BTreeSet<String> = BTreeSet::new();
    let mut searched: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(repo_root().join(WORKFLOWS)).expect("cannot read the workflows")
    {
        let path = entry
            .expect("cannot read a workflow directory entry")
            .path();
        if path.extension().and_then(|e| e.to_str()) != Some("yml") {
            continue;
        }
        searched.push(
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("<unnamed>")
                .to_owned(),
        );
        steps.extend(workflow_targets(
            &std::fs::read_to_string(&path).expect("cannot read a workflow"),
        ));
    }
    searched.sort();
    assert!(
        !searched.is_empty(),
        "no workflow was found under {WORKFLOWS}, so this test is not looking at anything",
    );

    // Walk the `$(MAKE)` edges out from whatever the steps name. `build-nostd`
    // is reached this way, from `test-nostd`, so it needs no step of its own.
    let mut covered: BTreeSet<String> = BTreeSet::new();
    let mut queue: Vec<String> = steps.iter().cloned().collect();
    while let Some(name) = queue.pop() {
        if !covered.insert(name.clone()) {
            continue;
        }
        if let Some(children) = makefile.invokes.get(&name) {
            queue.extend(children.iter().cloned());
        }
    }

    let unreached: Vec<&str> = makefile
        .target_set()
        .into_iter()
        .filter(|name| !covered.contains(*name))
        .collect();

    assert!(
        unreached.is_empty(),
        "the Makefile declares {} that no workflow reaches: {}. Searched {}. Add a step that runs \
         it, or, if it is a helper another target needs, have that target call it with `$(MAKE) \
         {}`.",
        unreached.len(),
        unreached.join(", "),
        searched.join(", "),
        unreached.first().copied().unwrap_or("target"),
    );
}

/// Every name in `.PHONY` has a rule behind it.
///
/// A `.PHONY` name with no rule is not a missing target, it is a satisfied one.
/// make treats a phony name with no prerequisites and no recipe as done, so
/// `make <name>` prints `Nothing to be done` and exits 0. That is a run that
/// looks like it passed, which is why it is checked here rather than left.
#[test]
fn every_phony_entry_has_a_rule() {
    let makefile = parse_makefile(&read("Makefile"));

    let orphans: Vec<&str> = makefile
        .phony_set()
        .into_iter()
        .filter(|name| !makefile.target_set().contains(*name))
        .collect();

    assert!(
        orphans.is_empty(),
        "the Makefile lists {} in .PHONY with no rule: {}. `make <name>` on one of these prints \
         `Nothing to be done` and exits 0 without doing anything, so it reads as a passing run. \
         Either write the rule or drop the name.",
        orphans.len(),
        orphans.join(", "),
    );
}
