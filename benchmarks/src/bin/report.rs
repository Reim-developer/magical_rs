//! Prints the pull-request summary, and writes it to a file when given a path.
//!
//! Both, always: see `main`. Two reasons this is a binary rather than something
//! the benchmark target does on the side are that the table has to be readable
//! without `cargo bench` having run, and that the step summary wants a file
//! rather than whatever the last benchmark happened to print.
//!
//! ```text
//! cargo run --release --bin report            # to stdout
//! cargo run --release --bin report -- out.md  # to stdout, and to out.md
//! ```

use magical_benchmarks::{adapter, report};

fn main() {
    let adapters = adapter::all();
    let report = report::build(&adapters);
    let markdown = report.to_markdown();

    // Written *and* printed, never one or the other.
    //
    // A caller has two possible reasons for running this: a terminal, where the
    // markdown is the point, and CI, where a file is what `$GITHUB_STEP_SUMMARY`
    // can be appended to. Writing only means the `make bench-report` recipe needs
    // a `cat` after it, and `cat` is not on every machine that has a C toolchain
    // -- which is exactly the set of machines that run this. Printing only means
    // CI has to capture stdout, and a step that pipes this into a file is a step
    // whose failure is a truncated file rather than an error.
    if let Some(path) = std::env::args().nth(1) {
        // Reported as an error rather than unwrapped: a truncated summary in a
        // pull request looks exactly like a short summary, and one of those is a
        // bug report about the benchmark.
        if let Err(error) = std::fs::write(&path, &markdown) {
            eprintln!("cannot write {path}: {error}");
            std::process::exit(1);
        }
        eprintln!("wrote {path} ({} bytes)", markdown.len());
    }
    print!("{markdown}");
}
