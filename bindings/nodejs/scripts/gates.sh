#!/usr/bin/env bash
# Every check the NodeJS binding has to pass, in one place.
#
# CI runs this so a local run and a CI run cannot disagree about what "green"
# means. Run it from anywhere; paths are resolved relative to this script.
#
# The order is not arbitrary and not alphabetical. Each step narrows what the next
# can be:
#
#   1. Generate the kinds files. `scripts/gen_formats.mjs` reads `formats.json`
#      and fails if the format count is not 114 or if `abi_order` is not a
#      permutation of 0..113, so a changed enum is caught before anything is built
#      against it.
#   2. Run the module crate's own Rust tests, before the build: it is `rlib` as
#      well as `cdylib`, so this links the exports and calls them directly as Rust
#      functions, which is what makes the ABI testable with no wasm runtime in the
#      loop.
#   3. Build the WebAssembly module, which also asserts the module has no imports
#      and exports everything `_wasm.js` calls. A missing export is a `TypeError`
#      for a caller; a *renamed* one is a failing build instead.
#   4. Type-check the hand-written declarations, because `tsc` is the only thing
#      that can see whether the generics still narrow.
#   5. Run the tests, which compare `_kinds.js` against the compiled module's own
#      discriminants and exercise the memory ABI.
#   6. rustfmt and clippy on the crate's Rust, last, because they are the slowest
#      and say nothing about whether the package works.
#
# `npm test` would run 1 to 5, but spelled out here so a failure names the step.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bindings="$(cd "$here/.." && pwd)"
# The generator lives at the repository root rather than beside the binding,
# because it reads `formats.json` -- the same dataset that generates the Rust and
# Python metadata -- and a script two directories away from the only file it reads
# is a script nobody finds. Three levels up from `bindings/nodejs/scripts`.
repo="$(cd "$here/../../.." && pwd)"
# The module's Rust, which is a sibling of this package rather than part of it.
# It publishes nothing and has its own lockfile; see its `Cargo.toml` for why.
crate="$bindings/../asm"

echo "== generating the kinds files =="
node "$repo/scripts/gen_formats.mjs"

echo "== cargo test (the module's own Rust) =="
cargo test --manifest-path "$crate/Cargo.toml"

echo "== building the WebAssembly module =="
node "$bindings/scripts/build.mjs"

echo "== type-checking the declarations =="
npm --prefix "$bindings" run --silent types

echo "== node --test =="
# Through the npm script, not `node --test "$bindings/test/"`, so this and a
# contributor running `npm test` run the same command. A directory argument also
# picks up `test/fixtures.js` and `test/types.ts` as test files, which is not what
# they are: the glob in the script is `test/*.test.js`.
npm --prefix "$bindings" run --silent test:only

echo "== running the examples =="
# Run, not build, and not merely check that they parse. `npm test` already imports
# every module the package ships, so a file in `examples/` is the only thing here
# that nothing else would execute - which is exactly why an example that has drifted
# stays green until somebody reads it. There is no bundler and no build step here, so
# `node` on each file is the whole verification.
#
# Each one in its own process, so an example that leaves the module's memory grown,
# or writes a temporary directory it fails to clean up, cannot make the next one lie.
for example in "$bindings"/examples/*.mjs; do
  echo "  $(basename "$example")"
  node "$example" > /dev/null
done

echo "== running the TypeScript example =="
# The flag is unneeded on Node 23.6+ and needed on 22.6 to 23.5; it is harmless on
# both, and `engines` allows 22. The important half is `npm run types` above, which
# is what makes this a TypeScript example rather than a JavaScript one -- running it
# only proves the stripping works.
echo "  06_typed.ts"
node --experimental-strip-types "$bindings/examples/06_typed.ts" > /dev/null

echo "== rustfmt =="
cargo fmt --manifest-path "$crate/Cargo.toml" --check

echo "== clippy =="
# The same lint groups the main crate's Makefile denies and that
# `scripts/gates.sh` denies for the Python binding. All three crates are held to
# the same bar, so a stricter standard never drifts in one direction.
cargo clippy \
  --manifest-path "$crate/Cargo.toml" \
  --all-targets -- \
  -D clippy::all \
  -D clippy::pedantic \
  -D clippy::nursery \
  -D clippy::perf \
  -D warnings

echo
echo "all gates passed"
