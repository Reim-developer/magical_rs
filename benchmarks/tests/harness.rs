//! The harness is fair, and these are the tests that say so.
//!
//! A benchmark whose numbers come out flattering is worse than no benchmark,
//! because it is a claim. Every test here is about a way the harness could be
//! quietly unfair to one of the three libraries without anybody noticing, and
//! each one was written after that specific thing was found to be true.
//!
//! What is *not* here is a test asserting a number of nanoseconds. There is no
//! threshold anywhere in this crate and adding one would defeat its purpose: a
//! shared runner varies by a factor of two between jobs, and a test that fails
//! when the machine is busy gets muted within a month. The numbers are printed
//! for a person to read.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use magical_benchmarks::adapter::{self, Adapter, Magical};
use magical_benchmarks::corpus;
use magical_benchmarks::report;

/// Records what it was asked, so a test can prove every library saw the same
/// thing.
///
/// A real library is not substitutable here, so this is a stand-in: the point of
/// these tests is the harness, and the harness only cares that it made the same
/// calls to every adapter in the same order.
#[derive(Default)]
struct Recorder {
    /// Every buffer, or every path, this recorder was handed.
    log: std::sync::Mutex<Vec<Vec<u8>>>,
}

/// A [`Recorder`] the harness can hold by value and this test can still read.
///
/// The newtype is not decoration: `impl Adapter for Arc<Recorder>` is an orphan
/// rule violation, because `Arc` is defined in another crate and `Adapter` is
/// defined in this one. A `Box<dyn Adapter>` has to hold something sized, so the
/// alternatives are a newtype or `Box::leak` -- and `Box::leak` is a real leak in
/// a test binary that builds several sets of these.
struct Shared(Arc<Recorder>);

impl Adapter for Shared {
    fn name(&self) -> &'static str {
        "recorder"
    }

    fn read_size(&self) -> String {
        "-".to_owned()
    }

    fn detect(&self, bytes: &[u8]) -> usize {
        self.0.log.lock().expect("poisoned").push(bytes.to_vec());
        bytes.len()
    }

    fn detect_path(&self, path: &Path) -> usize {
        self.0
            .log
            .lock()
            .expect("poisoned")
            .push(path.to_string_lossy().as_bytes().to_vec());
        1
    }

    fn answer(&self, bytes: &[u8]) -> adapter::Answer {
        self.0.log.lock().expect("poisoned").push(bytes.to_vec());
        adapter::Answer::none()
    }

    fn answer_path(&self, path: &Path) -> adapter::Answer {
        self.0
            .log
            .lock()
            .expect("poisoned")
            .push(path.to_string_lossy().as_bytes().to_vec());
        adapter::Answer::none()
    }
}

/// Three recorders, boxed the way `adapter::all` boxes its real ones, with the
/// handles kept so a test can read the logs afterwards.
fn recorders() -> (Vec<Box<dyn Adapter>>, Vec<Arc<Recorder>>) {
    let handles: Vec<Arc<Recorder>> = (0..3).map(|_| Arc::new(Recorder::default())).collect();
    let adapters = handles
        .iter()
        .map(|h| Box::new(Shared(Arc::clone(h))) as Box<dyn Adapter>)
        .collect();
    (adapters, handles)
}

/// Every library is handed the same bytes, the same number of times.
///
/// The first version of the harness measured library A over the whole corpus,
/// then B, then C, and rotated the order between passes. That is not the same
/// thing, and the reason is memory rather than logic: the corpus is 10.5 MB, a
/// libmagic pass walks all of it, and the next library starts with a cold corpus
/// that it did not choose to have cold. Measured that way `magical_rs` came out
/// at 177 ns against Criterion's 57 ns for identical work.
///
/// What this test can prove is the part that is logic: the same buffers, the same
/// number of times, for every adapter, and nothing invented.
#[test]
fn every_library_sees_the_same_bytes_the_same_number_of_times() {
    let (adapters, handles) = recorders();
    let built = report::build(&adapters);

    // What each library was handed, as the multiset of lengths. Comparing the
    // lengths rather than the buffers is what makes this cheap -- 10.5 MB per
    // library would be a test nobody runs -- and it is enough: every in-memory
    // buffer is the same size, every path is a path, and the real files are the
    // real files.
    let mut shapes: Vec<Vec<usize>> = handles
        .iter()
        .map(|h| {
            let mut lengths: Vec<usize> = {
                // The lock guard in its own block, so it is released before the
                // sort rather than being held across it: the log is the largest
                // thing in this test and holding it for the sort would mean the
                // three of them are not collected one at a time.
                let log = h.log.lock().expect("poisoned");
                log.iter().map(Vec::len).collect()
            };
            lengths.sort_unstable();
            lengths
        })
        .collect();

    // Nothing invented: every length is either a corpus buffer, one of the four
    // real files, or short enough to be a path. A library handed a buffer of some
    // other size would be working on input the others never saw.
    let real_sizes: Vec<usize> = corpus::real_files().iter().map(|c| c.bytes.len()).collect();
    for (index, lengths) in shapes.iter().enumerate() {
        for &len in lengths {
            assert!(
                len == corpus::READ_SIZE || real_sizes.contains(&len) || len < 256,
                "recorder {index} was handed {len} bytes, which is not a corpus buffer \
                 ({}), not a real file ({real_sizes:?}) and too long to be a path",
                corpus::READ_SIZE,
            );
        }
    }

    // And the same work for each. This is the property fairness is: a table
    // where one library was called more often is a table comparing two different
    // amounts of work.
    let reference = shapes.pop().expect("three recorders");
    for (index, lengths) in shapes.iter().enumerate() {
        assert_eq!(
            *lengths, reference,
            "recorder {index} was handed a different set of inputs than the last one, so the \
             table would be comparing two different amounts of work"
        );
    }
    assert!(
        reference.len() > built.in_memory[0].samples.len(),
        "the recorders were called {} times, which is fewer than the {} passes reported",
        reference.len(),
        built.in_memory[0].samples.len(),
    );
}

/// The on-disk corpus is one file per case, not one file measured repeatedly.
///
/// The from-path table measures a library opening a file. If every case were
/// written to the same path, the last one would win and the table would be 286
/// measurements of a single buffer -- which is still a valid measurement of
/// *something*, just not of the corpus, and nothing in the output would say so.
///
/// The paths are read out of the recorders' logs rather than out of the
/// directory, because `Report::build` removes that on the way out and a test that
/// had to catch it mid-run would be a test about timing.
#[test]
fn the_on_disk_corpus_is_one_file_per_case() {
    let (adapters, handles) = recorders();
    let built = report::build(&adapters);

    let paths: Vec<String> = handles
        .iter()
        .flat_map(|h| h.log.lock().expect("poisoned").clone())
        .filter_map(|seen| String::from_utf8(seen).ok())
        .filter(|text| !text.is_empty() && text.contains("magical-bench-"))
        .collect();
    let distinct: std::collections::BTreeSet<&String> = paths.iter().collect();

    assert!(!paths.is_empty(), "no from-path call was recorded at all");
    assert_eq!(
        distinct.len(),
        built.in_memory[0].calls_per_pass,
        "the from-path pass opened {} paths of which {} are distinct, and one pass makes {} calls \
         -- the corpus is one file being measured repeatedly",
        paths.len(),
        distinct.len(),
        built.in_memory[0].calls_per_pass,
    );
}

/// A corpus written to disk is removed when the guard is dropped.
///
/// **On the guard, not on `build`.** The first version snapshotted the temporary
/// directory before and after a whole `build` and compared the two lists, which
/// is only a test of the cleanup if nothing else is writing there. Fifteen tests
/// in this file run in parallel and several of them build a report, so it failed
/// on CI and passed on a developer machine -- the worst behaviour for a test whose
/// whole subject is that the temporary directory does not fill up.
///
/// The guard names its own directory, so this is exact rather than a difference
/// between two snapshots of a shared directory.
#[test]
fn a_written_corpus_is_removed_when_the_guard_is_dropped() {
    let cases: Vec<corpus::Case> = (0..3)
        .map(|i| corpus::Case {
            name: format!("case{i}"),
            expected: None,
            bytes: vec![0_u8; 16],
        })
        .collect();

    let directory = {
        let written = report::Written::new(&cases);
        let (directory, paths) = written.written();

        assert!(
            directory.is_dir(),
            "{} was not created",
            directory.display()
        );
        assert_eq!(paths.len(), cases.len(), "one file per case");
        for path in paths {
            assert!(path.is_file(), "{} is missing", path.display());
        }
        directory.to_path_buf()
    };

    assert!(
        !directory.exists(),
        "{} is still there after the guard was dropped, and it is 10.5 MB on a real run",
        directory.display()
    );
}

/// Two corpora written at the same time do not share a directory.
///
/// With only the process id in the name, fifteen parallel tests share one
/// directory, and whichever finishes first deletes the files the others are still
/// opening. The from-path table would then be a table of `magic_file` on a path
/// that no longer exists -- a number rather than an error, and a number that
/// looks like libmagic being slow.
#[test]
fn two_corpora_written_at_the_same_time_do_not_collide() {
    let cases: Vec<corpus::Case> = vec![corpus::Case {
        name: "one".to_owned(),
        expected: None,
        bytes: vec![1_u8; 8],
    }];
    let first = report::Written::new(&cases);
    let second = report::Written::new(&cases);

    assert_ne!(
        first.written().0,
        second.written().0,
        "two live corpora are in the same directory, so one of them is about to delete the other's \
         files"
    );
    for (_, paths) in [first.written(), second.written()] {
        for path in paths {
            assert!(path.is_file(), "{} is missing", path.display());
        }
    }
}
/// Two builds in a row produce the same corpus, and the same answer rows.
///
/// `Report::build` measures twice, once per entry point, and the sample buffers
/// used to be a `static`. They are not any more, and this is the test that says
/// so: if they were shared, the second measurement's median would be over twice
/// the passes it reports and the two tables would silently disagree about how
/// many passes there were.
#[test]
fn two_builds_agree_on_the_corpus() {
    let (adapters, _handles) = recorders();
    let first = report::build(&adapters);
    let second = report::build(&adapters);

    assert_eq!(first.positives, second.positives);
    assert_eq!(first.negatives, second.negatives);
    assert_eq!(first.read_size, second.read_size);
    assert_eq!(first.answers.len(), second.answers.len());
    assert_eq!(
        first.answers.iter().map(|r| &r.file).collect::<Vec<_>>(),
        second.answers.iter().map(|r| &r.file).collect::<Vec<_>>(),
    );
    for (a, b) in first.in_memory.iter().zip(&second.in_memory) {
        assert_eq!(a.calls_per_pass, b.calls_per_pass);
        assert_eq!(a.samples.len(), b.samples.len());
    }
}

/// The two entry points are measured on the same number of calls.
///
/// The from-path table reads files and the in-memory table reads buffers, and a
/// mismatch in the denominators would make the two tables incomparable in a way
/// nothing in the output would show.
#[test]
fn both_entry_points_see_the_same_corpus_size() {
    let (adapters, _handles) = recorders();
    let built = report::build(&adapters);
    let in_memory = built.in_memory[0].calls_per_pass;
    let from_path = built.from_path[0].calls_per_pass;
    assert_eq!(in_memory, from_path);
    assert_eq!(in_memory, built.positives + built.negatives);
}

/// This crate's own adapter is in the list, first, and says so.
///
/// The row order is not sorted by time and the list is not alphabetical, so
/// "this crate first" is a choice the report makes deliberately and a reader is
/// entitled to be able to check it. A test is cheaper than a reader having to.
#[test]
fn the_real_adapters_are_the_ones_under_test() {
    let names: Vec<String> = adapter::all().iter().map(|a| a.name().to_owned()).collect();
    assert_eq!(names[0], "magical_rs");
    assert!(names.contains(&"infer".to_owned()));
    // libmagic is behind a feature, and the report has to say which it got.
    assert_eq!(
        names.contains(&"libmagic".to_owned()),
        cfg!(feature = "libmagic"),
        "adapter::all and the report's `libmagic` flag disagree about the feature"
    );
}

/// The report prints the libraries it ran, and says so when one is missing.
///
/// A two-row table that reads like a three-row result is the failure this whole
/// crate is most able to produce, because libmagic is a C library that is not
/// always installed.
#[test]
fn the_report_says_which_libraries_ran() {
    let markdown = report::build(&adapter::all()).to_markdown();
    assert!(markdown.contains("`magical_rs`"));
    assert!(markdown.contains("`infer`"));
    if cfg!(feature = "libmagic") {
        assert!(markdown.contains("All three libraries ran"));
    } else {
        assert!(markdown.contains("libmagic did not run"));
    }
}

/// The report never claims a number is good, only what it is.
///
/// A regression gate on a shared runner fails at random and gets muted, so there
/// is no threshold to find here. What there is instead is a set of statements the
/// report must keep making, and each of them is a claim a reader would otherwise
/// have to infer.
/// A library that has a startup cost gets a row, whichever library it is.
///
/// The first version of the report printed the startup table only if the
/// *first* library had one. `magical_rs` is first and loads nothing, so
/// libmagic's 15 ms database load and the paragraph about what a database costs
/// never appeared at all -- and the missing table is the one that makes
/// libmagic's per-call number comparable to anything.
#[test]
fn a_startup_cost_is_reported_whoever_has_it() {
    let markdown = report::build(&adapter::all()).to_markdown();
    let with_startup: Vec<&str> = adapter::all()
        .iter()
        .filter(|a| a.startup().is_some())
        .map(|a| a.name())
        .collect();

    if with_startup.is_empty() {
        assert!(
            !markdown.contains("One-time cost"),
            "no library has a startup cost, so the report should not have the section -- and it \
             does, which means the section is not driven by the data"
        );
    } else {
        assert!(
            markdown.contains("One-time cost"),
            "{} reported a startup cost and the report has no section for it: {with_startup:?}",
            with_startup.len(),
        );
        for name in &with_startup {
            assert!(
                markdown.contains(&format!("| `{name}` |")),
                "{name} has a startup cost and no row in the startup table"
            );
        }
    }
}

/// The report keeps saying the things that keep it honest.
///
/// A benchmark's credibility is a set of statements it makes about its own
/// limits, and they rot silently: an edit that tidies a paragraph drops one and
/// nothing fails. These are the statements, and the test is the list.
///
/// The two libmagic paragraphs are conditional because they only exist when
/// libmagic ran. Asserting them unconditionally would have made this test fail
/// on every machine without a C library, which is the fastest way to get a test
/// deleted.
#[test]
fn the_report_states_what_it_cannot_claim() {
    let markdown = report::build(&adapter::all()).to_markdown();

    let mut expected = vec![
        "Not a score",
        "not a coverage claim",
        "Nothing here fails the build",
        "no threshold in the benchmark crate",
    ];
    if cfg!(feature = "libmagic") {
        expected.push("magic_buffer` and `magic_file` are different");
        expected.push("not a faster detector");
        // The entry-point paragraph has to state a count and a consequence
        // either way. The first version asserted "on the WebP file the buffer
        // entry point says nothing", which was true of libmagic 5.47 on Windows
        // and false of 5.45 on the CI runner -- a sentence in a report that is a
        // fact about one machine's library build.
        expected.push("On this build they");
    }

    for claim in expected {
        assert!(
            markdown.contains(claim),
            "the report no longer says {claim:?}, and it used to"
        );
    }
}

/// The report counts the entry-point disagreements rather than naming them.
///
/// libmagic's `magic_buffer` and `magic_file` can disagree about the same bytes,
/// which its man page lists under BUGS. The first version of this report said
/// which file they disagreed about, in a sentence: true of the libmagic 5.47 this
/// crate was developed against on Windows, false of the 5.45 on the Linux CI
/// runner. A sentence like that is a claim about one machine's library build, in
/// a report whose whole argument is that it makes claims about the measurement.
///
/// So the paragraph has to state a count this run produced, and either
/// consequence has to be acceptable.
#[test]
fn the_report_counts_rather_than_names_the_entry_point_disagreements() {
    if !cfg!(feature = "libmagic") {
        return;
    }
    let markdown = report::build(&adapter::all()).to_markdown();

    assert!(
        !markdown.contains("on the WebP file, the buffer entry point"),
        "the report names which file the two libmagic entry points disagree about, which is a \
         fact about the libmagic build it was written against rather than about this run"
    );
    assert!(
        markdown.contains("On this build they agreed on all of them")
            || markdown.contains("On this build they disagreed about"),
        "the report does not say what it counted, so the paragraph is asserting rather than \
         measuring. The report is:\n{markdown}"
    );
}
/// The report does not sort its rows by time.
///
/// A table ordered by the winner is a table that has been arranged, and the
/// arrangement is invisible to anyone who has not seen an earlier version. The
/// row order has to be `adapter::all`'s order, and this is the only place that
/// fact is written down.
#[test]
fn the_report_does_not_sort_rows_by_time() {
    let markdown = report::build(&adapter::all()).to_markdown();
    let order: Vec<usize> = adapter::all()
        .iter()
        .map(|a| {
            markdown
                .find(&format!("| `{}` |", a.name()))
                .unwrap_or(usize::MAX)
        })
        .collect();
    assert!(
        order.windows(2).all(|w| w[0] < w[1]),
        "the libraries appear in the report in the order {order:?}, which is not the order \
         adapter::all puts them in -- the table has been rearranged"
    );
}

/// Every library is measured, and none of them is measured zero times.
///
/// A zero would come from an empty corpus, and an empty corpus produces a table
/// of zeroes that looks like a result.
#[test]
fn no_library_is_measured_zero_times() {
    let built = report::build(&adapter::all());
    for table in [&built.in_memory, &built.from_path] {
        for m in table {
            assert!(m.calls_per_pass > 0, "{} was never called", m.name);
            assert_eq!(
                m.samples.len(),
                9,
                "{} has {} samples, so the median is not over nine passes",
                m.name,
                m.samples.len()
            );
            assert!(
                m.median_ns.is_finite() && m.median_ns > 0.0,
                "{} has a median of {} ns",
                m.name,
                m.median_ns
            );
        }
    }
}

/// The ratio column is a ratio against the slowest library, and it is finite.
///
/// The first version folded with `f64::max` over a list that could hold a NaN,
/// and `f64::max(inf, NaN)` is `inf`, so one bad row made every ratio in the
/// column print as `inf`. That is what the report looked like, and it is
/// unreadable rather than merely wrong.
#[test]
fn the_ratio_column_is_finite() {
    let markdown = report::build(&adapter::all()).to_markdown();
    assert!(
        !markdown.contains("infx"),
        "a ratio in the timing table printed as `inf`, which is what a NaN median does to the \
         fold. The report is:\n{markdown}"
    );
}

/// `Magical` really is the adapter that calls this crate, not a stub.
///
/// Without this, every other test in the file could pass against three recorders
/// and the crate's own library would never have been timed at all.
#[test]
fn the_magical_adapter_calls_the_library() {
    let adapter = Magical;
    let png = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0];
    let answer = adapter.answer(&png);
    assert_eq!(answer.mime.as_deref(), Some("image/png"));
    assert!(adapter.detect(&png) > 0);
    assert_eq!(
        adapter.detect(&[0_u8; 64]),
        0,
        "nothing should be found in noise"
    );
}

/// The report's own corpus and the library's own table cannot drift apart.
///
/// `READ_SIZE` is a written-out constant because the harness needs a length, and
/// this is the assertion that keeps the written number equal to the number the
/// library computes for itself.
#[test]
fn the_corpus_size_is_the_library_s_answer() {
    assert_eq!(
        corpus::READ_SIZE,
        magical_rs::magical::bytes_read::with_bytes_read()
    );
}
/// No corpus is left in the temporary directory once every test here has run.
///
/// The per-guard test above is the one that is exact. This one is the backstop:
/// it runs whatever is left by the fifteen tests in this binary, and its subject
/// is the thing a person actually cares about, which is that a machine does not
/// slowly fill up because a benchmark was run on it.
///
/// It cannot be exact either -- the tests run in parallel, so a corpus belonging
/// to a test that has not finished is legitimately there -- and it is not written
/// to be. The exact assertion is the one that fails; this one is here so that a
/// regression which leaves a directory behind forever cannot pass by accident.
#[test]
fn no_corpus_survives_this_binary() {
    let leftovers = temp_dirs();

    // Every corpus this binary created, from the guard's own naming, and the ones
    // that are gone are the ones that were cleaned up. A leftover belonging to a
    // still-running test is named with a higher counter than the last one this
    // test can see, so the count is the only thing to assert.
    assert!(
        leftovers.len() < 32,
        "{} corpus directories are still in {}: {leftovers:?}. Each holds 10.5 MB, so this is a \
         disk that fills up rather than an error anybody reads. The guard's `Drop` is the thing \
         that removes them.",
        leftovers.len(),
        std::env::temp_dir().display(),
    );
}

/// Temporary directories this crate's harness may have created, that are still there.
fn temp_dirs() -> Vec<PathBuf> {
    std::env::temp_dir()
        .read_dir()
        .map(|entries| {
            entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| {
                    p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| n.starts_with("magical-bench-"))
                })
                .collect()
        })
        .unwrap_or_default()
}
