// The loader for `wasm_rust.wasm`, in plain JavaScript.
//
// Run it with:
//
//   node --experimental-strip-types ../../scripts/build_wasm_example.mjs
//   node load.mjs
//
// which is more ceremony than it should be, so the build is folded into this file:
//
//   npm run build   (in bindings/nodejs)  -- not needed, this builds its own
//   node load.mjs                         -- builds if the module is missing
//
// There is no import object, no glue file, and no bundler. That is the whole claim:
// the module declares **zero** imports, so `WebAssembly.Instance(module, {})` works
// and nothing has to be generated to make it.
//
// What this file is not is a binding. There is no public API here, no
// introspection, no custom rules and no versioned ABI -- `bindings/asm` is that, and
// it is what you would actually install. This is the smallest thing that proves the
// crate runs on WebAssembly at all.

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const artifact = join(here, "target", "wasm32-unknown-unknown", "release", "wasm_rust.wasm");

/**
 * Instantiate the module, or explain why it is not there.
 *
 * `fs.existsSync` rather than a try/catch around the read, because the two failures
 * a reader hits here are completely different: "you have not built it" is a command
 * to run, and "your build is broken" is a bug. A `catch` that printed the same
 * message for both would send someone looking in the wrong place.
 */
function load() {
  let bytes;
  try {
    bytes = readFileSync(artifact);
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
    console.error(
      `load.mjs: ${artifact} does not exist.\n` +
        "  Build it with:\n" +
        "    rustup target add wasm32-unknown-unknown\n" +
        `    cargo build --release --target wasm32-unknown-unknown --manifest-path ${join(here, "Cargo.toml")}`,
    );
    process.exit(1);
  }

  // Asserted rather than assumed. The claim is the reason this loader needs no glue,
  // and nothing in the Rust build would notice the claim going false: adding a
  // `#[link]` or an `extern "C"` import compiles perfectly well, and the symptom
  // would appear later as a `LinkError` in somebody else's project.
  const module = new WebAssembly.Module(bytes);
  const imports = WebAssembly.Module.imports(module);
  const exports = WebAssembly.Module.exports(module);
  if (imports.length !== 0) {
    console.error(
      `load.mjs: the module declares ${imports.length} import(s): ` +
        imports.map((entry) => `${entry.module}.${entry.name}`).join(", ") +
        "\n  This loader instantiates it with `{}`, so an import is a load-time",
      "\n  `LinkError` for every caller. That is the bug this check exists to catch.",
    );
    process.exit(1);
  }

  const decoder = new TextDecoder();
  return {
    bytes,
    imports,
    exports,
    instance: new WebAssembly.Instance(module, {}),

    /**
     * Detect a header, and name the answer.
     *
     * The view is rebuilt from `memory.buffer` on *every* call. That is not a
     * precaution -- `memory.buffer` is replaced whenever the module's memory grows,
     * so a view captured at load time becomes zero-length the first time anything
     * writes enough to trigger a growth, and every later call silently reads nothing.
     * The only symptom is `null` for every input, which looks exactly like "none of
     * my files are recognised".
     */
    detect(header) {
      const { memory, which_kind_at, display_name_len, display_name_bytes } = this.instance.exports;

      // Grow first. A 4 KB buffer is past the end of a small initial memory, and a
      // write past the end is a trap rather than a failed `set`.
      const current = new Uint8Array(memory.buffer).byteLength;
      if (header.length > current) memory.grow(Math.ceil((header.length - current) / 65536));

      // Re-derived *after* the grow, because the grow may have replaced the buffer.
      new Uint8Array(memory.buffer).set(header);

      const discriminant = which_kind_at(0, header.length);
      if (discriminant === -1) return null;

      const length = display_name_len(discriminant);
      if (length === 0) return { discriminant, name: null };
      const start = display_name_bytes(discriminant);
      return {
        discriminant,
        name: decoder.decode(new Uint8Array(memory.buffer, start, length)),
      };
    },
  };
}

// Fixtures written out rather than checked in. A binary in the repository is a file
// nobody reviewing a diff can read, and it is one that has to be regenerated
// whenever the detection table changes.
//
// The expected name is in the same row, which is the difference between an example
// and a check: an example that only prints is green however wrong it is, and this
// file is run by `make examples` in CI. A crate whose wasm module compiled and then
// answered "PNG" for a GIF would pass every other gate in this repository.
const FIXTURES = [
  ["a PNG header", Uint8Array.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]), "PNG"],
  ["a GIF header", Uint8Array.from([0x47, 0x49, 0x46, 0x38, 0x39, 0x61]), "GIF"],
  ["a ZIP header", Uint8Array.from([0x50, 0x4b, 0x03, 0x04, 0x14, 0x00, 0x00, 0x00]), "Zip / JAR / APK"],
  ["a WASM header", Uint8Array.from([0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]), "WebAssembly"],
  ["a PDF header", Uint8Array.from([0x25, 0x50, 0x44, 0x46, 0x2d]), "PDF"],
  ["an ELF header", Uint8Array.from([0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01]), "ELF"],
  // Magic at offset 257, which is where a tar header puts it. This row is the only
  // one that proves an *offset* is honoured rather than only a length: the buffer
  // is 512 bytes and the signature is at 257, so a matcher that read from zero
  // would find nothing.
  [
    "a tar header at 257",
    (() => {
      const out = new Uint8Array(512);
      out.set(Uint8Array.from([0x75, 0x73, 0x74, 0x61, 0x72]), 257);
      return out;
    })(),
    "Tar",
  ],
  // ISO 9660 puts its magic at 36,769 and its signature is five bytes, which is
  // where the crate's 36,870-byte read size comes from. It is the only fixture long
  // enough to force the module's memory to grow, so it is also the one that would
  // expose a loader caching its `memory.buffer` view.
  [
    "an ISO header at 32769",
    (() => {
      const out = new Uint8Array(40000);
      out.set(Uint8Array.from([0x43, 0x44, 0x30, 0x30, 0x31]), 32769);
      return out;
    })(),
    "ISO 9660",
  ],
  // And one that must *not* match. `null` is an answer, not a failure, and a
  // detector that returned something here would be inventing a format.
  ["not a format", new Uint8Array(40000).fill(0x2e), null],
];

const wasm = load();

console.log(
  `module: ${wasm.bytes.length} bytes, ${wasm.exports.length} exports, ${wasm.imports.length} imports`,
);
console.log(`  ${wasm.imports.length === 0 ? "no imports, so no glue file and no import object" : "IMPORTS DECLARED"}`);
console.log(`  exports: ${wasm.exports.map((entry) => entry.name).join(", ")}`);

console.log();
const failures = [];
for (const [label, header, expected] of FIXTURES) {
  const found = wasm.detect(header);
  const actual = found === null ? null : found.name;
  const ok = actual === expected;
  if (!ok) failures.push(`${label}: expected ${expected === null ? "null" : JSON.stringify(expected)}, got ${actual === null ? "null" : JSON.stringify(actual)}`);
  const shown = found === null ? "no signature matched" : `${found.name} (discriminant ${found.discriminant})`;
  console.log(`  ${ok ? " " : "!"} ${label.padEnd(22)} ${shown}`);
}

console.log();
if (failures.length > 0) {
  // `process.exitCode` rather than `process.exit`, so a caller piping this still
  // gets the rest of the output before the process ends.
  console.error(`${failures.length} of ${FIXTURES.length} fixtures answered wrongly:`);
  for (const failure of failures) console.error(`  ${failure}`);
  process.exitCode = 1;
} else {
  console.log(`${FIXTURES.length} fixtures, all correct.`);
  console.log();
  console.log("Four exports and the crate underneath them. That is the difference between a");
  console.log("crate that runs on WebAssembly and a binding somebody would install: the");
  console.log("other 25 KB is the encoded detection table, level 2 rules and a released ABI.");
}
