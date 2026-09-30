// The text helpers the generators share, tested where a test runner already runs.
//
// `scripts/lib/text.mjs` exists because this repository is checked out with CRLF on
// a Windows contributor's machine and LF in CI. Neither of those is the bug. The
// bug is a generator that hardcodes one of them: it rewrites every file it touches
// on the other platform, reports "wrote 2 files" for a run that changed nothing,
// and the next person stops reading the output.
//
// There is no JavaScript test runner at the repository root -- the crate's is
// cargo, the Python binding's is pytest, and the NodeJS binding's is `node --test`
// with no dependency. So this file imports the helper across three directories
// rather than adding a fourth runner for three functions.

import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { detectEol, lines, readLines, readTextFile, withEol, writeIfChanged } from "../../../scripts/lib/text.mjs";

test("detectEol answers what the file mostly uses", () => {
  assert.equal(detectEol("a\nb\n"), "\n");
  assert.equal(detectEol("a\r\nb\r\n"), "\r\n");
  // Empty and single-line files, which is where a "does it contain \r" check goes
  // wrong: a file with no line at all has no answer, and the answer has to be the
  // one the index holds, which is LF.
  assert.equal(detectEol(""), "\n");
  assert.equal(detectEol("no newline at all"), "\n");
  assert.equal(detectEol("one\r\n"), "\r\n");
});

test("lines() strips both endings and no trailing empty line", () => {
  // The asymmetry that this function exists for: `split("\n")` on a CRLF file
  // leaves a `\r` on every line, and every regex that ends a line -- `/\*\*$/`,
  // `/^\s*$/` -- then stops matching on Windows only.
  assert.deepEqual(lines("a\nb\n"), ["a", "b"]);
  assert.deepEqual(lines("a\r\nb\r\n"), ["a", "b"]);
  assert.deepEqual(lines("a\r\nb"), ["a", "b"]);
  assert.deepEqual(lines(""), []);
  assert.deepEqual(lines("\n"), [""]);
  assert.deepEqual(lines("a\n\n\n"), ["a", "", ""]);
});

test("readLines and readTextFile answer null for a file that is not there", () => {
  // `null` rather than an exception, so a generator that has nothing to say about
  // one optional file does not need a try. `ENOENT` is the only absence treated as
  // normal: a permission error is a real problem and is reported as one, because
  // "the file is not there" is how a typo becomes an empty table.
  const missing = join(tmpdir(), `magical-js-no-such-file-${process.pid}`);
  assert.equal(readTextFile(missing), null);
  assert.equal(readLines(missing), null);
});

test("withEol normalises, and always leaves exactly one final newline", () => {
  assert.equal(withEol("a\nb\n", "\r\n"), "a\r\nb\r\n");
  assert.equal(withEol("a\r\nb\r\n", "\n"), "a\nb\n");
  // Trailing blank lines are collapsed rather than preserved, because a generated
  // file that grows one every run is a file nobody can diff.
  assert.equal(withEol("a\n\n\n", "\n"), "a\n");
  assert.equal(withEol("a", "\n"), "a\n");
  assert.equal(withEol("", "\n"), "\n");
});

test("writeIfChanged writes once and then reports unchanged forever", () => {
  const dir = mkdtempSync(join(tmpdir(), "magical-js-text-"));
  try {
    const path = join(dir, "generated.txt");

    assert.equal(writeIfChanged(path, "one\n"), "created");
    assert.equal(readFileSync(path, "utf8"), "one\n");

    // Same content: the file is not touched, and the timestamp proves it. A
    // generator that rewrote it would leave a modified working tree for a run that
    // changed nothing, and `git status` noise is how a real change gets missed.
    const first = statSync(path).mtimeMs;
    assert.equal(writeIfChanged(path, "one\n"), "unchanged");
    assert.equal(statSync(path).mtimeMs, first, "an unchanged write touched the file");

    assert.equal(writeIfChanged(path, "two\n"), "wrote");
    assert.equal(readFileSync(path, "utf8"), "two\n");
    assert.equal(writeIfChanged(path, "two\n"), "unchanged");
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("a CRLF file offered LF content is unchanged, not rewritten", () => {
  // The case the whole module is for. Git normalises line endings on commit, so
  // the blob would not change either way and `git diff` would be empty -- but the
  // generator's own report would say "wrote", and a report that is wrong about a
  // run which changed nothing is a report nobody reads.
  const dir = mkdtempSync(join(tmpdir(), "magical-js-eol-"));
  try {
    const path = join(dir, "generated.txt");
    writeFileSync(path, "a\r\nb\r\n", "utf8");

    assert.equal(writeIfChanged(path, "a\nb\n"), "unchanged");
    assert.equal(writeIfChanged(path, "a\nb\nc\n"), "wrote");
    assert.equal(readFileSync(path, "utf8"), "a\r\nb\r\nc\r\n");
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
/**
 * The keys of one `export const NAME = ... { ... }` block, in the order written.
 *
 * `Object.freeze({` counts as well as `{`: three of the six tables are frozen, and
 * a helper that only matched one spelling would report "no such table" for a table
 * that is right there, which is a confusing way to fail.
 */
function tableKeys(source, name) {
  const open = new RegExp(`^export const ${name} = (?:Object\\.freeze\\()?\\{$`, "m").exec(source);
  assert.ok(open, `_kinds.js has no ${name} table`);
  const from = open.index + open[0].length;
  // `});` for a frozen table and `};` for a plain one, and both end the block.
  const close = /\n\};?/.exec(source.slice(from));
  assert.ok(close, `_kinds.js's ${name} table is not closed`);
  return [...source.slice(from, from + close.index).matchAll(/^ {2}([A-Za-z_][A-Za-z0-9_]*):/gm)].map(
    (match) => match[1],
  );
}

test("the checked-in tables are what the dataset says, in its order", () => {
  // Not a re-run of the generator -- that would test the generator against itself.
  // What this asserts is the property that makes `formats.json` a source of truth
  // rather than a suggestion: every table Node loads carries the 114 names the ABI
  // has, in the order the dataset gives them. A contributor who edits the dataset
  // and forgets to regenerate gets a red test here rather than a package that
  // mislabels every file.
  const dataset = JSON.parse(readFileSync(new URL("../../../formats.json", import.meta.url), "utf8"));
  const expected = dataset.formats
    .toSorted((a, b) => a.abi_order - b.abi_order)
    .map((format) => format.variant);

  assert.equal(expected.length, 114);
  assert.equal(new Set(expected).size, 114, "formats.json names a variant twice");
  assert.deepEqual(
    dataset.formats.map((format) => format.abi_order).toSorted((a, b) => a - b),
    Array.from({ length: 114 }, (_, at) => at),
    "abi_order is not 0..113 with no gaps",
  );

  // Four keyed tables whose *order* is the ABI, read from the file rather than from
  // the module: importing `_kinds.js` would work, but reading it keeps this test
  // about the checked-in file, which is the thing a stale regeneration breaks.
  const generated = readFileSync(new URL("../_kinds.js", import.meta.url), "utf8");
  for (const name of [
    "FileKind",
    "FILE_KIND_DISPLAY_NAME",
    "FILE_KIND_MIME",
    "FILE_KIND_EXTENSION",
  ]) {
    assert.deepEqual(tableKeys(generated, name), expected, `_kinds.js's ${name} has drifted`);
  }

  // The index map holds `[name, abi_order]` pairs, which is the one table whose
  // values are load-bearing rather than merely ordered, so it is checked against
  // the dataset's positions and not only against its order.
  const pairs = new Map(
    [...generated.matchAll(/^ {2}\["([^"]+)", (\d+)\],$/gm)].map(([, name, at]) => [name, Number(at)]),
  );
  assert.equal(pairs.size, 114);
  for (const format of dataset.formats) {
    assert.equal(pairs.get(format.variant), format.abi_order, `${format.variant}'s index`);
  }

  // And the declaration file, whose `FileKind` union has to be the same list --
  // a type that dropped a format would compile and then reject a valid call.
  const declared = readFileSync(new URL("../_kinds.d.ts", import.meta.url), "utf8");
  assert.deepEqual(
    [...declared.matchAll(/^ {2}\| "([A-Za-z_][A-Za-z0-9_]*)"$/gm)].map((match) => match[1]),
    expected,
    "_kinds.d.ts's FileKind union has drifted from the dataset",
  );
});
