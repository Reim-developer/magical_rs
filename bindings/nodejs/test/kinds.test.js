// The drift test: every generated name, against the discriminant the compiled
// module actually reports.
//
// This is the test that makes `_kinds.js` safe to check in. `FileKind` is a
// fieldless enum, so a variant's value is its position in the declaration in
// `src/magical/magic.rs`. JavaScript cannot see that. Reordering the enum would
// compile cleanly, ship cleanly, and turn every `detectBytes` result into the
// wrong name — with no error anywhere, just confidently mislabelled files.
//
// So the generated list is not trusted: it is compared against the module.

import { test } from "node:test";
import assert from "node:assert/strict";

import * as api from "../index.js";
import { FILE_KIND_INDICES, FILE_KIND_NAMES } from "../_kinds.js";
import { kindMatchesAt, whichKindAt } from "../_wasm.js";
import { ascii } from "./fixtures.js";

test("the format count is still 114", () => {
  // Pinned because the project has a standing decision to keep it there, and a
  // silent 115 is exactly the change that should take a deliberate edit rather
  // than arrive by accident. `scripts/gen_formats.mjs` checks the same number
  // against formats.json; this checks it against the module.
  assert.equal(api.allKinds().length, 114);
  assert.equal(FILE_KIND_NAMES.length, 114);
  assert.equal(FILE_KIND_INDICES.size, 114);
});

test("allKinds(), FILE_KIND_NAMES and FILE_KIND_INDICES agree", () => {
  assert.deepEqual(api.allKinds(), [...FILE_KIND_NAMES]);
  assert.deepEqual(
    [...FILE_KIND_INDICES.keys()],
    [...FILE_KIND_NAMES],
    "the map's insertion order and the array's order must be the same list",
  );
  // A fresh array each call, so a caller's `sort()` cannot reorder it for the next
  // caller. `allKinds` hands out `FILE_KIND_NAMES` by copy precisely so this holds.
  const first = api.allKinds();
  first.reverse();
  assert.deepEqual(api.allKinds(), [...FILE_KIND_NAMES]);
});

test("FileKind names its 114 formats, and only those", () => {
  assert.equal(Object.keys(api.FileKind).length, 114);
  for (const [key, value] of Object.entries(api.FileKind)) {
    assert.equal(key, value, "FileKind maps a format name to itself");
    assert.ok(api.isFileKind(value), `${value} should be a format name`);
  }
  assert.equal(api.isFileKind("Nonsense"), false);
  assert.equal(api.isFileKind(0), false);
  assert.equal(api.isFileKind(undefined), false);
});

test("every generated index is what the module uses for that name", () => {
  // The load-bearing assertion. For each name: ask the module to report whether
  // *that* format's rule matches a buffer only that format matches, and ask it
  // under the index `_kinds.js` claims. If the two disagree, a format is
  // detectable under someone else's name.
  //
  // Built from each format's own first signature at its own first offset, so this
  // needs no hand-written fixture and cannot rot.
  //
  // Predicate entries are skipped, and not as a convenience: `Magic::matches`
  // dispatches on `MatchRules`, so for `ScriptExecute` it calls the crate's
  // predicate function and never looks at the `#!` bytes this builds. A predicate
  // that required only its own signature would make `#!` match any file beginning
  // with a hash and a bang. The predicate entries are checked on their own terms
  // two tests below.
  let checked = 0;
  for (const name of FILE_KIND_NAMES) {
    const claimed = FILE_KIND_INDICES.get(name);
    assert.equal(typeof claimed, "number", `${name} has no index`);
    assert.ok(
      Number.isInteger(claimed) && claimed >= 0 && claimed < 114,
      `${name} has index ${claimed}, which is not a discriminant`,
    );

    const rule = api.describe(name);
    if (rule.usesPredicate || rule.signatures.length === 0) continue;

    const offset = rule.offsets[0];
    const buffer = new Uint8Array(offset + rule.signatures[0].length);
    buffer.set(rule.signatures[0], offset);

    assert.equal(
      api.matches(name, buffer),
      true,
      `${name}'s own signature does not match ${name}, so the table and the matcher disagree`,
    );
    assert.equal(
      kindMatchesAt(claimed, buffer),
      api.matches(name, buffer),
      `${name} claims index ${claimed}, and the module's answer under that index differs`,
    );
    checked++;
  }
  // 114 formats, less the two predicate entries and nothing else. If this number
  // moves, a format stopped having a byte signature and the reason is not written
  // down anywhere.
  assert.equal(checked, 112);
});

test("the two predicate entries are the ones the table says they are", () => {
  const predicates = api.allKinds().filter((kind) => api.describe(kind).usesPredicate);
  assert.deepEqual(predicates, ["ScriptExecute", "WEBP"]);
  // WEBP has no signature at all; ScriptExecute has a prefilter it does not use
  // on its own. Both are exercised through the real predicate, since that is what
  // `matches` actually calls for them.
  assert.equal(api.matches("ScriptExecute", ascii("#!/bin/sh\necho hi\n")), true);
  assert.equal(api.matches("ScriptExecute", ascii("#!")), false, "a lone `#!` is not a script");
  assert.equal(api.matches("ScriptExecute", ascii("no shebang here\n")), false);
  // WEBP's predicate is a RIFF/WEBP container check, not a byte signature: the
  // four bytes at 4..8 are the container's own declared size, and the crate
  // requires it to exceed the four form bytes. A fixture that left those bytes
  // zero — the obvious thing to write — declares a zero-length file and is
  // correctly rejected.
  const webp = new Uint8Array(64);
  webp.set(ascii("RIFF"), 0);
  webp.set(Uint8Array.from([0x64, 0x00, 0x00, 0x00]), 4); // 100, little-endian
  webp.set(ascii("WEBP"), 8);
  assert.equal(api.matches("WEBP", webp), true);
  // Right magic, wrong form: WAVE is a RIFF container too.
  assert.equal(api.matches("WEBP", ascii("RIFF" + "d\0\0\0" + "WAVE" + "fmt ")), false);
  // Right magic, form size of 4 — the "no payload" case the crate excludes.
  assert.equal(api.matches("WEBP", ascii("RIFF" + "\x04\0\0\0" + "WEBP")), false);
  // Too short to have a size field at all.
  assert.equal(api.matches("WEBP", ascii("RIFF")), false);
});

test("the module's discriminant and describe()'s name are the same format", () => {
  // The reverse direction of the test above, and the one that would catch a
  // *reordering* specifically. `whichKindAt` returns a raw discriminant for the
  // first format whose rule matches; `describe()` names a format. Pairing them
  // over a buffer every format rejects must still be possible, so this uses a
  // format with one unambiguous signature and checks the name comes back intact
  // through the public API — and that a discriminant's name is stable.
  for (const name of ["Png", "ELF", "WASM", "Class", "OpenTypeFont"]) {
    const rule = api.describe(name);
    const offset = rule.offsets[0];
    const buffer = new Uint8Array(offset + rule.signatures[0].length);
    buffer.set(rule.signatures[0], offset);
    assert.equal(
      whichKindAt(buffer),
      FILE_KIND_INDICES.get(name),
      `${name} detects under a different discriminant than _kinds.js claims`,
    );
    assert.equal(api.detectBytes(buffer), name);
  }
});

test("WEBP is the one format with no signature, and it is a predicate", () => {
  // Measured from the table rather than assumed: exactly one entry carries no
  // bytes at all, and it is decided by a function. The Rust test
  // `a_predicate_entry_may_still_carry_a_signature` asserts the same two facts
  // from the other side of the boundary.
  const without = api.allKinds().filter((kind) => api.describe(kind).signatures.length === 0);
  assert.deepEqual(without, ["WEBP"]);
  assert.equal(api.describe("WEBP").usesPredicate, true);
});

test("ScriptExecute is a predicate that also carries a signature", () => {
  // The reason this binding does not blank the signature of every predicate
  // entry, which the Python binding does. `#!` is a real prefilter and the crate
  // uses it; reporting only `usesPredicate` would be a worse description of how
  // detection actually works.
  const rule = api.describe("ScriptExecute");
  assert.equal(rule.usesPredicate, true);
  assert.deepEqual([...rule.signatures[0]], [...ascii("#!")]);
  assert.deepEqual([...rule.offsets], [0]);
  // And it is genuinely a *prefilter*: the signature alone is not sufficient, which
  // is why the sweep above has to skip this entry rather than trust it.
  assert.equal(api.matches("ScriptExecute", ascii("#!")), false);
  assert.equal(api.matches("ScriptExecute", ascii("#!/bin/sh\n")), true);
});

test("every signature and offset is a usable Uint8Array and number", () => {
  for (const name of api.allKinds()) {
    const rule = api.describe(name);
    for (const signature of rule.signatures) {
      assert.ok(signature instanceof Uint8Array, `${name} has a non-Uint8Array signature`);
      assert.ok(signature.length > 0, `${name} has a zero-length signature`);
    }
    for (const offset of rule.offsets) {
      assert.ok(
        Number.isSafeInteger(offset) && offset >= 0,
        `${name} has offset ${offset}, which is not a byte position`,
      );
    }
    assert.ok(Number.isSafeInteger(rule.maxBytesRead) && rule.maxBytesRead > 0);
    // A rule with signatures must have somewhere to look for them. Not a
    // universal truth — `WEBP` has neither, being a predicate — but a rule that
    // has bytes and no offset can never match, and that is a table bug.
    if (rule.signatures.length > 0) {
      assert.ok(rule.offsets.length > 0, `${name} has signatures but no offset`);
    }
  }
});

test("describe() returns copies, so a caller cannot corrupt the table", () => {
  const first = api.describe("Png");
  first.offsets.push(999);
  first.signatures[0].fill(0);
  const second = api.describe("Png");
  assert.deepEqual([...second.offsets], [0]);
  assert.deepEqual([...second.signatures[0]], [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
});

test("signatureTable() is in detection order, one row per format", () => {
  const table = api.signatureTable();

  // One row per format, and the same set of formats — asserted as a set rather
  // than as a sequence, because the sequence is the thing under test elsewhere.
  assert.equal(table.length, 114);
  assert.deepEqual(
    [...table.map((rule) => rule.kind)].sort(),
    [...FILE_KIND_NAMES].sort(),
  );

  // A fresh array every call, so a caller's `sort()` is their own business.
  table.reverse();
  assert.notDeepEqual(
    api.signatureTable().map((rule) => rule.kind),
    table.map((rule) => rule.kind),
  );
});

test("walking signatureTable() reproduces detectBytes", () => {
  // The order is only worth anything if it is the order detection walks, so this
  // checks the property rather than a copied list of names. A copied list would
  // pass just as happily if the table and the list were wrong in the same way,
  // which is exactly how this went unnoticed: the previous test asserted that
  // `signatureTable()` was in *declaration* order, and it was, and that was the
  // bug. See issue #24.
  //
  // Every `(signature, offset)` pair of every format is tried, so this is not a
  // sample. `detectBytes` declines `ScriptExecute`'s own bytes, because its
  // predicate needs more of the line than the two it carries; that is one form out
  // of 144 and it is skipped rather than counted as agreement.
  const table = api.signatureTable();

  let tried = 0;
  let declined = 0;
  for (const kind of api.allKinds()) {
    const rule = api.describe(kind);
    for (const signature of rule.signatures) {
      for (const offset of rule.offsets.length ? rule.offsets : [0]) {
        const bytes = new Uint8Array(api.neededBytes());
        bytes.set(signature, offset);

        const detected = api.detectBytes(bytes);
        if (detected === null) {
          declined += 1;
          continue;
        }

        const walked = table.find((row) => api.matches(row.kind, bytes))?.kind;
        assert.equal(
          walked,
          detected,
          `${kind}'s own bytes are detected as ${detected} but the table walk says ${walked}`,
        );
        tried += 1;
      }
    }
  }

  assert.equal(tried + declined, 144, "every canonical form in the table was exercised");
  assert.ok(tried >= 143, `only ${tried} forms produced an answer to compare`);
});

test("the formats whose magic shadows another's are ordered the way detection orders them", () => {
  // Three pairs, and the reason each is ordered the way it is. These are facts
  // about the formats rather than about this table, which is what makes them safe
  // to hard-code where a copy of the whole 114 would not be.
  const at = new Map(api.signatureTable().map((row, i) => [row.kind, i]));

  // `Ktx`'s magic is a byte-for-byte prefix of `Ktx2`'s, and `Qcow`'s of `Qcow2`'s,
  // so asking the shorter one first would make the longer one undetectable. These
  // are the two formats where getting the order wrong produces a wrong answer, and
  // they are the reason the walk above is checked rather than assumed.
  assert.ok(at.get("Ktx2") < at.get("Ktx"), `Ktx2 at ${at.get("Ktx2")}, Ktx at ${at.get("Ktx")}`);
  assert.ok(at.get("Qcow2") < at.get("Qcow"), `Qcow2 at ${at.get("Qcow2")}, Qcow at ${at.get("Qcow")}`);

  // `ScriptExecute` and `RAR` do not shadow each other — `#!` and `Rar!` cannot
  // both match — but the shebang rule is asked first so that `#!AMR`, which is the
  // literal magic of AMR audio, is not claimed by the script rule before its own
  // predicate has had a chance to decline it.
  assert.ok(
    at.get("ScriptExecute") < at.get("RAR"),
    `ScriptExecute at ${at.get("ScriptExecute")}, RAR at ${at.get("RAR")}`,
  );
});

test("signatureTable() is not allKinds(), and allKinds() is the declaration order", () => {
  // The two lists are both 114 long and both stable, so confusing them compiles,
  // runs, and answers a different question. `allKinds()` is the enum's order and
  // should stay that way: it is what `_kinds.js` is generated as, and it is what
  // `describe()` and `FILE_KIND_INDICES` are keyed by.
  const table = api.signatureTable().map((row) => row.kind);

  assert.notDeepEqual(table, [...FILE_KIND_NAMES]);
  assert.deepEqual(api.allKinds(), [...FILE_KIND_NAMES]);
  assert.equal(new Set(table).size, 114, "the table lists each format exactly once");
});

test("an unknown format name is rejected, and the message says which", () => {
  assert.throws(() => api.describe("NotAFormat"), {
    name: "RangeError",
    message: /"NotAFormat" is not a file kind/,
  });
  assert.throws(() => api.matches("Jpeg", new Uint8Array()), {
    name: "RangeError",
    message: /"Jpeg" is not a file kind/,
  });
});
