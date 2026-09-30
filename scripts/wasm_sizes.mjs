// Builds the WebAssembly module at every `opt-level` and prints what each costs.
//
//   node scripts/wasm_sizes.mjs
//
// Five cargo builds, so this is not in any gate: `bindings/nodejs/scripts/gates.sh`
// already builds the module once and prints its size, which is the number that has
// to stay right. This one exists because the *choice* of `opt-level` was justified
// in three places by a figure nobody could reproduce, and the figure turned out to
// be wrong - in both the number and its direction. `bindings/asm/Cargo.toml` said
// `"s"` over `"z"` because "the measured difference between them is 243 bytes", and
// `"z"` is actually 183 bytes *larger*.
//
// A claim you cannot re-run is a claim you cannot correct either. This script is
// that correction, written down as the thing to run rather than as a number to
// trust.
//
// It edits `bindings/asm/Cargo.toml` and puts it back, including on failure. The
// original is read into memory first rather than copied aside, so there is no file
// to leave behind if this is interrupted, and the restore is in a `finally`.

import { readFileSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { gzipSync } from "node:zlib";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const bindingDir = join(repoRoot, "bindings", "nodejs");
const manifest = join(repoRoot, "bindings", "asm", "Cargo.toml");
const artifact = join(repoRoot, "bindings", "asm", "target", "wasm32-unknown-unknown", "release", "magical_js.wasm");

// The value the manifest actually carries today, which is the one to compare the
// others against. Read rather than assumed: a contributor who has already changed
// it gets their value as the baseline instead of a surprise.
const current = /^opt-level = (.+)$/m.exec(readFileSync(manifest, "utf8"))?.[1]?.trim();

const LEVELS = ['"s"', '"z"', "3", "2"];

// The `opt-level` values that are not strings. Cargo rejects `"3"` with "must be
// `0`, `1`, `2`, `3`, `s` or `z`, but found the string", which is worth knowing
// before anyone tries to reproduce a table that used one.
function withLevel(original, level) {
  // Anchored at the start of a line, because the comment above the setting quotes it
  // back: `# process downloads. `opt-level = "s"` over `z` ...`. Matching anywhere in a
  // line rewrites the comment, leaves the setting alone, and then cargo correctly
  // decides nothing changed - which is how this script first reported four identical
  // sizes for four different settings.
  return original.replace(/^opt-level = .+$/m, `opt-level = ${level}`);
}

function build(level) {
  writeFileSync(manifest, withLevel(original, level), "utf8");
  const built = spawnSync("cargo", ["build", "--release", "--target", "wasm32-unknown-unknown"], {
    cwd: join(repoRoot, "bindings", "asm"),
    stdio: "pipe",
    encoding: "utf8",
  });
  if (built.status !== 0) {
    console.error(built.stderr || built.stdout || `cargo exited with ${built.status}`);
    throw new Error(`cargo build failed at opt-level = ${level}`);
  }
  const bytes = readFileSync(artifact);
  return { raw: bytes.length, gzip: gzipSync(bytes).length };
}

const original = readFileSync(manifest, "utf8");
const rows = [];
try {
  for (const level of new Set([current, ...LEVELS].filter(Boolean))) {
    rows.push({ level, ...build(level) });
  }
} finally {
  // Restored whether or not the builds worked, and printed so a failure is visible
  // rather than showing up later as an unexplained diff.
  writeFileSync(manifest, original, "utf8");
}

const pad = (text, width) => String(text).padStart(width);
console.log(`bindings/asm, ${process.env.RUSTC_VERSION ?? "the local rustc"}:\n`);
console.log(`  ${pad("opt-level", 11)}${pad("raw", 9)}${pad("gzip", 9)}   against the manifest's own value`);
const base = rows.find((row) => row.level === current);
for (const row of rows) {
  const delta = base && row !== base ? pad(row.raw - base.raw, 6) : "";
  console.log(`  ${pad(row.level, 11)}${pad(row.raw, 9)}${pad(row.gzip, 9)}   ${delta}`);
}

console.log(`\n  the manifest says opt-level = ${current}; "z" costs ` +
  `${rows.find((row) => row.level === '"z"')?.raw - base?.raw} raw bytes against it, which is the ` +
  "figure the Cargo.toml comment used to get wrong.");
console.log("  Note that a bare `3` is a number and `\"3\"` is a string cargo rejects.");
console.log("\n  Restore the built artifact by re-running `npm run build` in bindings/nodejs.");
