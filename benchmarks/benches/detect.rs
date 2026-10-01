//! The rigorous timing, one library and one entry point at a time.
//!
//! This is the half of the crate that knows what it is doing: Criterion estimates
//! the measurement noise, saves a baseline, and will tell you that a 4% change
//! is indistinguishable from a run on a busier machine. The report in
//! `src/report.rs` is the half that produces a table a person can read, and this
//! file is the one to believe when the two disagree.
//!
//! Four groups, because these are four different costs and averaging them would
//! hide all four:
//!
//! - `single` -- one named format, so a change to one entry's cost is visible
//!   rather than diluted across 286.
//! - `negative` -- a buffer matching nothing. This is the case the dispatch index
//!   exists for, and it is the one that separates an indexed search from a linear
//!   one; a positive case at offset 0 finds its entry immediately and the index is
//!   not exercised at all.
//! - `corpus` -- every buffer, from memory. The headline number.
//! - `from-file` -- every buffer, written to disk and opened by the library. The
//!   I/O-inclusive number.
//!
//! The adapters are called through `&dyn Adapter`, so all three libraries go
//! through one indirect call and none of them gets inlined into the harness while
//! the others do not.

use std::hint::black_box;
use std::path::PathBuf;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use magical_benchmarks::adapter::all;
use magical_benchmarks::corpus;

/// Formats the single-format group is measured on, and why each one is here.
///
/// `Png` and `Jpg` match on their first byte, `Gzip` and `PkgZip` are what a
/// caller actually asks about, and `Tar` and `ISO` have their signatures at
/// offsets 257 and 36,865 -- the two cases where a table that ignored offsets
/// would give a different answer, and therefore the two where the index has to do
/// something.
const FORMATS: [&str; 6] = [
    "Png@0",
    "Jpg@0",
    "Gzip@0",
    "PkgZip@0",
    "Tar@257",
    "ISO@36865",
];

/// The synthetic corpus, on disk, for the from-file group.
///
/// Written once per run into a directory named after the process, and removed
/// when the returned guard is dropped. Criterion keeps running the closure for
/// seconds, so the files have to outlive the first `bench_function` and cannot be
/// written inside it.
struct OnDisk {
    directory: PathBuf,
    paths: Vec<PathBuf>,
}

impl OnDisk {
    /// Writes one file per `(name, bytes)` pair.
    ///
    /// Takes pairs rather than a `&[Case]` because the caller holds references
    /// into two different vectors -- the positives and the negatives -- and
    /// collecting 286 owned `Case`s just to hand them over would copy 10 MB for
    /// no reason.
    ///
    /// # Panics
    ///
    /// If the directory or any file cannot be written.
    fn new<'a>(cases: impl IntoIterator<Item = (&'a str, &'a [u8])>) -> Self {
        let directory =
            std::env::temp_dir().join(format!("magical-bench-criterion-{}", std::process::id()));
        std::fs::create_dir_all(&directory)
            .unwrap_or_else(|e| panic!("cannot create {}: {e}", directory.display()));
        let mut paths = Vec::new();
        for (name, bytes) in cases {
            let path = directory.join(name);
            std::fs::write(&path, bytes)
                .unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
            paths.push(path);
        }
        Self { directory, paths }
    }
}

impl Drop for OnDisk {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

fn benches(c: &mut Criterion) {
    let adapters = all();
    let positives = corpus::positives();
    let negatives = corpus::negatives(positives.len());
    let read_size = corpus::READ_SIZE;
    let cases: Vec<&corpus::Case> = positives.iter().chain(&negatives).collect();

    // Each group in its own block. `BenchmarkGroup` has a `Drop` that flushes,
    // and a `Drop` at the end of the whole function means the first group is
    // still alive -- and holding its bookkeeping -- while the last one runs.
    // `finish()` already does the flush; the block lets the group be gone before
    // the next one starts.

    // -- one format, from memory -------------------------------------------
    {
        let mut single = c.benchmark_group("single");
        single.throughput(Throughput::Bytes(read_size as u64));
        for name in FORMATS {
            let Some(case) = positives.iter().find(|case| case.name == name) else {
                panic!("the corpus has no case named {name}, so this group is stale");
            };
            for adapter in &adapters {
                single.bench_with_input(
                    BenchmarkId::new(adapter.name(), name),
                    &case.bytes,
                    |b, bytes| b.iter(|| adapter.detect(black_box(bytes))),
                );
            }
        }
        single.finish();
    }

    // -- nothing matches ----------------------------------------------------
    // Two buffers, each measured on its own so Criterion can estimate its noise
    // separately rather than averaging them.
    {
        let mut negative = c.benchmark_group("negative");
        negative.throughput(Throughput::Bytes(read_size as u64));
        for case in negatives.iter().take(2) {
            for adapter in &adapters {
                negative.bench_with_input(
                    BenchmarkId::new(adapter.name(), &case.name),
                    &case.bytes,
                    |b, bytes| b.iter(|| adapter.detect(black_box(bytes))),
                );
            }
        }
        negative.finish();
    }

    // -- the whole corpus, from memory --------------------------------------
    {
        let mut whole = c.benchmark_group("corpus");
        whole.throughput(Throughput::Bytes((read_size * cases.len()) as u64));
        for adapter in &adapters {
            whole.bench_function(adapter.name(), |b| {
                b.iter(|| {
                    let mut sink = 0_usize;
                    for case in &cases {
                        sink += adapter.detect(black_box(&case.bytes));
                    }
                    black_box(sink)
                });
            });
        }
        whole.finish();
    }

    // -- the whole corpus, from disk ----------------------------------------
    // I/O inside the measurement, on purpose. This is what a caller with a path
    // pays, and the report prints it as its own table for the same reason.
    //
    // The guard is declared here rather than at the top of the function so that
    // the 10.5 MB of files exists for exactly as long as this group runs.
    let on_disk = OnDisk::new(
        cases
            .iter()
            .map(|case| (case.name.as_str(), case.bytes.as_slice())),
    );
    {
        let mut from_file = c.benchmark_group("from-file");
        from_file.throughput(Throughput::Bytes((read_size * cases.len()) as u64));
        for adapter in &adapters {
            from_file.bench_function(adapter.name(), |b| {
                b.iter(|| {
                    let mut sink = 0_usize;
                    for path in &on_disk.paths {
                        sink += adapter.detect_path(black_box(path.as_path()));
                    }
                    black_box(sink)
                });
            });
        }
        from_file.finish();
    }
}

criterion_group!(group, benches);
criterion_main!(group);
