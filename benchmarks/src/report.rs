//! The numbers, and the table a pull request gets.
//!
//! Criterion does the rigorous timing -- it estimates the noise, it saves a
//! baseline, it tells you whether a change moved anything beyond the noise. This
//! module does a cruder thing on purpose: one number per library, measured the
//! same way for all of them, that can be printed in a table a human reads in a
//! browser.
//!
//! It is a separate measurement rather than a parser over Criterion's output
//! because parsing another tool's human-facing output to produce a number is how
//! a summary ends up quoting a figure that is one format change away from being
//! a panic.
//!
//! **The passes are interleaved, and rotated.** All libraries are measured in
//! round-robin, and the starting library is rotated each pass, so none of them
//! systematically gets the first pass on a cold cache or the last one on a
//! throttled core. Measuring library A entirely and then library B entirely is
//! the obvious way to write this and it quietly charges the second one for
//! whatever the machine did during the first.
//!
//! **No threshold.** Nothing in this file compares a number to a limit, and
//! nothing should be added that does. An absolute nanosecond figure on a shared
//! CI runner is not a property of the library; a gate built on one fails at
//! random and gets muted, which is worse than not having it at all.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::adapter::{Adapter, Answer, Startup};
use crate::corpus::{self, Case};

/// How many timed passes are taken over the whole corpus.
///
/// Nine, because the report quotes a median and a median of three is not one.
/// Not configurable through an environment variable: a benchmark whose number
/// can be made quieter by a variable is a benchmark that will be made quieter.
const PASSES: usize = 9;

/// How many whole-corpus passes run before the first timed one.
///
/// Two. The corpus is 10.5 MB and the first pass through it is dominated by
/// faulting pages in, and one pass is not always enough for the branch predictor
/// to settle on a table that is walked end to end every time. The Criterion
/// benches warm up separately and for longer; this only has to be good enough
/// that the *first* library measured is not the one that pays for the pages.
const WARMUP_PASSES: usize = 2;

/// One library's result for one entry point.
pub struct Measurement {
    /// The library's name.
    pub name: String,
    /// What its own API says a caller must read. See [`Adapter::read_size`].
    pub read_size: String,
    /// The middle of [`Self::samples`].
    pub median_ns: f64,
    /// The smallest of [`Self::samples`], the closest thing here to a floor: it
    /// is the one pass the machine did not interrupt.
    pub best_ns: f64,
    /// The slowest pass, printed so the spread is visible rather than implied.
    pub worst_ns: f64,
    /// Every pass, sorted.
    pub samples: Vec<f64>,
    /// A cost paid before any of the above.
    pub startup: Option<Startup>,
    /// How many times the library was called per pass.
    pub calls_per_pass: usize,
}

impl Measurement {
    /// The slowest pass over the slowest pass: how much worse the worst run was.
    fn spread(&self) -> f64 {
        self.worst_ns / self.median_ns.max(f64::MIN_POSITIVE)
    }
}

/// One real file, and what every library said about it.
pub struct AnswerRow {
    /// The file's name.
    pub file: String,
    /// What this crate's own library says.
    pub expected: String,
    /// One entry per adapter, in [`Adapter`] order.
    pub in_memory: Vec<(String, Answer)>,
    /// One entry per adapter, from the file on disk.
    ///
    /// A second set rather than a second column per library, because the two
    /// differ for exactly one library on one of the four files and squeezing
    /// that into a cell would hide it.
    pub from_path: Vec<(String, Answer)>,
}

/// Everything the summary is made of.
pub struct Report {
    /// `true` when libmagic was compiled in, so the header can say so in words
    /// rather than leaving a reader to count the rows and wonder.
    pub libmagic: bool,
    /// Bytes in every timed buffer.
    pub read_size: usize,
    /// How many buffers a format was planted in.
    pub positives: usize,
    /// How many buffers match nothing.
    pub negatives: usize,
    /// How many table entries use a predicate and so have no case.
    pub predicate_rules: usize,
    /// One per library, for the in-memory entry point.
    pub in_memory: Vec<Measurement>,
    /// One per library, for the from-path entry point.
    pub from_path: Vec<Measurement>,
    /// One per real file.
    pub answers: Vec<AnswerRow>,
    /// How many bytes the answer pass read.
    pub answer_bytes: usize,
}

/// Builds the corpus, measures every adapter, and reads the answers.
///
/// # Panics
///
/// If the temporary directory for the from-path corpus cannot be created or
/// written. Every library in the second table needs a real file to open, and
/// silently measuring nothing would print a table of zeroes next to a table of
/// numbers.
#[must_use]
pub fn build(adapters: &[Box<dyn Adapter>]) -> Report {
    let positives = corpus::positives();
    // Read before the move below, and used again in the struct at the bottom.
    let positive_count = positives.len();
    let negatives = corpus::negatives(positive_count);
    let negative_count = negatives.len();
    let cases: Vec<Case> = positives.into_iter().chain(negatives).collect();

    // The synthetic corpus, on disk, for the from-path table. Written once and
    // removed when `build` returns, so a crashed run leaves one directory behind
    // rather than 10 MB per attempt.
    let on_disk = Written::new(&cases);

    let in_memory = measure(adapters, &cases, &[]);
    let from_path = measure(adapters, &[], &on_disk.paths);

    // The answer pass runs over *real* files, not over the synthetic corpus the
    // timing used. That split is the point rather than an inconsistency: a
    // planted signature in a buffer of noise is a fair thing to time a search on,
    // and it is not a thing to ask two libraries to agree about.
    let real = corpus::real_files();
    let answer_bytes: usize = real.iter().map(|case| case.bytes.len()).sum();
    let answers = real
        .iter()
        .map(|case| {
            let path = corpus::tests_directory().join(&case.name);
            AnswerRow {
                file: case.name.clone(),
                expected: case
                    .expected
                    .map_or_else(|| "nothing".to_owned(), |k| k.variant_name().to_owned()),
                in_memory: adapters
                    .iter()
                    .map(|a| (a.name().to_owned(), a.answer(&case.bytes)))
                    .collect(),
                from_path: adapters
                    .iter()
                    .map(|a| (a.name().to_owned(), a.answer_path(&path)))
                    .collect(),
            }
        })
        .collect();

    Report {
        libmagic: cfg!(feature = "libmagic"),
        read_size: corpus::READ_SIZE,
        positives: positive_count,
        negatives: negative_count,
        predicate_rules: corpus::predicate_rule_entries(),
        in_memory,
        from_path,
        answers,
        answer_bytes,
    }
}

/// The synthetic corpus, written to a temporary directory, and the paths.
///
/// The synthetic corpus, written to disk, and the paths to it.
///
/// Public so the cleanup can be tested directly. The test that used to check it
/// took a snapshot of the temporary directory before and after a whole
/// `build`, which is only sound if nothing else is writing there — and
/// `tests/harness.rs` runs fifteen tests in parallel, several of which build a
/// report. It failed on CI and passed locally, which is the worst way for a test
/// about hygiene to behave. Testing the guard itself has no such problem.
///
/// Dropped when this goes out of scope, which is the whole lifetime management
/// this needs: the report is built in one function and the directory exists for
/// exactly that.
pub struct Written {
    directory: PathBuf,
    paths: Vec<PathBuf>,
}

impl Written {
    /// Writes one file per case, named after the case.
    ///
    /// # Panics
    ///
    /// If the directory or any file cannot be written.
    #[must_use]
    pub fn new(cases: &[Case]) -> Self {
        // A counter, not just the process id. Two `build` calls in one process
        // happen: `build` measures twice, and the tests call it fifteen times.
        // With only the pid they would share one directory, so whichever finished
        // first would `remove_dir_all` the files the other was still opening —
        // and the from-path table would be a table of `magic_file` on a path that
        // no longer exists, which is a number rather than an error.
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let directory =
            std::env::temp_dir().join(format!("magical-bench-{}-{n}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap_or_else(|e| {
            panic!("cannot create {}: {e}", directory.display());
        });

        let mut paths = Vec::with_capacity(cases.len());
        for case in cases {
            let path = directory.join(&case.name);
            std::fs::write(&path, &case.bytes)
                .unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
            paths.push(path);
        }
        Self { directory, paths }
    }

    /// The directory the corpus is in, and the files in it.
    ///
    /// A method rather than two public fields so a caller cannot hold the paths
    /// without the directory that has to be removed alongside them.
    #[must_use]
    pub fn written(&self) -> (&Path, &[PathBuf]) {
        (&self.directory, &self.paths)
    }
}

impl Drop for Written {
    fn drop(&mut self) {
        // Best effort, and a failure to clean up is not worth a panic in a
        // `Drop` during normal unwinding.
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

/// Times every adapter over the corpus, in whole-corpus passes.
///
/// # The fairness problem, and what is done about it
///
/// The obvious shape is: time library A over the whole corpus, then B, then C.
/// The second version rotates the order so no library is always first. Both are
/// unfair, and the reason is the factor of 7000 between libmagic and this
/// crate.
///
/// The corpus is 10.5 MB. One libmagic pass walks all of it, taking about 115 ms,
/// so by the time the next pass starts the fast library's buffers are no longer
/// in cache and it is charged for the slow library's memory traffic. Measured
/// that way this file reported 177 ns for `magical_rs` where Criterion measured
/// 57 ns for the same work.
///
/// The fix is not to interleave more finely. Interleaving per call was tried and
/// is worse: `Instant::now()` twice per call is tens of nanoseconds on Windows,
/// against a 57 nanosecond measurement, and `Instant`'s resolution there is
/// coarse enough that the subtraction does not recover what the quantisation
/// lost. The number came out as 397 ns, which is neither.
///
/// The fix is to equalise the starting state instead: [`warm`] reads the whole
/// corpus immediately before every timed pass, so every library starts its pass
/// with the corpus in cache. libmagic's own pass also starts warm. Nobody is
/// charged for anybody else's memory traffic, and the clock is read twice per
/// pass rather than twice per call.
///
/// # What is left
///
/// The warm read is sequential; the libraries' own access is not. A library that
/// touches 50 bytes of each 36,870-byte buffer leaves most of the corpus out of
/// cache by the time it comes back around, and it does so at its own rate. That
/// is a real property of the library and it is left in.
///
/// # Panics
///
/// Panics if neither `cases` nor `paths` is non-empty, or if both are.
fn measure(adapters: &[Box<dyn Adapter>], cases: &[Case], paths: &[PathBuf]) -> Vec<Measurement> {
    assert!(
        cases.is_empty() != paths.is_empty(),
        "measure() needs either cases or paths, not both and not neither"
    );
    let calls = f64::from(u32::try_from(case_count(cases, paths)).unwrap_or(u32::MAX));

    for _ in 0..WARMUP_PASSES {
        for adapter in adapters {
            warm(cases);
            sink_of(pass(adapter.as_ref(), cases, paths));
        }
    }

    let mut samples: Vec<Vec<f64>> = vec![Vec::with_capacity(PASSES); adapters.len()];
    for round in 0..PASSES {
        // Rotation rather than a fixed order, so no library systematically gets
        // the first pass on whatever state the machine was in beforehand.
        for step in 0..adapters.len() {
            let index = (round + step) % adapters.len();
            warm(cases);
            samples[index].push(pass(adapters[index].as_ref(), cases, paths) / calls);
        }
    }

    adapters
        .iter()
        .zip(samples)
        .map(|(adapter, mut taken)| {
            taken.sort_by(f64::total_cmp);
            Measurement {
                name: adapter.name().to_owned(),
                read_size: adapter.read_size(),
                median_ns: taken[taken.len() / 2],
                best_ns: taken[0],
                worst_ns: taken[taken.len() - 1],
                samples: taken,
                startup: adapter.startup(),
                calls_per_pass: case_count(cases, paths),
                // Not measured, because nothing is measured: a whole pass is two
                // clock reads, so the cost of the clock is not in the number. The
                // field is gone rather than left at zero, which would be a
                // claim.
            }
        })
        .collect()
}

/// How many calls one pass makes.
const fn case_count(cases: &[Case], paths: &[PathBuf]) -> usize {
    if cases.is_empty() {
        paths.len()
    } else {
        cases.len()
    }
}

/// Reads every buffer, so the next pass starts with the corpus in cache.
///
/// Not `black_box` on the sum alone: the point is the reads, and the sum is
/// what stops the optimiser from deleting them.
fn warm(cases: &[Case]) {
    let mut seen = 0_u64;
    for case in cases {
        // One byte every 64, so the pass touches every cache line without
        // reading 10.5 MB. A full read would work too and would take about a
        // millisecond; this takes about two hundred microseconds and warms the
        // same lines.
        let mut at = 0;
        while at < case.bytes.len() {
            seen += u64::from(case.bytes[at]);
            at += 64;
        }
    }
    std::hint::black_box(seen);
}

/// One timed pass over the whole corpus, in nanoseconds.
///
/// Per call, not per pass: this is the number `measure` divides by the call
/// count. Accumulating and reading the clock once at the end is what keeps the
/// clock out of the measurement -- 286 clock reads per pass would be 286 times
/// the cost of the two it actually needs.
fn pass(adapter: &dyn Adapter, cases: &[Case], paths: &[PathBuf]) -> f64 {
    let mut sink = 0_usize;
    let started = Instant::now();
    if paths.is_empty() {
        for case in cases {
            sink += adapter.detect(&case.bytes);
        }
    } else {
        for path in paths {
            sink += adapter.detect_path(path.as_path());
        }
    }
    let elapsed = started.elapsed().as_secs_f64();
    // Without this the optimiser is entitled to delete the calls, and the whole
    // report becomes a measurement of an empty loop.
    sink_of(sink);
    elapsed * 1e9
}

/// Consumes a value the optimiser would otherwise be free to discard.
///
/// Generic so that both the integer sum of answers and the `f64` nanoseconds can
/// go through it, rather than casting one to the other's type to satisfy a
/// signature.
#[inline]
fn sink_of<T>(value: T) {
    std::hint::black_box(value);
}

impl Report {
    /// The summary, as GitHub-flavoured markdown.
    ///
    /// Written for a pull request comment, so: no HTML, no nested lists, and
    /// every table carries a caption saying what it is not.
    #[must_use]
    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        self.write_header(&mut out);
        Self::write_timing(&mut out, "in memory", &self.in_memory);
        Self::write_timing(&mut out, "from a file on disk", &self.from_path);
        self.write_startup(&mut out);
        self.write_answers(&mut out);
        out.push_str(
            "\nNothing here fails the build, and there is no threshold in the benchmark crate to \
             fail. A nanosecond figure on a shared runner is a property of the runner, and a gate \
             built on one is muted within a month -- and a muted gate looks exactly like a passing \
             one.\n",
        );
        out
    }

    /// The title, the two corpora, and which libraries actually ran.
    ///
    /// The "which libraries ran" line comes before any number on purpose. A table
    /// with three rows when two libraries are compiled in reads as a complete
    /// result, and a reader who has to count the rows to notice is a reader who
    /// will eventually not count them.
    fn write_header(&self, out: &mut String) {
        out.push_str("## Detection benchmarks\n\n");
        let _ = writeln!(
            out,
            "**Timed on** {} synthetic buffers of {} bytes -- {} with a format planted at a \
             declared signature and offset, {} matching nothing. {} table entries are matched by a \
             predicate rather than a byte string and have no case. **Compared on** the {} real \
             files this repository already commits, {} bytes in total.",
            self.positives + self.negatives,
            self.read_size,
            self.positives,
            self.negatives,
            self.predicate_rules,
            self.answers.len(),
            self.answer_bytes,
        );
        out.push('\n');
        out.push_str(
            "Two corpora on purpose. What a search costs does not depend on what is in the buffer, \
             so timing runs on generated ones, and 286 of them is 286 times more evidence than \
             four. What two libraries *answer* absolutely does, so that runs on files a program \
             actually produced -- a planted PNG signature in 36 KB of noise is not a PNG, and \
             libmagic says `application/octet-stream` about it, correctly.\n\n",
        );
        out.push_str(if self.libmagic {
            "All three libraries ran.\n\n"
        } else {
            "**libmagic did not run.** It is behind this crate's `libmagic` feature, which is off \
             because `magic-sys`'s build script fails rather than degrades when it cannot find \
             the C library. Two rows below is two rows, not a result.\n\n"
        });
    }

    /// One timing table.
    ///
    /// The from-path table gets its own sentence underneath, because the two are
    /// not the same measurement and the second is mostly I/O. Takes the rows
    /// rather than reading a field, because the two tables are separate
    /// measurements and reading `self` in here would suggest they are two views of
    /// one.
    fn write_timing(out: &mut String, title: &str, rows: &[Measurement]) {
        let _ = writeln!(out, "### Time to identify one buffer, {title}\n");
        out.push_str(
            "| Library | Header size its own API asks for | Median | Best pass | Spread | \
             vs slowest |\n|---|---|---:|---:|---:|---:|\n",
        );

        // The slowest median, so each row can be printed as a ratio. Sorting the
        // rows by this would be arranging them, and the order they are in is the
        // order `adapter::all` puts them in: this crate first, which is a choice
        // a reader is entitled to notice.
        //
        // `is_finite` before the fold rather than `f64::max` alone, because a NaN
        // anywhere would make the whole column infinite and every ratio in it
        // would print as `inf` -- which is what the first version of this file
        // did, and which a reader cannot interpret. A library that produced a NaN
        // deserves to see its own NaN in its own row.
        let slowest = rows
            .iter()
            .map(|m| m.median_ns)
            .filter(|n| n.is_finite())
            .fold(0.0_f64, f64::max);

        for m in rows {
            let ratio = if m.median_ns > 0.0 && slowest > 0.0 {
                format!("{:.1}x", slowest / m.median_ns)
            } else {
                "-".to_owned()
            };
            let _ = writeln!(
                out,
                "| `{}` | {} | {} | {} | {} | {} |",
                m.name,
                m.read_size,
                ns(m.median_ns),
                ns(m.best_ns),
                times(m.spread()),
                ratio,
            );
        }

        out.push_str(
            "\nThe same buffers, the same order, the same number of calls, whole-corpus passes with \
             the order rotated and the corpus read into cache before each one. That last part is \
             the fairness fix and it is not optional: libmagic is about 7000 times slower per call \
             than this crate, so a pass from it evicts the 10.5 MB corpus, and timing without the \
             warm read charged this crate 177 ns where Criterion measures 57 ns for the same \
             work. The difference was the slow library's memory traffic.\n",
        );
        if title == "from a file on disk" {
            out.push_str(
                "\nThis table includes opening and reading each file, so it is dominated by I/O for \
                 all three and the gap between them is mostly how much each one reads -- \
                 `magical_rs` reads 36,870 bytes because that is what its table needs, and the \
                 other two read what they need. It is the number a caller holding a path actually \
                 pays, and it is the one to quote when the caller holds a path.\n",
            );
        }
    }

    /// The one-time costs, in their own table.
    ///
    /// Separate because a millisecond figure in a table of nanoseconds reads as a
    /// rounding error rather than as a different kind of thing, and a library
    /// with a 15 ms startup next to one with none should not look like the faster
    /// library.
    fn write_startup(&self, out: &mut String) {
        // Every library that has one, not the first one that has one. Checking
        // only the first is what the first version did, and `magical_rs` is
        // first and has no startup cost, so libmagic's 15 ms database load and
        // the paragraph explaining what it means never appeared at all.
        let startups: Vec<&Measurement> = self
            .in_memory
            .iter()
            .filter(|m| m.startup.is_some())
            .collect();
        if startups.is_empty() {
            return;
        }

        out.push_str("\n### One-time cost, before the first call\n\n");
        out.push_str("| Library | What it does | Time | Note |\n|---|---|---:|---|\n");
        for m in &startups {
            let Some(startup) = &m.startup else {
                continue;
            };
            let _ = writeln!(
                out,
                "| `{}` | {} | {} | {} |",
                m.name,
                startup.what,
                ms(startup.millis),
                startup.note.as_deref().unwrap_or("-")
            );
        }
        out.push_str(
            "\nOnly libmagic has one. A library that loads nothing is not a faster detector -- it \
             is a library with nothing to load, and 10 MB of rules is the honest cost of \
             libmagic's coverage. Paid once, and in neither table above.\n",
        );
    }

    /// What each library answered, from memory and from a path.
    ///
    /// All four files, always, with no filtering. A cap here would be a way to
    /// choose which disagreements a reader sees, which is the one thing this
    /// table must not do.
    fn write_answers(&self, out: &mut String) {
        out.push_str(
            "\n### What each library said about each real file\n\n\
             **Not a score, and not a coverage claim.** Four files is not a corpus, and they were \
             committed to check a signature table rather than to be representative. What is here is \
             a disagreement list, which is worth having: three libraries reading the same bytes and \
             naming three different things is the sort of thing only visible when someone prints \
             all three columns.\n\n",
        );

        for (heading, from_path) in [("from the bytes in memory", false), ("from the path", true)] {
            let cells: Vec<&[(String, Answer)]> = self
                .answers
                .iter()
                .map(|row| {
                    if from_path {
                        &row.from_path[..]
                    } else {
                        &row.in_memory[..]
                    }
                })
                .collect();
            let agreed = cells.iter().filter(|c| all_agree(c)).count();

            let _ = writeln!(out, "**{heading}**\n");
            let _ = writeln!(
                out,
                "{agreed} of {} files: every library that named a format named the same MIME.\n",
                self.answers.len()
            );
            out.push_str("| File | This crate says |");
            for m in &self.in_memory {
                let _ = write!(out, " `{}` |", m.name);
            }
            out.push_str(" Agree |\n|---|---|");
            for _ in &self.in_memory {
                out.push_str("---|");
            }
            out.push_str("---|\n");

            for (row, answers) in self.answers.iter().zip(&cells) {
                let _ = write!(out, "| {} | {} |", row.file, row.expected);
                for (_, answer) in *answers {
                    let _ = write!(out, " {} |", cell(answer));
                }
                let _ = writeln!(out, " {} |", if all_agree(answers) { "yes" } else { "no" });
            }
            out.push('\n');
        }

        self.write_entry_point_difference(out);

        out.push_str(
            "A `-` means the library named no format. `application/octet-stream` is libmagic \
             saying exactly that, and it is shown as `-` rather than passed through, because \
             \"I do not know this\" and \"this is binary data\" are different things for a caller to \
             have to handle next.\n",
        );
    }

    /// How many files libmagic answers differently depending on how it was given them.
    ///
    /// **Measured, not asserted.** The first version of this said "on the WebP
    /// file, the buffer entry point says nothing and the path entry point says
    /// `image/webp`", which was true of the libmagic 5.47 this crate was developed
    /// against and false of the 5.45 on the CI runner. A sentence in a report that
    /// is a fact about one machine's library build is a claim, and this report's
    /// whole argument is that it makes claims about the measurement rather than
    /// about the author.
    ///
    /// So it prints the number it counted. On a build where the two agree, that is
    /// zero, and the zero is the finding.
    ///
    /// libmagic's own `libmagic(3)` man page lists the underlying behaviour under
    /// BUGS: *"The results from `magic_buffer()` and `magic_file()` where the buffer
    /// and the file contain the same data can produce different results, because in
    /// the `magic_file()` case, the program can `lseek(2)` and `stat(2)` the file
    /// descriptor."* That is why both are asked at all rather than one of them.
    fn write_entry_point_difference(&self, out: &mut String) {
        // The last adapter is the one with a path-based entry point, so it is the
        // one whose two answers can differ.
        let Some(libmagic) = self.in_memory.last().map(|m| m.name.as_str()) else {
            return;
        };
        let differing: Vec<&str> = self
            .answers
            .iter()
            .filter(|row| {
                let (Some((_, from_bytes)), Some((_, from_path))) =
                    (row.in_memory.last(), row.from_path.last())
                else {
                    return false;
                };
                from_bytes.mime != from_path.mime
            })
            .map(|row| row.file.as_str())
            .collect();

        let _ = writeln!(
            out,
            "\n**`{libmagic}` is asked twice, on purpose.** `magic_buffer` and `magic_file` are \
             different code paths inside libmagic, and libmagic's `libmagic(3)` man page lists it \
             under BUGS that they can disagree on the same bytes, because the path case can \
             `lseek(2)` and `stat(2)` the descriptor and the buffer case cannot. A benchmark that \
             had picked one would have picked whichever made libmagic look better, so both are \
             above.",
        );
        if differing.is_empty() {
            out.push_str(
                "\nOn this build they agreed on all of them. That is a fact about this build, not a \
                 correction of the man page.\n",
            );
        } else {
            let _ = writeln!(
                out,
                "\nOn this build they disagreed about {}: {}. A different libmagic version may \
                 disagree about different files, or about none.",
                differing.len(),
                differing.join(", ")
            );
        }
    }
}

/// A time in nanoseconds, as a table cell.
fn ns(value: f64) -> String {
    if !value.is_finite() {
        return value.to_string();
    }
    if value >= 1_000.0 {
        format!("{:.1} us", value / 1_000.0)
    } else {
        format!("{value:.0} ns")
    }
}

/// A time in milliseconds, as a table cell.
fn ms(value: f64) -> String {
    format!("{value:.1} ms")
}

/// A ratio, as a table cell.
fn times(value: f64) -> String {
    if value.is_finite() {
        format!("{value:.1}x")
    } else {
        value.to_string()
    }
}

/// Whether every library that named a format named the same MIME type.
///
/// A library that named nothing is not counted as agreeing, because the
/// interesting disagreement is "one of them found a format and the others did
/// not" and counting silence as consensus would hide exactly that.
fn all_agree(cells: &[(String, Answer)]) -> bool {
    let mut mimes = cells.iter().filter_map(|(_, a)| a.mime.as_deref());
    let Some(first) = mimes.next() else {
        // Nobody named a format: vacuously consistent.
        return true;
    };
    mimes.all(|m| m == first)
}

/// One answer, as a table cell.
fn cell(answer: &Answer) -> String {
    match (&answer.mime, &answer.extension) {
        (None, _) => "-".to_owned(),
        (Some(mime), None) => mime.clone(),
        (Some(mime), Some(extension)) => format!("`{mime}` `{extension}`"),
    }
}
