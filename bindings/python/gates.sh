#!/usr/bin/env bash
# Every check the Python bindings have to pass, in one place.
#
# CI runs this so a local run and a CI run cannot disagree about what "green"
# means. Run it from anywhere; paths are resolved relative to this script.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "== building the extension =="
uv run --directory "$here" maturin develop

echo "== pytest =="
uv run --directory "$here" pytest -q

echo "== pyright (strict) =="
uv run --directory "$here" pyright

echo "== rustfmt =="
cargo fmt --manifest-path "$here/Cargo.toml" --check

echo "== clippy =="
# The same lint groups the main crate's Makefile denies. The binding is held
# to the same bar, so a stricter standard never drifts in one direction.
cargo clippy \
  --manifest-path "$here/Cargo.toml" \
  --all-targets -- \
  -D clippy::all \
  -D clippy::pedantic \
  -D clippy::nursery \
  -D clippy::perf

echo
echo "all gates passed"
