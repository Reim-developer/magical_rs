// Detection against the built-in table, using fixtures written out as bytes.

import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import * as api from "../index.js";
import { FIXTURES, NOT_A_FILE, ascii } from "./fixtures.js";

test("every fixture detects as the format it is a fixture for", () => {
  for (const [kind, data] of FIXTURES) {
    assert.equal(api.detectBytes(data), kind, `${kind} fixture did not detect as ${kind}`);
    assert.equal(api.matches(kind, data), true, `matches(${kind}, ...) said no`);
  }
});

test("there are enough fixtures to be worth having", () => {
  // A fixture list that shrinks to two entries still passes the test above. This
  // is the assertion that the list has not quietly been gutted.
  assert.ok(FIXTURES.size >= 40, `only ${FIXTURES.size} fixtures`);
});

test("bytes that are not a file detect as nothing", () => {
  assert.equal(api.detectBytes(NOT_A_FILE), null);
  assert.equal(api.detectBytes(new Uint8Array()), null);
  assert.equal(api.detectBytes(ascii("PNG")), null, "the name is not the magic");
});

test("a truncated signature does not detect", () => {
  const png = FIXTURES.get("Png");
  assert.equal(api.detectBytes(png.subarray(0, 4)), null);
  assert.equal(api.detectBytes(png.subarray(0, 7)), null);
  assert.equal(api.detectBytes(png.subarray(0, png.length - 1)), null);
});

test("the signature has to be at the offset the rule wants", () => {
  // `ustar` is at 257 for a tarball. One byte earlier and it is not a tarball.
  const tar = FIXTURES.get("Tar");
  const early = new Uint8Array(tar.length);
  early.set(tar.subarray(1), 0);
  assert.equal(api.detectBytes(early), null);
  assert.equal(api.detectBytes(tar), "Tar");
});

test("maxBytesRead narrows the table rather than merely reading less", () => {
  const png = FIXTURES.get("Png");
  assert.equal(api.detectBytes(png), "Png");
  // PNG's own rule reads 2048 bytes, so an 8-byte window cannot return it — even
  // though the buffer is long enough to hold the signature. This is the crate's
  // `match_with_max_read_rule` and the one option here that changes answers.
  assert.equal(api.detectBytes(png, { maxBytesRead: 8 }), null);
  assert.equal(api.detectBytes(png, { maxBytesRead: 2048 }), "Png");

  // Tar's rule also reads 2048, even though its signature is 5 bytes at offset
  // 257. The crate sizes a rule by the *buffer it needs*, not by the signature, so
  // a 262-byte window finds nothing. This is the assertion that would fail if
  // `maxBytesRead` were quietly taken from the signature length instead.
  const tar = FIXTURES.get("Tar");
  assert.equal(api.describe("Tar").maxBytesRead, 2048);
  assert.equal(api.detectBytes(tar, { maxBytesRead: 262 }), null);
  assert.equal(api.detectBytes(tar, { maxBytesRead: 2048 }), "Tar");

  // `readLimits().tarMaxBytesRead` is 262 and is a *different* number: how far the
  // crate reads when walking a tar archive. It has nothing to do with recognising
  // one, and conflating the two is a mistake this test now refuses to make.
  assert.equal(api.readLimits().tarMaxBytesRead, 262);
  assert.notEqual(api.readLimits().tarMaxBytesRead, api.describe("Tar").maxBytesRead);
});

test("a large buffer does not lose the view of memory", () => {
  // `memory.buffer` is replaced whenever memory grows, which detaches every view
  // onto the old one. A loader that cached its view breaks here and nowhere else:
  // a caller passing a 4 MB buffer is the only path that grows memory, so this is
  // the test that would catch it.
  const big = new Uint8Array(4 * 1024 * 1024);
  big.set(FIXTURES.get("Png"));
  assert.equal(api.detectBytes(big), "Png");
  // And still works afterwards, from the grown memory.
  assert.equal(api.detectBytes(FIXTURES.get("ELF")), "ELF");
  assert.equal(api.detectBytes(NOT_A_FILE), null);
});

test("a signature past the end of a short buffer does not over-read", () => {
  // Tar's signature is at 257, so a 10-byte buffer must decline rather than read
  // whatever happens to be in memory there.
  assert.equal(api.detectBytes(new Uint8Array(10)), null);
  assert.equal(api.detectBytes(FIXTURES.get("ISO").subarray(0, 100)), null);
});

test("a Buffer works as well as a Uint8Array, because on Node it is one", () => {
  assert.equal(api.detectBytes(Buffer.from(FIXTURES.get("Png"))), "Png");
});

test("an ArrayBuffer is wrapped rather than refused", () => {
  const png = FIXTURES.get("Png");
  assert.equal(api.detectBytes(png.buffer.slice(0, png.length)), "Png");
});

test("a shared ArrayBuffer view is respected, not the whole buffer", () => {
  // `.buffer` on a subarray is the whole underlying buffer, so this is the one
  // case where the caller has to hand over a view rather than its buffer.
  const padded = new Uint8Array(64);
  padded.set(FIXTURES.get("Png"), 32);
  assert.equal(api.detectBytes(padded), null, "the signature is at 32, not 0");
  assert.equal(api.detectBytes(padded.subarray(32, 40)), "Png");
});

test("matches() sees the overlapping formats that detectBytes resolves", () => {
  // Ktx's signature is a strict prefix of Ktx2's, and Qcow's of Qcow2's. The
  // table lists the longer one first so a real file detects correctly, and
  // `matches` is the call that asks about the shorter one anyway.
  const ktx2 = FIXTURES.get("Ktx2");
  assert.equal(api.detectBytes(ktx2), "Ktx2");
  assert.equal(api.matches("Ktx2", ktx2), true);
  assert.equal(api.matches("Ktx", ktx2), true, "Ktx's own rule matches a KTX2 file");

  const qcow2 = FIXTURES.get("Qcow2");
  assert.equal(api.detectBytes(qcow2), "Qcow2");
  assert.equal(api.matches("Qcow", qcow2), true);
  // But not the other way round: Ktx2 does not match a bare KTX 11 header.
  assert.equal(api.matches("Ktx2", FIXTURES.get("Ktx")), false);
});

test("a non-Uint8Array argument is refused with a message that says what is wanted", () => {
  assert.throws(() => api.detectBytes("PNG"), {
    name: "TypeError",
    message: /needs a Uint8Array or an ArrayBuffer, got string/,
  });
  assert.throws(() => api.detectBytes(null), { name: "TypeError" });
  assert.throws(() => api.matches("Png", 42), { name: "TypeError" });
});

test("a nonsense maxBytesRead is refused rather than floored", () => {
  for (const value of [-1, 1.5, Number.NaN, "2048", Infinity]) {
    assert.throws(() => api.detectBytes(FIXTURES.get("Png"), { maxBytesRead: value }), {
      name: "RangeError",
      message: /must be a non-negative safe integer/,
    });
  }
});

test("readLimits reports the crate's own numbers", () => {
  const limits = api.readLimits();
  // These are the crate's, so they are stated rather than derived: a change here
  // is a change to how much of a file has to be read, which is a real cost to
  // every caller, and it should fail a test rather than pass quietly.
  assert.deepEqual(limits, {
    defaultMaxBytesRead: 2048,
    bytesRead: 36870,
    defaultOffset: 0,
    isoMaxBytesRead: 36870,
    tarMaxBytesRead: 262,
    isoOffsets: [32769, 34817, 36865],
    tarOffsets: [257],
  });
  assert.equal(api.neededBytes(), 36870);
  assert.equal(api.DEFAULT_MAX_BYTES_READ, 2048);
});

test("readLimits hands out copies of its arrays", () => {
  const limits = api.readLimits();
  limits.isoOffsets.push(0);
  assert.deepEqual(api.readLimits().isoOffsets, [32769, 34817, 36865]);
});

test("readHeader and detectPath read a real file", () => {
  const dir = mkdtempSync(join(tmpdir(), "magical-js-"));
  try {
    const path = join(dir, "sample.png");
    writeFileSync(path, FIXTURES.get("Png"));

    assert.equal(api.detectPath(path), "Png");

    const header = api.readHeader(path);
    assert.ok(header instanceof Uint8Array);
    assert.deepEqual([...header], [...FIXTURES.get("Png")]);

    assert.equal(api.readHeader(path, { maxBytes: 4 }).length, 4);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("a file smaller than the header size is not an error", () => {
  const dir = mkdtempSync(join(tmpdir(), "magical-js-"));
  try {
    const path = join(dir, "tiny");
    writeFileSync(path, ascii("hi"));
    // Shorter than asked for, which is normal, and detectable as nothing.
    assert.equal(api.readHeader(path).length, 2);
    assert.equal(api.detectPath(path), null);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("a missing file throws Node's own error, code and all", () => {
  // Re-wrapping would replace a `code` a caller can branch on with a message that
  // says less, so the error is left alone.
  assert.throws(() => api.detectPath(join(tmpdir(), "magical-js-not-here")), { code: "ENOENT" });
  assert.throws(() => api.readHeader(42), {
    name: "TypeError",
    message: /needs a path string, got number/,
  });
});

test("a directory is refused as what it is, on every platform", () => {
  const dir = mkdtempSync(join(tmpdir(), "magical-js-"));
  try {
    // `EISDIR` rather than whatever Node's own `openSync` does, because that
    // differs: it throws on Linux and succeeds on Windows, where a directory
    // reports a size of zero. Without this check `detectPath(dir)` would answer
    // `null` on Windows — "not a recognised format" — for a path that cannot be a
    // file at all, and the same call would throw elsewhere.
    for (const call of [api.readHeader, api.detectPath]) {
      assert.throws(() => call(dir), { code: "EISDIR" }, `${call.name} accepted a directory`);
    }
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
