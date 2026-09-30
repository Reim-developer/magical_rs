// Builds the WebAssembly module and installs it beside the JavaScript.
//
// The crate is one directory up, in `bindings/asm`, and this script is the only
// thing that knows it. That is the point of the split: the module's Rust and the
// package's JavaScript change for different reasons and are versioned apart, but
// the copy step below is what keeps the memory ABI honest, and it has to be run by
// whatever builds the module rather than remembered by whoever builds it.
//
// Three jobs, in this order because each one narrows what the next can be:
//
//   1. `cargo build --release --target wasm32-unknown-unknown` in `bindings/asm`.
//   2. Copy the artifact to `magical_js.wasm` in this package's root, which is
//      where `_wasm.js` looks for it. A copy rather than a symlink: npm packs
//      symlinks as links to paths that do not exist inside the tarball, so a
//      published package built with a symlink installs a module that cannot be
//      read.
//   3. Verify what was built.
//
// Step 3 is the part that earns its place. The design claim in
// `bindings/asm/src/lib.rs` is that this module has *no imports*, which is what
// lets a loader instantiate it with `{}` and no glue. Nothing in the Rust build
// would notice that claim going false - adding a `#[link]` or an `extern "C"`
// import compiles perfectly well, and the symptom would appear later as an
// `LinkError` in somebody else's project. So it is asserted here, at the moment
// the artifact exists.
//
// The export list is checked for the same reason. A renamed export breaks every
// caller with a `TypeError: not a function`, which is at least loud; a *removed*
// one that nothing calls would be silent forever.

import { copyFileSync, readFileSync, statSync } from "node:fs";
import { gzipSync } from "node:zlib";
import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const bindingDir = join(dirname(fileURLToPath(import.meta.url)), "..");
const crateDir = join(bindingDir, "..", "asm");
const artifact = join(
  crateDir,
  "target",
  "wasm32-unknown-unknown",
  "release",
  "magical_js.wasm",
);
const installed = join(bindingDir, "magical_js.wasm");

/**
 * Every export `_wasm.js` calls.
 *
 * Listed here rather than derived from the module, because deriving it would
 * agree with the module by construction and check nothing. The point is to catch
 * a rename on the Rust side, and only a hand-written list can do that.
 */
const REQUIRED_EXPORTS = [
  // Level 1.
  "which_kind_at",
  "which_kind_max_at",
  "kind_matches_at",
  // The table blob.
  "table_ptr",
  "table_len",
  // Where to write. The module allocates, because address 0 holds its own data.
  "input_ptr",
  "answer_ptr",
  // Level 2.
  "rules_new",
  "rules_add_signature",
  "rules_add_offset",
  "rules_next_rule",
  "rules_finish",
  "rules_release",
  "rules_match",
  "rules_match_all",
  "rules_len",
  // The module's memory, which `_wasm.js` reads and writes directly.
  "memory",
];

function fail(message) {
  console.error(`build: ${message}`);
  process.exit(1);
}

// No `shell: true`. Node appends `.exe` itself when resolving a bare command
// name on Windows, so `cargo` resolves to `cargo.exe` without it — and with it
// Node prints DEP0190, because arguments passed through a shell are concatenated
// rather than escaped.
//
// `cwd` is the crate, not this package. Cargo resolves a bare `--manifest-path`
// against the current directory, so pointing it at this package and passing the
// crate's manifest would work too — but a `cd` is one fewer path to get wrong
// when the two directories move.
const build = spawnSync("cargo", ["build", "--release", "--target", "wasm32-unknown-unknown"], {
  cwd: crateDir,
  stdio: "inherit",
});
if (build.error) {
  fail(`could not run cargo: ${build.error.message}`);
}
if (build.status !== 0) {
  fail(`cargo build exited with ${build.status}`);
}

copyFileSync(artifact, installed);

const bytes = readFileSync(installed);
const module = new WebAssembly.Module(bytes);
const imports = WebAssembly.Module.imports(module);
const exports = WebAssembly.Module.exports(module);

if (imports.length !== 0) {
  fail(
    `the module declares ${imports.length} import(s): ` +
      imports.map((entry) => `${entry.module}.${entry.name}`).join(", ") +
      "\n  `_wasm.js` instantiates it with `{}`, so an import is a load-time\n" +
      "  `LinkError` for every caller. If a new dependency needs one, the loader\n" +
      "  and index.d.ts both have to change with it — do it deliberately.",
  );
}

const present = new Set(exports.map((entry) => entry.name));
const missing = REQUIRED_EXPORTS.filter((name) => !present.has(name));
if (missing.length > 0) {
  fail(`the module is missing export(s): ${missing.join(", ")}`);
}

const kib = (n) => `${(n / 1024).toFixed(1)} KiB`;
console.log(`build: ${installed}`);
console.log(`  crate     ${crateDir}`);
console.log(`  raw       ${String(bytes.length).padStart(7)} bytes  (${kib(bytes.length)})`);
console.log(`  gzip      ${String(gzipSync(bytes).length).padStart(7)} bytes  (${kib(gzipSync(bytes).length)})`);
console.log(`  imports   ${imports.length}`);
console.log(`  exports   ${exports.length}`);
if (statSync(installed).size !== bytes.length) {
  fail("the installed copy is not the artifact that was built");
}
