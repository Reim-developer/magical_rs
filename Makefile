.PHONY: target

test:
	@cargo test

linter:
	@cargo clippy \
    --all-targets \
    --all-features \
    -- -D clippy::all\
    -D clippy::pedantic \
    -D clippy::nursery \
    -D clippy::perf

test-dyn:
	@cargo test --features magical_dyn

test-unsafe:
	@cargo test --features unsafe_context
