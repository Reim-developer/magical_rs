// The fluent spelling: `detected(path).is("Png")`.
//
// These are the assertions that could not be made against the free functions, which
// is most of them. `Detected` promises three things the rest of the package does not:
// that nothing is read until a member needs an answer, that the bytes are read once
// however many members are read off the result, and that `within` re-asks rather than
// answering from a cache keyed on a window that has changed. Each of those is checked
// by counting, because a test that only checks the answers passes just as happily when
// the file is read on every call.

import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { Detected, detectBytes, detected, matches, neededBytes } from "../index.js";

const JPEG = Uint8Array.from([0xff, 0xd8, 0xff, 0xe0, ...new Array(32).fill(0)]);
const GIF = Uint8Array.from([0x47, 0x49, 0x46, 0x38, 0x39, 0x61]);
const NOISE = new Uint8Array(64).fill(0x2e); // ".", which no signature starts with
const ISO_OFFSET = 36_865;

/** An ISO 9660 image: the magic sits past a 2,048-byte window, which is the point. */
function iso(size = 40_000) {
  const bytes = new Uint8Array(size);
  bytes.set([0x43, 0x44, 0x30, 0x30, 0x31], ISO_OFFSET); // "CD001"
  return bytes;
}

/** A reader that counts how many times it was asked for bytes. */
function countingReader(bytes = JPEG) {
  const reads = [];
  return {
    reads,
    read(size) {
      reads.push(size);
      return bytes.subarray(0, Math.min(size, bytes.length));
    },
  };
}

/** A scratch directory holding the file the pages read by name. */
function withPhoto(run) {
  const dir = mkdtempSync(join(tmpdir(), "magical-fluent-"));
  try {
    writeFileSync(join(dir, "photo.jpg"), JPEG);
    run(dir);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

test("the page's first example holds", () => {
  // docs/getting-started.md — "The data-first spelling"
  withPhoto((dir) => {
    assert.equal(detected(join(dir, "photo.jpg")).kind, "Jpg");
    assert.equal(detected(join(dir, "photo.jpg")).is("Jpg"), true);
  });
});

test("kind is the same answer detectBytes gives", () => {
  // The claim is that this is a spelling and not a second opinion, so it is checked
  // as an equality against the function rather than against a fixture. The exhaustive
  // sweep over all 144 canonical forms is in `kinds.test.js`; what matters here is
  // that the two paths share one answer.
  for (const bytes of [JPEG, GIF, iso(), NOISE]) {
    assert.equal(detected(bytes).kind, detectBytes(bytes));
  }

  // And the window is honoured the same way `detectBytes` honours its option.
  assert.equal(detected(iso()).within(2_048).kind, detectBytes(iso(), { maxBytesRead: 2_048 }));
  assert.equal(detected(iso(), { maxBytesRead: 2_048 }).kind, detectBytes(iso(), { maxBytesRead: 2_048 }));
});

test("is() asks about one format, which is not the same question as kind", () => {
  // `Ktx`'s magic is a byte-for-byte prefix of `Ktx2`'s, so a KTX2 file is reported
  // as `Ktx2` and is still "yes, it is Ktx" to `is`. This is the whole reason `is`
  // exists, and it is the one case where the two answers differ.
  const ktx2 = Uint8Array.from([
    0xab, 0x4b, 0x54, 0x58, 0x20, 0x32, 0x30, 0xbb, 0x0d, 0x0a, 0x1a, 0x0a,
  ]);

  const what = detected(ktx2);
  assert.equal(what.kind, "Ktx2");
  assert.equal(what.is("Ktx2"), true);
  assert.equal(what.is("Ktx"), true, "Ktx's own rule matches these bytes too");
  assert.equal(what.kind === "Ktx", false);

  // And the free function agrees with the method, which is the point of the pair.
  assert.equal(what.is("Ktx"), matches("Ktx", ktx2));
});

test("isAny takes varargs or one iterable, and never iterates a string", () => {
  const what = detected(JPEG);

  assert.equal(what.isAny("Png", "Jpg"), true);
  assert.equal(what.isAny(["Png", "GIF", "Jpg"]), true);
  assert.equal(what.isAny(new Set(["Jpg"])), true);
  assert.equal(what.isAny("Png", "GIF"), false);

  // The string case is the one that passes a casual test: a `FileKind` is a string and
  // a string is iterable as characters, so treating the single argument as a
  // collection would test `"J"`, `"p"`, `"g"` and find nothing.
  assert.equal(what.isAny("Jpg"), true, "one kind, not three letters");
  assert.equal(what.isAny("Png"), false);

  // A file cannot be any of nothing.
  assert.equal(what.isAny(), false);
  assert.equal(what.isAny([]), false);

  // A generator is consumed once, not once per format.
  let pulls = 0;
  function* once() {
    for (const kind of ["Png", "Jpg"]) {
      pulls += 1;
      yield kind;
    }
  }
  assert.equal(what.isAny(once()), true);
  assert.equal(pulls, 2);
});

test("nothing is read until a member needs an answer", () => {
  const reader = countingReader();
  const what = detected(reader);

  assert.deepEqual(reader.reads, [], "building one is free");

  // Passing it on is still free.
  const passed = what;
  assert.equal(passed, what);
  assert.deepEqual(reader.reads, []);

  assert.equal(what.kind, "Jpg");
  assert.equal(reader.reads.length, 1);
});

test("the bytes are read once, however many members are read", () => {
  const reader = countingReader();
  const what = detected(reader);

  // Every member that needs the bytes, twice each.
  what.kind;
  what.kind;
  what.is("Jpg");
  what.is("Png");
  what.isAny("Png", "Jpg");
  what.matched;
  what.mime;
  what.extension;
  what.displayName;
  what.rule;

  assert.equal(reader.reads.length, 1, `read ${reader.reads.length} times`);
});

test("the window decides how much is read off a readable object", () => {
  // A narrow window on a path is a small read rather than a large read of which only
  // part is used, so the size asked for is part of the contract.
  const narrow = countingReader();
  detected(narrow, { maxBytesRead: 2_048 }).kind;
  assert.deepEqual(narrow.reads, [2_048]);

  const wide = countingReader();
  detected(wide).kind;
  assert.deepEqual(wide.reads, [neededBytes()]);

  const chained = countingReader();
  detected(chained).within(2_048).kind;
  assert.deepEqual(chained.reads, [2_048], "within() before the read narrows the read");
});

test("within re-asks, and returns this so it chains", () => {
  const what = detected(iso());

  assert.equal(what.kind, "ISO");
  assert.equal(what.within(2_048), what, "within returns this");
  assert.equal(what.kind, null, "ISO's own read size does not fit a 2,048 window");
  assert.equal(what.within(36_870).kind, "ISO", "and back again");

  assert.equal(what.window, 36_870);

  // The window a caller set at construction is the one in force.
  assert.equal(detected(iso(), { maxBytesRead: 2_048 }).kind, null);
  assert.equal(detected(iso(), { maxBytesRead: 2_048 }).window, 2_048);
});

test("widening past the read cannot reach bytes that were never read", () => {
  // The limit of `within`, stated rather than left to be discovered.
  //
  // It needs a source that actually reads, because for bytes already in memory there
  // is no such thing as bytes that were never read: the whole buffer is in hand
  // before `detected` is called, so widening the window reaches all of it. The limit
  // is about the *read*, and only a path or a readable object has one.
  const reader = countingReader(iso());
  const what = detected(reader, { maxBytesRead: 2_048 });

  assert.equal(what.kind, null, "ISO's magic is past a 2,048 window");
  assert.deepEqual(reader.reads, [2_048], "and only 2,048 bytes were read");
  assert.equal(what.within(36_870).kind, null, "widening cannot reach what was not read");
  assert.deepEqual(reader.reads, [2_048], "and it did not read again to try");

  // With a window that reaches the magic, the same source answers.
  const wide = countingReader(iso());
  assert.equal(detected(wide, { maxBytesRead: 36_870 }).kind, "ISO");
  assert.deepEqual(wide.reads, [36_870]);

  // In-memory bytes are the other case, and widening does work there.
  const held = detected(iso(), { maxBytesRead: 2_048 });
  assert.equal(held.kind, null);
  assert.equal(held.within(36_870).kind, "ISO", "the whole buffer was in hand already");
});

test("window defaults to the number that reaches every format", () => {
  // The two numbers both called "the default", and this says which is in play.
  assert.equal(detected(JPEG).window, 36_870);
  assert.equal(detected(JPEG).window, neededBytes());
  assert.notEqual(detected(JPEG).window, 2_048);
});

test("an object is always truthy, so matched is the way to ask", () => {
  // The one thing JavaScript cannot copy from `bool(detected(path))`. A test that only
  // asserted `matched` would pass even if a caller wrote `if (detected(p))`, so this
  // asserts the trap too: the mistake is invisible from the type system.
  const no = detected(NOISE);
  const yes = detected(JPEG);

  assert.equal(no.matched, false);
  assert.equal(yes.matched, true);

  assert.ok(no, "an object is truthy whatever it found");
  assert.ok(!no === false, "so `!detected(p)` is never true either");
});

test("there is no toString, because one would read the file to be logged", () => {
  // A JavaScript `toString` is called by template literals and concatenation without
  // the caller asking, so a repr-like one would turn `console.log(`${d}`)` into a disk
  // read. Printing a member is the explicit way, and this is what makes it the only
  // way.
  const reader = countingReader();
  const what = detected(reader);

  assert.equal(what.toString, Object.prototype.toString);
  assert.equal(`${what}`, "[object Object]");
  assert.deepEqual(reader.reads, [], "and nothing was read to produce it");
});

test("the metadata members answer for a match and are null or a string for none", () => {
  const yes = detected(JPEG);
  assert.equal(yes.mime, "image/jpeg");
  assert.equal(yes.extension, "jpg");
  assert.equal(yes.displayName, "JPEG");
  assert.equal(yes.rule.kind, "Jpg");
  assert.deepEqual([...yes.rule.signatures[0]], [0xff, 0xd8, 0xff, 0xe0]);

  const no = detected(NOISE);
  assert.equal(no.mime, null);
  assert.equal(no.extension, null);
  assert.equal(no.displayName, "no match");
  assert.equal(no.rule, null);
});

test("a path, bytes, an ArrayBuffer and a readable object are all sources", () => {
  withPhoto((dir) => {
    assert.equal(detected(join(dir, "photo.jpg")).kind, "Jpg");
  });
  assert.equal(detected(JPEG).kind, "Jpg");
  assert.equal(detected(JPEG.buffer).kind, "Jpg");
  assert.equal(detected(Buffer.from(JPEG)).kind, "Jpg", "Buffer is a Uint8Array");
  assert.equal(detected({ read: (n) => JPEG.subarray(0, n) }).kind, "Jpg");
});

test("a source that is none of those says which three were accepted", () => {
  // The message is the whole point: an `AttributeError`-shaped failure about a missing
  // `read` would send a caller looking for a typo rather than for the argument.
  for (const bad of [42, null, undefined, {}, true]) {
    assert.throws(
      () => detected(bad).kind,
      {
        name: "TypeError",
        message: /detected\(\) takes a path, bytes, or an object with read\(size\)/,
      },
      `${String(bad)} should be refused with a sentence, not a property error`,
    );
  }
});

test("a negative window is refused, and the message is the one detectBytes gives", () => {
  assert.throws(() => detected(JPEG, { maxBytesRead: -1 }), {
    name: "RangeError",
    message: /maxBytesRead must be a non-negative safe integer/,
  });
  assert.throws(() => detected(JPEG).within(-1), {
    name: "RangeError",
    message: /maxBytesRead must be a non-negative safe integer/,
  });
  assert.throws(() => detected(JPEG, { maxBytesRead: 1.5 }), { name: "RangeError" });
});

test("a missing file throws Node's own error, not null", () => {
  // The same distinction `detectPath` draws: a path that cannot be read is not a file
  // whose format is unknown.
  withPhoto((dir) => {
    assert.throws(() => detected(join(dir, "absent.jpg")).kind, { code: "ENOENT" });
  });
});

test("the internal cache is not reachable from outside", () => {
  // `#private` rather than `_underscore`, because a caller who set the cached bytes
  // would not get a wrong answer *from* a member — they would get a wrong answer
  // *instead of* one, with nothing to say so.
  const what = detected(JPEG);
  assert.deepEqual(Object.keys(what), []);
  assert.equal(what._data, undefined);
  assert.equal(what._kind, undefined);
  assert.equal(what.kind, "Jpg", "and it still works");
});

test("it is the class the barrel exports, so instanceof means something", () => {
  assert.ok(detected(JPEG) instanceof Detected);
  assert.ok(detected(NOISE) instanceof Detected);
});