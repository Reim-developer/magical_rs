// Level 2: rules the caller builds, matched by the crate's own level 2 matcher.
//
// These are not reimplementations. `rules_finish` builds `MagicCustom` values and
// `rules_match` calls `match_types_custom`, so what these tests pin is that the
// bridge reaches the crate's matcher and does not decide anything itself.

import { test } from "node:test";
import assert from "node:assert/strict";

import * as api from "../index.js";
import { ascii, bytes } from "./fixtures.js";

const PNG = bytes(0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a);
const GIF = ascii("GIF89a");

/** Two rules, one per format, with names this package has never heard of. */
function twoRules() {
  return [
    { kind: "HousePng", signatures: [PNG], offsets: [0] },
    { kind: "HouseGif", signatures: [GIF], offsets: [0] },
  ];
}

test("a rule set matches its own bytes and nothing else", () => {
  const rules = twoRules();
  assert.equal(api.matchTypes(rules, PNG), "HousePng");
  assert.equal(api.matchTypes(rules, GIF), "HouseGif");
  assert.equal(api.matchTypes(rules, ascii("neither")), null);
  assert.equal(api.matchTypes(rules, new Uint8Array()), null);
});

test("a rule's kind need not be a file kind", () => {
  // The point of level 2 here: the rule set names its own formats, and the wasm
  // side stays ignorant of the names, which is why one compiled rule set serves
  // any strings at all.
  const rules = [{ kind: "Bananas", signatures: [PNG], offsets: [0] }];
  assert.equal(api.matchTypes(rules, PNG), "Bananas");
});

test("a rule may carry several signatures and several offsets", () => {
  const rules = [
    {
      kind: "Either",
      signatures: [PNG, GIF],
      offsets: [0, 8],
    },
  ];
  assert.equal(api.matchTypes(rules, PNG), "Either");
  assert.equal(api.matchTypes(rules, GIF), "Either");
  // PNG's magic moved to offset 8.
  const shifted = new Uint8Array(16);
  shifted.set(PNG, 8);
  assert.equal(api.matchTypes(rules, shifted), "Either");
  // At offset 1, which the rule does not ask about.
  const wrong = new Uint8Array(16);
  wrong.set(PNG, 1);
  assert.equal(api.matchTypes(rules, wrong), null);
});

test("matchAllTypes reports every rule that matched, in order", () => {
  // Two rules given the same signature. Both match, which is the only thing that
  // separates `matchAllTypes` from `matchTypes` returning the first and stopping.
  const rules = [
    { kind: "First", signatures: [PNG], offsets: [0] },
    { kind: "Second", signatures: [PNG], offsets: [0] },
  ];
  assert.deepEqual(api.matchAllTypes(rules, PNG), ["First", "Second"]);
  assert.deepEqual(api.matchAllTypes(rules, GIF), []);
  // And it is order, not the order the table happens to hold.
  assert.deepEqual(api.matchAllTypes(rules, PNG).reverse(), ["Second", "First"]);
});

test("the first matching rule wins, in declaration order", () => {
  const rules = [
    { kind: "Short", signatures: [PNG.subarray(0, 4)], offsets: [0] },
    { kind: "Long", signatures: [PNG], offsets: [0] },
  ];
  assert.equal(api.matchTypes(rules, PNG), "Short");
  assert.deepEqual(api.matchAllTypes(rules, PNG), ["Short", "Long"]);
});

test("a rule set is compiled once per array and reused", () => {
  const rules = twoRules();
  // The cache is a WeakMap keyed on the array, so the same rules in a loop
  // compile once. Observable only indirectly — through `releaseRules`, which is
  // the only handle a caller has on the compiled copy.
  for (let i = 0; i < 100; i++) assert.equal(api.matchTypes(rules, PNG), "HousePng");
  assert.equal(api.releaseRules(rules), true, "there should be a compiled rule set");
  assert.equal(api.releaseRules(rules), false, "and only one");
});

test("a released rule set compiles again on next use", () => {
  const rules = twoRules();
  assert.equal(api.matchTypes(rules, PNG), "HousePng");
  assert.equal(api.releaseRules(rules), true);
  // Releasing drops the handle, so the next call rebuilds. If this stopped
  // working, `releaseRules` would be a function that quietly disabled the API.
  assert.equal(api.matchTypes(rules, PNG), "HousePng");
  assert.equal(api.releaseRules(rules), true);
});

test("two rule sets over the same bytes stay independent", () => {
  const first = twoRules();
  const second = [{ kind: "OnlyPng", signatures: [PNG], offsets: [0] }];
  assert.equal(api.matchTypes(first, GIF), "HouseGif");
  assert.equal(api.matchTypes(second, GIF), null);
  assert.equal(api.matchTypes(first, PNG), "HousePng");
  assert.equal(api.matchTypes(second, PNG), "OnlyPng");
});

test("an incomplete rule is rejected, naming the rule and the kind", () => {
  const noSignature = [{ kind: "Nothing", signatures: [], offsets: [0] }];
  assert.throws(() => api.matchTypes(noSignature, PNG), {
    name: "TypeError",
    message: /rules\[0\] \(Nothing\) needs at least one signature/,
  });

  const noOffset = [{ kind: "Nowhere", signatures: [PNG], offsets: [] }];
  assert.throws(() => api.matchTypes(noOffset, PNG), {
    name: "TypeError",
    message: /rules\[0\] \(Nowhere\) needs at least one offset/,
  });

  // Not just the first rule: the second one is incomplete, and the message has to
  // be able to say so.
  const secondBroken = [
    { kind: "Fine", signatures: [PNG], offsets: [0] },
    { kind: "Broken", signatures: [GIF], offsets: [] },
  ];
  assert.throws(() => api.matchTypes(secondBroken, PNG), {
    name: "TypeError",
    message: /rules\[1\] \(Broken\) needs at least one offset/,
  });
});

test("an empty rule set is refused rather than answering null forever", () => {
  assert.throws(() => api.matchTypes([], PNG), {
    name: "RangeError",
    message: /rules is empty/,
  });
});

test("a malformed rule set is refused before it reaches the module", () => {
  assert.throws(() => api.matchTypes("nope", PNG), {
    name: "TypeError",
    message: /rules must be an array, got string/,
  });
  assert.throws(() => api.matchTypes([null], PNG), {
    name: "TypeError",
    message: /rules\[0\] must be an object, got null/,
  });
  assert.throws(() => api.matchTypes([{ kind: "", signatures: [PNG], offsets: [0] }], PNG), {
    name: "TypeError",
    message: /rules\[0\]\.kind must be a non-empty string/,
  });
  assert.throws(
    () => api.matchTypes([{ kind: "X", signatures: ["PNG"], offsets: [0] }], PNG),
    { name: "TypeError", message: /rules\[0\]\.signatures\[0\] needs a Uint8Array/ },
  );
  assert.throws(() => api.matchTypes([{ kind: "X", signatures: [PNG], offsets: [-1] }], PNG), {
    name: "RangeError",
    message: /rules\[0\]\.offsets\[0\] must be a non-negative safe integer/,
  });
});

test("an answer buffer is never confused for the input", () => {
  // `matchAllTypes` writes its index list *after* the input bytes. A rule set with
  // a wide signature and a buffer long enough to reach the answer region would
  // match its own result if the two were allowed to overlap.
  const filler = new Uint8Array(4096);
  filler.set(PNG, 0);
  const rules = [{ kind: "Wide", signatures: [filler], offsets: [0] }];
  assert.deepEqual(api.matchAllTypes(rules, filler), ["Wide"]);
});

test("matchAllTypes on a large buffer still reads its own answer", () => {
  // Same hazard as the memory-growth case in detect.test.js, from the other side:
  // the answer is read out of linear memory after the call, and the call may have
  // grown it. A stale view here reads zeroes.
  const big = new Uint8Array(4 * 1024 * 1024);
  big.set(PNG);
  const rules = [
    { kind: "A", signatures: [PNG], offsets: [0] },
    { kind: "B", signatures: [PNG], offsets: [0] },
  ];
  assert.deepEqual(api.matchAllTypes(rules, big), ["A", "B"]);
});

test("a zero-length signature matches everything, which is a real answer", () => {
  // Worth pinning because the loader passes pointer 0 for an empty buffer, and
  // `from_raw_parts` rejects a null pointer even for a zero-length slice. The Rust
  // side handles it; this checks the JavaScript side can express the rule at all.
  const rules = [{ kind: "Anything", signatures: [new Uint8Array()], offsets: [0] }];
  assert.equal(api.matchTypes(rules, PNG), "Anything");
  assert.equal(api.matchTypes(rules, new Uint8Array()), "Anything");
});

test("a large offset is honoured", () => {
  const rules = [{ kind: "Far", signatures: [PNG], offsets: [100_000] }];
  const buffer = new Uint8Array(100_008);
  buffer.set(PNG, 100_000);
  assert.equal(api.matchTypes(rules, buffer), "Far");
  assert.equal(api.matchTypes(rules, new Uint8Array(100_008)), null);
});
