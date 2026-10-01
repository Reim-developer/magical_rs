# The `test`, `linter` and `fmt` targets above are deliberately unscoped. The
# root is a virtual workspace now, so they cover every crate in it, which is how
# the CLI comes to be tested and linted at all without a target of its own. Only
# the cross-compiled targets are scoped, and each of those says why in place.
.PHONY: test linter fmt test-dyn test-unsafe test-fluent test-nostd build-nostd build-wasm examples bench bench-report bench-mutations

# Whether this machine has a libmagic the benchmark crate can find, as a cargo
# feature list.
#
# Defined here rather than next to `bench` because `make linter` needs it too: the
# `libmagic` branch of the benchmark crate is half the code in `adapter.rs`, and
# a branch that is only compiled on the machines that happen to have a C library
# is a branch that rots everywhere else.
#
# `pkg-config` first because that is what `magic-sys` tries first, and `$VCPKG_ROOT`
# second because that is what it tries second. `$VCPKGRS_TRIPLET` is deliberately
# not consulted: the probe only has to decide whether to *try*, and the crate does
# the rest. `benchmarks/README.md` has the per-platform instructions, including the
# three variables the `vcpkg` Rust crate reads and the two it ignores.
BENCH_FEATURES := $(shell \
	if pkg-config --exists libmagic 2>/dev/null; then echo libmagic; \
	elif [ -n "$$VCPKG_ROOT" ] && [ -f "$$VCPKG_ROOT/.vcpkg-root" ]; then echo libmagic; \
	else echo ""; fi)

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
#
# `benchmarks/` is formatted separately because it is not a workspace member --
# `cargo fmt --check` at the root walks the root workspace and never looks at an
# excluded crate, so without this line a benchmark file could sit unformatted
# forever. The `--manifest-path` is what reaches it.
fmt:
	@cargo fmt --check
	@cargo fmt --check --manifest-path benchmarks/Cargo.toml

linter:
	@cargo clippy \
    --all-targets \
    --all-features \
    -- -D clippy::all\
    -D clippy::pedantic\
    -D clippy::nursery\
    -D clippy::perf
	@cargo clippy \
    --manifest-path benchmarks/Cargo.toml \
    --all-targets \
    -- -D clippy::all\
    -D clippy::pedantic\
    -D clippy::nursery\
    -D clippy::perf
	# The same crate with libmagic on, because a conditional compilation branch
	# that is never compiled is a branch that rots. Gated on the probe rather
	# than unconditional: `magic-sys`'s build script fails rather than degrading
	# when it cannot find the C library, so a `make linter` that compiled that
	# branch on a machine without libmagic would be red for a reason that has
	# nothing to do with the code.
	@if [ -n "$(BENCH_FEATURES)" ]; then \
		cargo clippy --manifest-path benchmarks/Cargo.toml --all-targets --features libmagic \
			-- -D clippy::all -D clippy::pedantic -D clippy::nursery -D clippy::perf; \
	else \
		echo "  benchmarks: no libmagic found, so the libmagic branch of the linter was skipped"; \
	fi

test-dyn:
	@cargo test --features magical_dyn

test-unsafe:
	@cargo test --features unsafe_context

# `magical_fluent` is a separate target for the same reason `unsafe_context` is.
# The methods are behind a flag, so the code they wrap is only ever compiled when
# somebody asks for the flag — and `make test` runs with default features, which do
# not include it. Without this the flag's own code would be built by `linter`
# (clippy runs --all-features) and never run by anything, which is the exact shape
# of gap `test-unsafe` was added to close for `unsafe_context`.
test-fluent:
	@cargo test --features magical_fluent

# The `no_std` claim is the one this crate makes that a default-features test
# run cannot check at all, so it gets its own target rather than being assumed.
build-nostd:
	@rustup target add thumbv7em-none-eabi
	@cargo build --no-default-features --target thumbv7em-none-eabi
	# `magical_fluent` too, because a feature that only ever compiles against
	# `std` would not be gated for anything. The second build is a separate
	# invocation rather than a feature flag on the first: with the flag on the
	# first, the second would prove nothing about the un-gated build.
	@cargo build --no-default-features --features magical_fluent --target thumbv7em-none-eabi

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
build-wasm:
	@rustup target add wasm32-unknown-unknown
	@cargo build --target wasm32-unknown-unknown

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
		if [ ! -f "$$dir/src/main.rs" ]; then continue; fi; \
		echo "  $$dir"; \
		( cd "$$dir" && cargo run --quiet ); \
	done
	@echo "  examples/wasm_rust/"
	@rustup target add wasm32-unknown-unknown
	@cargo build --release --target wasm32-unknown-unknown --manifest-path examples/wasm_rust/Cargo.toml
	@node examples/wasm_rust/load.mjs

# The benchmarks, against `magical_rs`, `infer` and libmagic.
#
# `benchmarks/` is outside the root workspace, so it is `--manifest-path` rather
# than `-p`, and the reason it is outside is in its own `Cargo.toml`: criterion
# is a few hundred packages and the root lockfile is quoted in the readme as
# holding exactly one.
#
# libmagic is turned on if it can be found, and left off if it cannot, rather
# than being required -- see `BENCH_FEATURES` at the top. The report prints which
# libraries actually ran, so a two-row table is never mistaken for a three-row one.
#
# `cargo bench` is not in `test`, and this target is not in CI's gate, for the
# same reason: a nanosecond figure on a shared runner is a property of the runner.
# `.github/workflows/benchmarks.yml` runs it and writes the numbers to the job's
# step summary, where a person reads them.
# `BENCH_ARGS` exists so that CI and a local run go through this one target
# rather than through two copies of the same cargo line. Criterion's defaults are
# five seconds of measurement per benchmark and there are 26 of them, which is
# fine on a developer machine and too slow for a shared runner; CI passes shorter
# timings. The alternative -- a `cargo bench` line written into the workflow --
# is a second place for the benchmark invocation to live, which is the drift
# `tests/ci_coverage.rs` exists to prevent.
BENCH_ARGS ?=

bench:
	@cargo bench --manifest-path benchmarks/Cargo.toml --features "$(BENCH_FEATURES)" $(BENCH_ARGS)
	@$(MAKE) --no-print-directory bench-report

# The report on its own, which is what CI publishes. Kept separate from `bench`
# because this is the fast half -- no criterion, one pass each -- and it is the
# half a person reads. Writes `summary.md` beside the crate's manifest as well as
# to stdout, because a step summary wants a file.
BENCH_SUMMARY ?= benchmarks/summary.md
bench-report:
	@cargo run --release --manifest-path benchmarks/Cargo.toml --features "$(BENCH_FEATURES)" --bin report -- $(BENCH_SUMMARY)
	@cat $(BENCH_SUMMARY)

# Proves the benchmark harness is not quietly flattering: each mutation is a way
# the report could lie, applied in turn, and every test is expected to go red.
# There is no equivalent for the library itself, and the reason is in
# `benchmarks/mutations.ps1`.
bench-mutations:
	@pwsh -NoProfile -File benchmarks/mutations.ps1

