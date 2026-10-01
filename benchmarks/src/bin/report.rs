//! Prints the pull-request summary, and can write it to a file.
//!
//! Two reasons this is a binary rather than something the benchmark target does
//! on the side: the table has to be readable without `cargo bench` having run,
//! and the step summary wants a file rather than whatever the last benchmark
//! happened to print.
//!
//! ```text
//! cargo run --release --bin report            # to stdout
//! cargo run --release --bin report -- out.md  # to a file
//! ```

use magical_benchmarks::{adapter, report};

fn main() {
    let adapters = adapter::all();
    let report = report::build(&adapters);
    let markdown = report.to_markdown();

    match std::env::args().nth(1) {
        Some(path) => {
            // Written with `fs::write` and reported as an error rather than
            // unwrapped: a truncated summary in a pull request looks exactly
            // like a short summary, and one of those is a bug report about the
            // benchmark.
            if let Err(error) = std::fs::write(&path, &markdown) {
                eprintln!("cannot write {path}: {error}");
                std::process::exit(1);
            }
            println!("wrote {path} ({} bytes)", markdown.len());
        }
        None => print!("{markdown}"),
    }
}
