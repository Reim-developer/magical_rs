// Classifying a whole directory tree and summarising what is in it.
//
// Run it with:
//
//   node examples/04_scan_a_directory.mjs
//
// The tree is written to a temporary directory rather than committed, so the
// example is self-contained and nothing in it has to be regenerated when the
// detection table changes.

import { mkdtempSync, mkdirSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, relative } from "node:path";

import { detectPath, displayName, mime } from "../index.js";

/** Bytes of an ASCII string, without going through UTF-8 encoding. */
function ascii(text) {
  const out = new Uint8Array(text.length);
  for (let i = 0; i < text.length; i++) out[i] = text.charCodeAt(i) & 0xff;
  return out;
}

function png() {
  return Uint8Array.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
}

const ZIP = Uint8Array.from([0x50, 0x4b, 0x03, 0x04, 0x14, 0x00, 0x00, 0x00]);
const JPEG = Uint8Array.from([0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10]);
const GIF = ascii("GIF89a");

const TREE = {
  "photo.png": png(),
  "photo-2.png": png(),
  "holiday.jpg": JPEG,
  "archive.zip": ZIP,
  "notes.txt": ascii("Just some text, with no magic bytes anywhere in it.\n"),
  "nested/report.pdf": ascii("%PDF-1.7\n"),
  "nested/deep/animated.gif": GIF,
  "nested/deep/static.gif": GIF,
};

function build(root) {
  for (const [name, payload] of Object.entries(TREE)) {
    const path = join(root, name);
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, payload);
  }
}

/**
 * Every file under `root`, recursively.
 *
 * `readdirSync(withFileTypes: true)` rather than a glob, because there is no glob
 * in Node's standard library and this package has no dependencies to bring one in.
 * A directory scan is the case where a dependency would have to be earned, and it
 * is fifteen lines.
 */
function* walk(root) {
  for (const entry of readdirSync(root, { withFileTypes: true })) {
    const path = join(root, entry.name);
    if (entry.isDirectory()) yield* walk(path);
    else if (entry.isFile()) yield path;
  }
}

function scan(root) {
  const counts = new Map();
  const unknown = [];
  for (const path of walk(root)) {
    const kind = detectPath(path);
    if (kind === null) {
      unknown.push([relative(root, path).replaceAll("\\", "/"), statSync(path).size]);
    } else {
      counts.set(kind, (counts.get(kind) ?? 0) + 1);
    }
  }
  return { counts, unknown };
}

const dir = mkdtempSync(join(tmpdir(), "magical-js-example-"));
try {
  build(dir);
  const { counts, unknown } = scan(dir);
  const total = [...counts.values()].reduce((sum, n) => sum + n, 0) + unknown.length;

  console.log(`${total} files under the tree, tallied by kind:`);
  const ordered = [...counts].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]));
  for (const [kind, count] of ordered) {
    // The key is a `FileKind`, so it groups and sorts on its own. A tally keyed by
    // a string would need the media type looked up separately and could not tell
    // `null` apart from a name it had never seen.
    console.log(`  ${displayName(kind).padEnd(16)} x ${count}  (${mime(kind) ?? "no media type"})`);
  }

  console.log();
  console.log(`${unknown.length} of them matched no signature:`);
  for (const [name, size] of unknown) {
    console.log(`  ${name} (${size} bytes)`);
  }
  console.log();
  console.log(
    "  a text file has no magic bytes, and guessing at one produces false positives",
  );
  console.log("  on every file that starts with a letter. That is why they are excluded.");
} finally {
  rmSync(dir, { recursive: true, force: true });
}
