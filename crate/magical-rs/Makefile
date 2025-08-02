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

test-nostd:
	@cargo test --features no_std

test-dyn:
	@cargo test --features magical_dyn
