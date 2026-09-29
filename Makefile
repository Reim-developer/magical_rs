.PHONY: test linter fmt test-dyn test-unsafe test-nostd build-nostd examples

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
build-nostd:
	@rustup target add thumbv7em-none-eabi
	@cargo build --no-default-features --target thumbv7em-none-eabi

# Doctests are excluded here, and that is deliberate rather than convenient.
# The readme's quick start and the `bytes_read` examples all call
# `read_file_header`, which is `#[cfg(feature = "std")]`, so a no_std doctest
# run cannot compile them. Making them compile would mean either a second
# readme or hidden `cfg` scaffolding in the one the reader sees. The cross
# compile in `build-nostd` is what actually proves the `no_std` build works;
# this target proves the rest of the suite still passes without `std`.
test-nostd:
	@cargo test --no-default-features --lib --tests
	@$(MAKE) build-nostd

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
