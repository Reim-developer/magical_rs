// The format metadata: display name, MIME type, extension.
//
// These are the answers a caller reaches for after detection, and none of them
// comes out of the wasm module. `formats.json` is generated into `_kinds.js` by
// `scripts/gen_formats.mjs` from the same table the Rust `FileKind::mime` and the
// Python `FileKind.mime` are generated from, and this file checks that the
// answers are internally consistent and that the API refuses a name it cannot
// answer for -- rather than checking any particular value, which is the
// dataset's business and not this binding's.

import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import * as api from "../index.js";
import {
  FILE_KIND_DISPLAY_NAME,
  FILE_KIND_EXTENSION,
  FILE_KIND_INDICES,
  FILE_KIND_MIME,
  FILE_KIND_NAMES,
} from "../_kinds.js";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");

test("every format has a display name, and the API answers for all of them", () => {
  for (const kind of api.allKinds()) {
    const name = api.displayName(kind);
    assert.equal(typeof name, "string", `${kind} has no display name`);
    assert.ok(name.length > 0, `${kind}'s display name is empty`);
    assert.equal(name, FILE_KIND_DISPLAY_NAME[kind]);
  }
});

test("a display name is never the variant name in lower case", () => {
  // Not a style rule: a display name that were mechanically derived from the
  // variant would mean the dataset has no real value for the field, and the one
  // thing the field is *for* -- printing something a person recognises -- would
  // silently be lost. `PkgZip` is the load-bearing case: its variant says zip and
  // its display name says which three formats share that magic.
  assert.equal(api.displayName("Png"), "PNG");
  assert.equal(api.displayName("PkgZip"), "Zip / JAR / APK");
  assert.equal(api.displayName("Class"), "Java class");

  // Measured rather than assumed: how many display names are exactly the variant
  // with its first letter upper-cased. A handful is fine -- `Bitmap` really is
  // called Bitmap -- and "most of them" is the failure this is here to catch.
  const mechanical = api.allKinds().filter(
    (kind) => api.displayName(kind) === kind[0].toUpperCase() + kind.slice(1),
  );
  assert.ok(
    mechanical.length < api.allKinds().length / 2,
    `${mechanical.length} of 114 display names are just the variant name capitalised`,
  );
});

test("a MIME type is a string or null, never a guess", () => {
  for (const kind of api.allKinds()) {
    const value = api.mime(kind);
    if (value === null) continue;
    assert.equal(typeof value, "string", `${kind}'s MIME type is neither a string nor null`);
    // `type/subtype` and nothing else. A value like `image/png; charset=binary`
    // would be a content type rather than a media type, and the two are
    // different fields of an HTTP header; a caller that needs one can add it.
    assert.match(
      value,
      /^[a-z0-9][a-z0-9!#$&^_.+-]*\/[a-z0-9][a-z0-9!#$&^_.+-]*$/,
      `${kind}'s MIME type ${JSON.stringify(value)} is not a media type`,
    );
    assert.equal(value, value.toLowerCase(), `${kind}'s MIME type is not lower case`);
    assert.ok(!value.includes(" "), `${kind}'s MIME type has a space in it`);
  }
});

test("the 16 formats with no registered MIME type answer null", () => {
  // Counted from the table rather than hard-coded per format, so a format
  // gaining a verified type does not fail this -- but a format *losing* one does,
  // which is the direction that matters: a null is visible to a caller, an
  // invented type is not.
  const without = api.allKinds().filter((kind) => api.mime(kind) === null);
  assert.equal(without.length, 16);
  assert.equal(FILE_KIND_MIME.ELF, null);
  assert.equal(FILE_KIND_MIME.AceCompressed, null);
  // And it is null rather than a fallback the project invented: a MIME type gets
  // served to a browser, so `application/octet-stream` is a decision about the
  // caller's response to opaque data and belongs at the call site.
  assert.ok(
    !Object.values(FILE_KIND_MIME).includes("application/octet-stream"),
    "no format may be given a made-up MIME type",
  );
});

test("an extension has no leading dot and no path separator", () => {
  for (const kind of api.allKinds()) {
    const value = api.extension(kind);
    if (value === null) continue;
    assert.equal(typeof value, "string", `${kind}'s extension is neither a string nor null`);
    assert.equal(value, value.toLowerCase(), `${kind}'s extension is not lower case`);
    assert.ok(!value.startsWith("."), `${kind}'s extension ${value} starts with a dot`);
    assert.ok(!value.includes("/"), `${kind}'s extension ${value} has a path separator`);
    assert.ok(!value.includes("\\"), `${kind}'s extension ${value} has a path separator`);
    assert.match(
      value,
      /^[a-z0-9][a-z0-9+._-]*$/,
      `${kind}'s extension ${JSON.stringify(value)} is not a file extension`,
    );
  }
  assert.equal(api.extension("Png"), "png");
  assert.equal(api.extension("SevenZip"), "7z");
});

test("the generated tables and the API cannot drift apart", () => {
  // The tables are the storage and the three functions are the interface. A
  // change to one that is not the other is invisible until a caller reads the
  // other, so this asks both in the same loop rather than trusting either.
  for (const kind of api.allKinds()) {
    assert.equal(api.displayName(kind), FILE_KIND_DISPLAY_NAME[kind]);
    assert.equal(api.mime(kind), FILE_KIND_MIME[kind]);
    assert.equal(api.extension(kind), FILE_KIND_EXTENSION[kind]);
  }

  // And the tables are keyed by the same 114 names the ABI is keyed by, which is
  // what makes `FILE_KIND_DISPLAY_NAME[kind]` safe rather than a lookup that
  // answers `undefined` for a format added to one list and not the other.
  assert.deepEqual(Object.keys(FILE_KIND_DISPLAY_NAME), [...FILE_KIND_NAMES]);
  assert.deepEqual(Object.keys(FILE_KIND_MIME), [...FILE_KIND_NAMES]);
  assert.deepEqual(Object.keys(FILE_KIND_EXTENSION), [...FILE_KIND_NAMES]);
  assert.equal(Object.keys(FILE_KIND_MIME).length, FILE_KIND_INDICES.size);
});

test("the tables are frozen, so a caller cannot rewrite the metadata", () => {
  // Same reason `describe()` copies its arrays: these are module-level constants
  // shared by every caller, and a `sort()` or a deletion on one of them would be
  // invisible to the next call. In strict mode -- which every ES module is --
  // the assignment throws rather than failing silently.
  for (const table of [FILE_KIND_DISPLAY_NAME, FILE_KIND_MIME, FILE_KIND_EXTENSION]) {
    assert.ok(Object.isFrozen(table));
    assert.throws(() => {
      "use strict";
      table.Png = "tampered";
    }, TypeError);
  }
  assert.equal(api.mime("Png"), "image/png");
});

test("an unknown format name is refused by all three, and says so", () => {
  // A `Map` would answer `undefined` here, and `mime(await guessFormat(f))`
  // would then be `undefined` for two unrelated reasons: "not a format we know"
  // and "a format with no registered type". Those are worth telling apart, and
  // the first is worth a throw.
  for (const call of [api.displayName, api.mime, api.extension]) {
    assert.throws(() => call("NotAFormat"), {
      name: "RangeError",
      message: /"NotAFormat" is not a file kind/,
    });
  }
  // The same error as `describe()` and `matches()`, which is what lets a caller
  // catch one thing and handle "that is not a format" once.
  assert.throws(() => api.displayName("NotAFormat"), (error) => {
    assert.throws(() => api.describe("NotAFormat"), { message: error.message });
    return true;
  });

  // A non-string is a `TypeError` rather than the `RangeError` above, because it
  // is a different mistake: not "a name that is wrong" but "not a name at all".
  for (const call of [api.displayName, api.mime, api.extension]) {
    for (const value of [undefined, null, 0, 42, {}, []]) {
      assert.throws(() => call(value), { name: "TypeError" }, `${String(value)} should be a TypeError`);
    }
  }
});

test("the answers are the ones formats.json holds", () => {
  // The check that makes `formats.json` worth having: this binding, the Rust
  // `kinds_meta.rs` and the Python `_kinds.py` are generated from it, so a value
  // can only be wrong in all three at once if the dataset itself is wrong. This
  // reads the dataset directly rather than trusting the generator, because a
  // generator that reads the wrong file and emits a consistent table is the one
  // failure the other three tests cannot see.
  const dataset = JSON.parse(readFileSync(join(repoRoot, "formats.json"), "utf8"));
  assert.equal(dataset.formats.length, 114);

  const byVariant = new Map(dataset.formats.map((format) => [format.variant, format]));
  assert.equal(byVariant.size, 114, "formats.json has a duplicate variant");

  for (const kind of api.allKinds()) {
    const format = byVariant.get(kind);
    assert.ok(format, `formats.json has no entry for ${kind}`);
    assert.equal(api.displayName(kind), format.name, `${kind}'s display name`);
    assert.equal(api.mime(kind), format.mime, `${kind}'s MIME type`);
    assert.equal(api.extension(kind), format.extension, `${kind}'s extension`);
    // `abi_order` is the JavaScript side's ABI, so a hand-edited dataset that
    // renumbered it would make one format answer under another's name -- and
    // `test/kinds.test.js` catches that against the compiled module, which is the
    // only place it can be caught.
    assert.equal(FILE_KIND_INDICES.get(kind), format.abi_order, `${kind}'s index`);
  }
});

test("the dataset's positions are a permutation of 0..113", () => {
  // Checked here rather than only in the generator, because the generator
  // rewriting `_kinds.js` from a bad dataset is how a bad dataset becomes
  // permanent: the generated file is checked in, so the next contributor sees a
  // consistent-looking `_kinds.js` and nothing to re-run.
  const dataset = JSON.parse(readFileSync(join(repoRoot, "formats.json"), "utf8"));
  const orders = dataset.formats.map((format) => format.abi_order).sort((a, b) => a - b);
  assert.deepEqual(
    orders,
    Array.from({ length: 114 }, (_, at) => at),
    "abi_order is not 0..113 with no gaps or repeats",
  );
});

test("a metadata table with a hole in it would have been caught", () => {
  // Not a mutation test -- the real ones are in the commit message. This asserts
  // the property the mutation would break: every name the ABI knows has an entry
  // in every metadata table, and an entry in a metadata table belongs to a name
  // the ABI knows. Both directions, because either one alone is satisfiable by a
  // table that is merely the same wrong length.
  const names = new Set(FILE_KIND_NAMES);
  for (const table of [FILE_KIND_DISPLAY_NAME, FILE_KIND_MIME, FILE_KIND_EXTENSION]) {
    const keys = Object.keys(table);
    assert.equal(keys.length, names.size);
    for (const key of keys) assert.ok(names.has(key), `${key} is in a table but not the ABI`);
    for (const name of names) assert.ok(name in table, `${name} is in the ABI but not a table`);
  }
});
