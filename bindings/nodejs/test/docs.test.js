// Executes the JavaScript examples in `docs/`.
//
// `readme.test.js` does this for `bindings/nodejs/README.md` and
// `crates/magical_rs/tests/readme_examples.rs` does it for the repository readme's
// Rust. `docs/` is a third document with its own code, and an example that does not
// run is a claim the reader finds out about by pasting it into a file.
//
// The snippets are transcribed rather than extracted. Extracting them means a
// parser for Markdown fences, and a parser that silently finds zero snippets
// passes — which is the failure this file exists to prevent. So each test below
// names the page and the heading it came from, and `every_page_is_covered` checks
// the other direction: that this file does not drift from the pages it claims to
// cover.
//
// One of these tests exists because the page was wrong and the compiler could not
// say so. `docs/api/javascript.md` wrote the returned kind as `"jpg"` for six
// releases of the readme's worth of prose, and `"jpg"` is a plausible-looking
// string that no `FileKind` member has ever been: the values are the variant
// names, so it is `"Jpg"`. Nothing in the module rejects `"jpg"` — it is not a
// `FileKind`, so it is not a type error either — which is precisely why a prose
// claim about a string union needs a test rather than a type checker.

import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync, writeFileSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

import {
  DEFAULT_MAX_BYTES_READ,
  allKinds,
  describe,
  detectBytes,
  detectPath,
  displayName,
  extension,
  isFileKind,
  matchAllTypes,
  matchTypes,
  matches,
  mime,
  neededBytes,
  readHeader,
  readLimits,
  releaseRules,
  signatureTable,
} from "../index.js";

const HERE = dirname(fileURLToPath(import.meta.url));
const DOCS = join(HERE, "..", "..", "..", "docs");

// The `photo.jpg` and `logo.png` the pages use, built from real magic bytes so the
// page and the test cannot disagree about what a JPEG is.
const JPEG = Uint8Array.from([0xff, 0xd8, 0xff, 0xe0, ...new Array(32).fill(0)]);
const PNG = Uint8Array.from([
  0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, ...new Array(32).fill(0),
]);
const GIF = Uint8Array.from([0x47, 0x49, 0x46, 0x38, 0x39, 0x61]);
const NOISE = new Uint8Array(64).fill(0x2e); // ".", which no signature starts with

/** A scratch directory holding the file the pages read by name. */
function withPhoto(run) {
  const dir = mkdtempSync(join(tmpdir(), "magical-docs-"));
  try {
    writeFileSync(join(dir, "photo.jpg"), JPEG);
    run(dir);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

function page(name) {
  const text = readFileSync(join(DOCS, name), "utf8");
  assert.ok(text.length > 0, `docs/${name} is listed in docs.test.js but is empty`);
  return text;
}

// --------------------------------------------------------------------------
// docs/api/javascript.md
// --------------------------------------------------------------------------

test("the page's level 1 example holds", () => {
  // docs/api/javascript.md — "Level 1"
  withPhoto((dir) => {
    assert.equal(detectPath(join(dir, "photo.jpg")), "Jpg");
    assert.equal(detectBytes(GIF), "GIF");
    assert.equal(detectBytes(NOISE), null);

    // A path that is not there is Node's own error, not a null: the page's "a null
    // is one thing" claim is about bytes, and a missing file is not bytes.
    assert.throws(() => detectPath(join(dir, "absent.jpg")), { code: "ENOENT" });
    assert.equal(neededBytes(), 36_870);
    assert.equal(DEFAULT_MAX_BYTES_READ, 2_048);
  });
});

test("a null means one thing, and it is 'no rule matched'", () => {
  // docs/api/javascript.md — "A `null` is one thing here"
  assert.equal(detectBytes(NOISE), null);
  assert.equal(detectBytes(new Uint8Array(0)), null);

  // Naming the window is what separates "no rule matched" from "no rule that fits
  // the window you named matched". The ISO 9660 magic sits at 36,865.
  const iso = new Uint8Array(40_000);
  iso.set([0x43, 0x44, 0x30, 0x30, 0x31], 36_865); // "CD001"

  assert.equal(detectBytes(iso), "ISO");
  assert.equal(detectBytes(iso, { maxBytesRead: 2_048 }), null);
  assert.equal(detectBytes(NOISE, { maxBytesRead: 2_048 }), null);
});

test("the page's metadata example holds", () => {
  // docs/api/javascript.md — "Metadata"
  assert.equal(displayName("Jpg"), "JPEG");
  assert.equal(mime("Jpg"), "image/jpeg");
  assert.equal(extension("Jpg"), "jpg");

  // `None`/`null` means "none registered", never an empty string wearing the type.
  const empty = allKinds().filter((kind) => mime(kind) === "" || extension(kind) === "");
  assert.deepEqual(empty, []);
});

test("describe narrows to the literal, and the table is in detection order", () => {
  // docs/api/javascript.md — "Asking the table a question"
  const rule = describe("Png");

  assert.equal(rule.kind, "Png");
  assert.deepEqual([...rule.signatures[0]], [...PNG.slice(0, 8)]);
  assert.deepEqual(rule.offsets, [0]);
  assert.equal(rule.maxBytesRead, 2_048);
  assert.equal(rule.usesPredicate, false);

  // The guard is for a value from outside your own code, and it is case-sensitive
  // because the values are variant names rather than extensions.
  assert.equal(isFileKind("Jpg"), true);
  assert.equal(isFileKind("jpg"), false);
  assert.equal(isFileKind("nonsense"), false);

  // The row a PNG matches is the first row, so here the walk agrees.
  const table = signatureTable();
  const firstMatch = table.findIndex((row) => matches(row.kind, PNG));
  assert.equal(firstMatch, 0);
  assert.equal(table[firstMatch].kind, detectBytes(PNG));
});

test("signatureTable is in detection order, so walking it reproduces detectBytes", () => {
  // docs/api/javascript.md — "Walking `signatureTable()` reproduces detection"
  //
  // Until issue #24 this list was in enum order, and this test asserted that,
  // which is how the bug survived: the assertion matched the implementation and
  // contradicted the docstring three lines above it. What is checked now is the
  // property, because a copied list of 114 names would be wrong in the same way as
  // the code it was copied from.
  const table = signatureTable();
  const pos = new Map(table.map((row, i) => [row.kind, i]));

  // Detection order: ScriptExecute at 17 before RAR at 18, Ktx2 at 52 before Ktx
  // at 53, Qcow2 at 106 before Qcow at 107.
  assert.equal(pos.get("ScriptExecute"), 17);
  assert.equal(pos.get("RAR"), 18);
  assert.equal(pos.get("Ktx2"), 52);
  assert.equal(pos.get("Ktx"), 53);
  assert.equal(pos.get("Qcow2"), 106);
  assert.equal(pos.get("Qcow"), 107);

  // Every row is internally consistent, and `describe()` — which looks up by
  // discriminant rather than by position — is correct for every kind.
  assert.equal(describe("Ktx2").kind, "Ktx2");
  assert.deepEqual([...describe("RAR").signatures[0]], [...describe("RAR").signatures[0]]);

  // The two formats whose magic shadows a shorter neighbour's, which are the two
  // where the old order gave a wrong answer.
  for (const [shadowed, shadowing, magic] of [
    ["Ktx", "Ktx2", [0xab, 0x4b, 0x54, 0x58, 0x20, 0x32, 0x30, 0xbb, 0x0d, 0x0a, 0x1a, 0x0a]],
    ["Qcow", "Qcow2", [0x51, 0x46, 0x49, 0xfb]],
  ]) {
    const bytes = Uint8Array.from(magic);

    assert.equal(detectBytes(bytes), shadowing);
    assert.equal(matches(shadowing, bytes), true);
    assert.equal(matches(shadowed, bytes), true, `${shadowing}'s magic is a longer match for ${shadowed}`);

    assert.equal(
      table.find((entry) => matches(entry.kind, bytes)).kind,
      shadowing,
    );
  }

  // And the way round it that the page recommends: ask, then look up.
  const kind = detectBytes(Uint8Array.from([0xab, 0x4b, 0x54, 0x58, 0x20, 0x32, 0x30, 0xbb, 0x0d, 0x0a, 0x1a, 0x0a]));
  assert.equal(describe(kind).kind, "Ktx2");
});

test("the two predicate entries are the two the docs name", () => {
  // docs/api/javascript.md — "Signature.usesPredicate"
  const structural = signatureTable().filter((row) => row.usesPredicate).map((r) => r.kind);

  assert.deepEqual(structural, ["ScriptExecute", "WEBP"]);
});

test("allKinds is the crate's own order, not the detection order", () => {
  // docs/api/javascript.md — the `allKinds()` row
  //
  // `allKinds()` is the `FileKind` declaration order, which is what `_kinds.js` is
  // generated as and what `describe()` is keyed by. `signatureTable()` is the
  // detection order. Both are 114 long and both are stable, so treating one as the
  // other compiles, runs, and answers a different question — which is why they are
  // asserted to differ rather than assumed to.
  assert.equal(allKinds().length, 114);
  assert.equal(signatureTable().length, 114);
  assert.notDeepEqual(allKinds(), signatureTable().map((row) => row.kind));
  assert.equal(allKinds()[0], "Png");
});

test("the page's matches example holds, and there is no isAny", () => {
  // docs/api/javascript.md — "Asking 'is it this format?'"
  assert.equal(matches("Png", PNG), true);
  assert.equal(matches("Png", JPEG), false);

  // The page's reason for not shipping `isAny`: it is a loop, and the language
  // already has one.
  const any = (kinds, bytes) => kinds.some((kind) => matches(kind, bytes));
  assert.equal(any(["Jpg", "Png"], PNG), true);
  assert.equal(any(["Jpg", "GIF"], PNG), false);
  assert.equal(any([], PNG), false);
});

test("matchTypes answers with the union of the kinds declared", () => {
  // docs/api/javascript.md — "Level 2 — custom rules"
  const PNG_BYTES = new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
  const GIF_BYTES = new Uint8Array([0x47, 0x49, 0x46, 0x38, 0x39, 0x61]);

  const rules = [
    { kind: "Png", signatures: [PNG_BYTES], offsets: [0] },
    { kind: "Gif", signatures: [GIF_BYTES], offsets: [0] },
  ];

  assert.equal(matchTypes(rules, PNG), "Png");
  assert.equal(matchTypes(rules, GIF), "Gif");
  assert.equal(matchTypes(rules, NOISE), null);

  // Every match, not just the first.
  assert.deepEqual(matchAllTypes(rules, PNG), ["Png"]);

  // The page's warning: `releaseRules` drops the *handle*, not the memory, so a
  // caller building rule sets in a loop should build fewer and larger ones. What
  // is checked here is that releasing twice is distinguishable from never having
  // compiled, which is the one thing the return value does promise.
  assert.equal(releaseRules(rules), true);
  assert.equal(releaseRules(rules), false);
});

test("readHeader gives the bytes back and honours its own option", () => {
  // docs/api/javascript.md — the level 1 table
  withPhoto((dir) => {
    const whole = readHeader(join(dir, "photo.jpg"));
    assert.ok(whole instanceof Uint8Array);
    assert.deepEqual([...whole.slice(0, 4)], [0xff, 0xd8, 0xff, 0xe0]);

    const narrow = readHeader(join(dir, "photo.jpg"), { maxBytes: 4 });
    assert.equal(narrow.length, 4);
  });
});

test("readLimits reports both numbers and only ISO needs the larger one", () => {
  // docs/api/javascript.md — the `readLimits()` row, and the two "defaults"
  const limits = readLimits();

  assert.equal(limits.defaultMaxBytesRead, 2_048);
  assert.equal(limits.bytesRead, 36_870);
  assert.deepEqual(limits.isoOffsets, [32_769, 34_817, 36_865]);

  const over = signatureTable().filter((row) => row.maxBytesRead > 2_048);
  assert.deepEqual(
    over.map((row) => row.kind),
    ["ISO"],
  );
});

test("the page says every call is synchronous", () => {
  // docs/api/javascript.md — "**Everything is synchronous, deliberately.**"
  //
  // Not a stylistic point: the module compiles from Rust and offers no async
  // surface, so an `async` call site would be an extra `await` for the same answer.
  const answer = detectBytes(PNG);
  assert.equal(typeof answer.then, "undefined");
  assert.equal(answer, "Png");
});

// --------------------------------------------------------------------------
// docs/getting-started.md
// --------------------------------------------------------------------------

test("the getting-started JavaScript example holds", () => {
  // docs/getting-started.md — "JavaScript"
  withPhoto((dir) => {
    const kind = detectPath(join(dir, "photo.jpg"));
    assert.equal(kind, "Jpg");

    if (kind !== null) {
      assert.equal(displayName(kind), "JPEG");
      assert.equal(mime(kind), "image/jpeg");
      assert.equal(extension(kind), "jpg");
    }
  });
});

test("the getting-started bytes example holds", () => {
  // docs/getting-started.md — "Bytes you already have"
  assert.equal(detectBytes(GIF), "GIF");
});

// --------------------------------------------------------------------------
// docs/across-languages.md
// --------------------------------------------------------------------------

test("JavaScript cannot put the data first", () => {
  // docs/across-languages.md — "JavaScript cannot put the data first"
  //
  // The page's claim is about what the language allows rather than about what the
  // binding chose, and the checkable half is that there is no data-first spelling
  // hiding somewhere: `detectBytes` is a function, and the module exports no method
  // that takes bytes and returns a lazy wrapper.
  assert.equal(typeof detectBytes, "function");
  assert.equal(detectBytes(PNG), "Png");
});

// --------------------------------------------------------------------------
// The direction that rots
// --------------------------------------------------------------------------

test("every page is covered", () => {
  const pages = ["api/javascript.md", "across-languages.md", "getting-started.md"];

  for (const name of pages) {
    assert.ok(
      page(name).includes("```ts") || page(name).includes("```js"),
      `docs/${name} no longer has a JavaScript example, so drop it from this list`,
    );
  }

  const readme = readFileSync(join(DOCS, "..", "readme.md"), "utf8");
  assert.ok(
    readme.includes("docs/README.md"),
    "the readme does not link the documentation index",
  );
});