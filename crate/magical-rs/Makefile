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