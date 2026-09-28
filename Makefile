.PHONY: target test linter test-dyn test-unsafe test-nostd build-nostd

test:
	@cargo test

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
