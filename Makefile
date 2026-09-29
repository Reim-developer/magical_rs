# The `test`, `linter` and `fmt` targets above are deliberately unscoped. The
# root is a virtual workspace now, so they cover every crate in it, which is how
# the CLI comes to be tested and linted at all without a target of its own. Only
# the cross-compiled targets are scoped, and each of those says why in place.
.PHONY: test linter fmt test-dyn test-unsafe test-nostd build-nostd build-wasm examples

test:
	@cargo test

# Separate from `linter` on purpose. Clippy answers "is this code wrong" and
# rustfmt answers "is this written the way the tool writes it", so a failure
# here is one `cargo fmt` away and not a thought. Keeping them apart means a
# formatting failure is never mistaken for a lint failure, and neither is
# hidden inside the other.
#
# This is a gate rather than a chore because nothing was enforcing it. Until it
# existed, 17 rustfmt diffs sat in `src/` and `tests/` unseen, because no
# target in this file and no step in crate_dev.yml ran rustfmt. A format rule
# nothing checks is a preference.
fmt:
	@cargo fmt --check

linter:
	@cargo clippy \
    --all-targets \
    --all-features \
    -- -D clippy::all\
    -D clippy::pedantic\
    -D clippy::nursery\
    -D clippy::perf

test-dyn:
	@cargo test --features magical_dyn

test-unsafe:
	@cargo test --features unsafe_context

# The `no_std` claim is the one this crate makes that a default-features test
# run cannot check at all, so it gets its own target rather than being assumed.
#
# `-p magical_rs` rather than the whole workspace, and the reason is the CLI.
# `magical-file` is a filesystem program: it reads `std::env::args` and calls
# `std::fs`, and neither exists on `thumbv7em-none-eabi`, so an unscoped build
# would fail on the CLI and say nothing about whether the library still honours
# the claim. Scoping it to the library keeps the target answering the question
# it was written to answer.
build-nostd:
	@rustup target add thumbv7em-none-eabi
	@cargo build -p magical_rs --no-default-features --target thumbv7em-none-eabi

# Doctests are excluded here, and that is deliberate rather than convenient.
# The readme's quick start and the `bytes_read` examples all call
# `read_file_header`, which is `#[cfg(feature = "std")]`, so a no_std doctest
# run cannot compile them. Making them compile would mean either a second
# readme or hidden `cfg` scaffolding in the one the reader sees. The cross
# compile in `build-nostd` is what actually proves the `no_std` build works;
# this target proves the rest of the suite still passes without `std`.
test-nostd:
	@cargo test -p magical_rs --no-default-features --lib --tests
	@$(MAKE) build-nostd

# The readme's first paragraph claims this crate works in WebAssembly builds,
# and nothing was checking it. `build-nostd` does not cover that claim:
# thumbv7em-none-eabi is a bare-metal ARM target, and one target compiling says
# nothing about a different one.
#
# The teeth here are narrower than the target name suggests, which is worth
# stating rather than letting a reader assume otherwise. wasm32-unknown-unknown
# ships a `std` whose file-system, process, network and thread calls all
# *compile* and then fail at runtime. Measured, all of these compile clean for
# wasm32: `std::process::Command`, `std::net::TcpStream`, `std::thread::spawn`,
# `std::env::var`, `std::time::SystemTime::now`, and `AtomicU64::fetch_add`.
# So this build catches much less than "it builds for wasm" sounds like.
#
# What it does catch is target-gated `std`: `std::os::unix` and
# `std::os::windows` resolve on a host that is one of them and not on wasm32.
# That is a real class and a small one, and it is the class a contributor adding
# platform code to the one `std` file in this crate would land in.
#
# It is the default-features build for the same reason. A `--no-default-features`
# one would be strictly weaker: wasm32 has a `std` shim, so it cannot catch a
# stray `std` in a core path either. `build-nostd` catches that, against a
# target with no `std` at all. Between them the crate's `std` half is compiled
# for wasm exactly once, and this is it.
#
# `-p magical_rs` again because of the CLI. It would *compile* here, which is the
# problem rather than the reassurance: wasm32's `std` shim declares `std::env`
# and `std::fs` and then fails them at runtime, so a green build of a
# command-line program for wasm says that nothing and reads like evidence. The
# claim this target checks is the library's, so it builds the library.
build-wasm:
	@rustup target add wasm32-unknown-unknown
	@cargo build -p magical_rs --target wasm32-unknown-unknown

# Run, not build. Every directory under `examples/` is its own crate, and cargo
# only treats `examples/*.rs` and `examples/*/main.rs` as example targets of the
# root package, so `--all-targets` in `linter` never reaches a tree laid out as
# `examples/<name>/src/main.rs`. Nothing here was compiled by anything until this
# target existed.
#
# That is not a theoretical gap. The out-of-bounds read in `unsafe_context`
# compiled cleanly the whole time; only running it, or Miri, showed it. And
# `normal_usage` depended on `magical_rs = "0.1.2"` from crates.io, so the most
# ordinary example in the repository demonstrated a release from 2024 while
# sitting two directories from the crate. Both are invisible to a build and
# obvious to a run, which is why this runs them.
#
# The `cd` is per example rather than once, because `normal_usage` opens
# `img/1.png` by relative path and has to be running from its own directory.
examples:
	@set -e; for dir in examples/*/; do \
		echo "  $$dir"; \
		( cd "$$dir" && cargo run --quiet ); \
	done
