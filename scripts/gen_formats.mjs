// Regenerates the JavaScript and TypeScript format tables from `formats.json`.
//
//   bindings/nodejs/_kinds.js     what Node runs
//   bindings/nodejs/_kinds.d.ts   what TypeScript reads
//
// `formats.json` is the project's single source of truth for a format's name,
// MIME type and extension, and it is also what generates the Rust and Python
// metadata (see `scripts/gen_kinds.ps1`). This script is the JavaScript half of
// that, and it exists rather than extending the PowerShell one because the two
// emit very different things: this one emits two plain text files, and asking a
// PowerShell script to do it would mean asking it to manage a `.d.ts`.
//
// ## The indices are not metadata, and that is the reason this file exists
//
// `FILE_KIND_INDICES` is the ABI, not a lookup table. `FileKind` is a fieldless
// Rust enum, so `FileKind::Png as i32` is 0 and stays 0 only for as long as `Png`
// stays first in `pub enum FileKind`. JavaScript cannot see Rust's discriminants,
// so the order of that declaration is this file's contract, and nothing at
// runtime would notice if it drifted. `formats.json` carries the order as
// `abi_order` -- read out of the declaration when the dataset was built -- and
// `test/kinds.test.js` compares all 114 names against the kind indices the
// compiled module reports, so a reordered enum fails a test rather than silently
// mis-naming every format.
//
// ## Why two files
//
// `_kinds.js` is what Node runs and `_kinds.d.ts` is what TypeScript reads, and
// the split is a deliberate consequence rather than a convenience: `as const` and
// `export type` are TypeScript syntax, and they would be a SyntaxError in the
// `.js` Node actually loads. Collapsing the pair into one `.ts` would fix that by
// needing a build step and a loader -- a whole toolchain added to a package whose
// entire point is that it has none.
//
// ## Why JSON
//
// Because three languages read it with nothing installed: `JSON.parse` is in
// Node, `json` is in Python's standard library, and the Rust side never parses it
// at all -- it is generated *from* it by a script. A dataset this project can
// afford to keep is one that can be read from every side of it for free.

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { writeIfChanged } from "./lib/text.mjs";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = join(scriptDir, "..");
const bindingDir = join(repoRoot, "bindings", "nodejs");

/** The table every target is generated from. */
const DATASET = join(repoRoot, "formats.json");

// Pinned because every generated file, the readme and every test state 114, so a
// silent 115 has to take a deliberate edit here rather than arrive by being one
// row longer in a data file.
const EXPECTED_FORMATS = 114;

/** Reads and validates the dataset. Returns the formats in declaration order. */
function readDataset() {
  const parsed = JSON.parse(readFileSync(DATASET, "utf8"));
  const { formats } = parsed;

  if (!Array.isArray(formats)) throw new Error(`${DATASET} has no \`formats\` array`);
  if (formats.length !== EXPECTED_FORMATS) {
    throw new Error(
      `${DATASET} has ${formats.length} formats, expected ${EXPECTED_FORMATS}. Every generated ` +
        "file and every test states the count, so changing it is a deliberate edit here rather " +
        "than a row added here.",
    );
  }

  // `abi_order` is what makes the tables below load-bearing, so it is checked
  // before anything is emitted: a duplicate or missing position would mean one
  // format answerable under another format's name, and the error would otherwise
  // surface as a mislabelled file rather than as a bad dataset.
  const positions = new Set();
  const variants = new Set();
  for (const format of formats) {
    if (typeof format.variant !== "string" || !/^[A-Za-z_][A-Za-z0-9_]*$/.test(format.variant)) {
      throw new Error(`\`variant\` is the Rust enum member's name and must be an identifier: ${JSON.stringify(format)}`);
    }
    if (variants.has(format.variant)) throw new Error(`two formats are both \`${format.variant}\``);
    variants.add(format.variant);

    if (!Number.isInteger(format.abi_order) || format.abi_order < 0 || format.abi_order >= formats.length) {
      throw new Error(
        `\`${format.variant}\`.abi_order is ${JSON.stringify(format.abi_order)}, which is not one of ` +
          `0..${formats.length - 1}. It is the position in \`pub enum FileKind\`; re-run ` +
          "`node scripts/build_formats.mjs` to read it back out of the declaration.",
      );
    }
    if (positions.has(format.abi_order)) {
      throw new Error(
        `abi_order ${format.abi_order} is claimed by both ${format.variant} and ` +
          `${[...formats].find((f) => f.abi_order === format.abi_order && f !== format).variant}`,
      );
    }
    positions.add(format.abi_order);

    if (typeof format.name !== "string" || format.name.length === 0) {
      throw new Error(`\`${format.variant}\` has no display \`name\``);
    }
    // `null` is a real answer for these two -- "not registered" rather than
    // "unknown" -- so the key has to be *present*. A missing key would read as
    // `undefined` and generate a table with a hole in it.
    for (const field of ["mime", "extension"]) {
      if (!(field in format)) throw new Error(`\`${format.variant}\` has no \`${field}\` key; use null for none`);
      const value = format[field];
      if (value !== null && (typeof value !== "string" || value.length === 0)) {
        throw new Error(`\`${format.variant}\`.${field} is neither a non-empty string nor null`);
      }
    }
  }

  return [...formats].sort((a, b) => a.abi_order - b.abi_order);
}

/** A JSON literal, which is also a legal JavaScript and TypeScript one. */
function quoted(text) {
  return JSON.stringify(text);
}

/** An optional string as JavaScript, `null` for null. */
function optional(value) {
  return value === null ? "null" : quoted(value);
}

/**
 * One `value,` line per format, at the given indent.
 *
 * The trailing comma is on every line including the last, rather than only where
 * a linter would want it. One rule instead of two means a generated line has no
 * shape that depends on its neighbours, so adding or removing a format cannot
 * reformat the row next to it.
 */
function elements(formats, at, value) {
  return formats.map((format) => `${at}${value(format)},`).join("\n");
}

/** One `name: value,` line per format, at the given indent. */
function properties(formats, at, value) {
  return elements(formats, at, (format) => `${format.variant}: ${value(format)}`);
}

const BANNER = [
  "// Generated by scripts/gen_formats.mjs from formats.json. Do not edit;",
  "// run `npm run gen`.",
  "//",
  "// The indices and the order come from `abi_order`, which is the position of each",
  "// variant in `pub enum FileKind` in crates/magical_rs. That order *is* the ABI: a",
  "// file kind crosses the wasm boundary as its Rust discriminant, so entry N here is",
  "// what the compiled module reports as N, and nothing at runtime would notice if it",
  "// drifted. test/kinds.test.js compares every name against the kind indices the",
  "// compiled module reports.",
  "//",
  "// The names, MIME types and extensions come from the same table the Rust and Python",
  "// metadata are generated from, so a MIME type cannot be one thing in Rust and",
  "// another in JavaScript.",
  "//",
  "// The types live in the .d.ts beside this file, not here: `as const` and",
  "// `export type` are TypeScript syntax and would not parse in the .js Node loads. A",
  "// single .ts file would mean a build step, and this package ships no build.",
].join("\n");

// ---------------------------------------------------------------------------
// _kinds.js
// ---------------------------------------------------------------------------

const formats = readDataset();

const js = `${BANNER}

/** Every format the built-in detection table can return, in the crate's own order. */
export const FileKind = {
${properties(formats, "  ", (format) => quoted(format.variant))}
};

/**
 * Name to Rust discriminant, in the order the crate declares them.
 *
 * Exported because it is what turns a name into a call. The wasm side answers in
 * discriminants because that is all it can pass; this is where a name becomes
 * one.
 */
export const FILE_KIND_INDICES = new Map([
${elements(formats, "  ", (format) => `[${quoted(format.variant)}, ${format.abi_order}]`)}
]);

/**
 * Rust discriminant to name, indexed by that discriminant.
 *
 * The other direction, and the one the introspection API needs: the table arrives
 * from wasm as 114 anonymous integers, and each has to become a name. An array
 * rather than a reversed Map because this is read once per entry at import and
 * then never again.
 */
export const FILE_KIND_NAMES = Object.freeze([
${elements(formats, "  ", (format) => quoted(format.variant))}
]);

/**
 * The display name of each format, such as \`PNG\` or \`Zip / JAR / APK\`.
 *
 * Its own table because the two names answer different questions: this one is
 * \`PNG\` and \`FileKind.Png\` is the identifier, and a caller that wants a stable
 * identifier wants that one. It is the same string the Rust \`display_name\` and the
 * Python \`description\` answer.
 */
export const FILE_KIND_DISPLAY_NAME = Object.freeze({
${properties(formats, "  ", (format) => quoted(format.name))}
});

/**
 * The registered MIME type of each format, or \`null\` where there is none.
 *
 * \`null\` means "not registered or not verified", never "unknown". A caller that
 * needs an answer should fall back to \`application/octet-stream\` itself, so the
 * choice stays visible at the call site rather than baked in here. A wrong MIME
 * type is served to a browser, so this project does not ship one.
 */
export const FILE_KIND_MIME = Object.freeze({
${properties(formats, "  ", (format) => optional(format.mime))}
});

/**
 * The conventional extension of each format, or \`null\` where there is none.
 *
 * Without a leading dot, and advisory: detection never reads a file name, so a
 * \`.jpg\` holding a PNG is reported as a PNG. The extension is for choosing what
 * to *write*, which is the one question a magic number cannot answer.
 */
export const FILE_KIND_EXTENSION = Object.freeze({
${properties(formats, "  ", (format) => optional(format.extension))}
});
`;

// ---------------------------------------------------------------------------
// _kinds.d.ts
// ---------------------------------------------------------------------------

// The `FileKind` union is written out rather than derived, because a `const`
// object would give `"Png" | "Bitmap" | ...` in an order TypeScript is free to
// change, and this union is the package's public type -- it shows up in hover text
// and in error messages. The same order as the table below it, so the two read as
// one list.
const union = formats.map((format) => `  | ${quoted(format.variant)}`).join("\n");

const dts = `${BANNER}

/** Every format the built-in detection table can return, in the crate's own order. */
export declare const FileKind: {
  readonly [K in FileKind]: K
}

/** The name of a format, as a TypeScript type. */
export type FileKind =
${union}

/**
 * Name to Rust discriminant, in the order the crate declares them.
 *
 * Exported because it is what turns a name into a call. The wasm side answers in
 * discriminants because that is all it can pass; this is where a name becomes
 * one.
 */
export declare const FILE_KIND_INDICES: ReadonlyMap<FileKind, number>

/**
 * Rust discriminant to name, indexed by that discriminant.
 *
 * The other direction, and the one the introspection API needs: the table arrives
 * from wasm as 114 anonymous integers, and each has to become a name. An array
 * rather than a reversed Map because this is read once per entry at import and
 * then never again.
 */
export declare const FILE_KIND_NAMES: readonly FileKind[]

/** The display name of a format, such as \`PNG\` or \`Zip / JAR / APK\`. */
export declare const FILE_KIND_DISPLAY_NAME: {
  readonly [K in FileKind]: string
}

/**
 * The registered MIME type of a format, or \`null\` where there is none.
 *
 * \`null\` means "not registered or not verified", never "unknown".
 */
export declare const FILE_KIND_MIME: {
  readonly [K in FileKind]: string | null
}

/**
 * The conventional extension of a format, or \`null\` where there is none.
 *
 * No leading dot, and advisory: detection never reads a file name.
 */
export declare const FILE_KIND_EXTENSION: {
  readonly [K in FileKind]: string | null
}
`;

// ---------------------------------------------------------------------------
// Writing
// ---------------------------------------------------------------------------

for (const [name, text] of [
  ["_kinds.js", js],
  ["_kinds.d.ts", dts],
]) {
  const path = join(bindingDir, name);
  console.log(`  ${writeIfChanged(path, text).padEnd(9)} bindings/nodejs/${name}`);
}

console.log(`\n${formats.length} formats from formats.json.`);
