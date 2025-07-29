#!/bin/bash

cargo="cargo"
venus_core="venus_core"

function run_test() {
    cd ..
    if ! command -v "$cargo" >/dev/null 2>&1; then 
        echo "Fatal error: Could not find $cargo."
        exit 1
    fi 

    if [ ! -d "$venus_core" ]; then
        echo "Fatal error: The Rust crate '$venus_core' is missing."
        exit 1
    fi

    cd "$venus_core" || {
        echo "Fatal error: Could not change working directory to: $venus_core"
        exit 1
    }

    cargo test
    cargo clippy \
    --all-targets \
    --all-features \
    -- -D clippy::all\
    -D clippy::pedantic \
    -D clippy::nursery \
    -D clippy::perf
}

run_test