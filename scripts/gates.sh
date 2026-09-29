#!/usr/bin/env bash
# Every check the Python bindings have to pass, in one place.
#
# CI runs this so a local run and a CI run cannot disagree about what "green"
# means. Run it from anywhere; paths are resolved relative to this script.
set -euo pipefail

# The script lives in `scripts/` and the crate it gates lives in
# `bindings/python`, so those are two different directories. Deriving one from
# the other is what keeps "run it from anywhere" true: the only absolute path
# here is the one `BASH_SOURCE` gives, and everything else is read from
# `bindings`.
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bindings="$(cd "$here/../bindings/python" && pwd)"

echo "== building the extension =="
uv run --directory "$bindings" maturin develop

echo "== pytest =="
uv run --directory "$bindings" pytest -q

echo "== pyright (strict) =="
uv run --directory "$bindings" pyright

echo "== rustfmt =="
cargo fmt --manifest-path "$bindings/Cargo.toml" --check

echo "== clippy =="
# The same lint groups the main crate's Makefile denies. The binding is held
# to the same bar, so a stricter standard never drifts in one direction.
cargo clippy \
  --manifest-path "$bindings/Cargo.toml" \
  --all-targets -- \
  -D clippy::all \
  -D clippy::pedantic \
  -D clippy::nursery \
  -D clippy::perf

echo
echo "all gates passed"
