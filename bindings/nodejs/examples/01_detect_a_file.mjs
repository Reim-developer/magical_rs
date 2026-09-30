// Detecting a file on disk, and what its format is called.
//
// Run it with:
//
//   node examples/01_detect_a_file.mjs
//
// The import is a relative path rather than a bare specifier so the examples run
// from a checkout with nothing installed. In a project of your own it is:
//
//   import { detectPath, mime } from "@reim-developer/magical-js";

import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { describe, detectPath, displayName, extension, mime, neededBytes } from "../index.js";

// A file header written out rather than a fixture checked into the repository. A
// checked-in `.png` is a binary whose bytes nobody reviewing a diff can check, and
// it is a file that has to be regenerated whenever the detection table changes.
// Writing the signature longhand says exactly what is being matched.
const PNG = Uint8Array.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);

/** Bytes of an ASCII string, without going through UTF-8 encoding. */
function ascii(text) {
  const out = new Uint8Array(text.length);
  for (let i = 0; i < text.length; i++) out[i] = text.charCodeAt(i) & 0xff;
  return out;
}

// `detectPath` reads the header itself. The default is `neededBytes()` — 36,870,
// the size the largest offset in the table needs — so no format can be missed for
// want of bytes. A smaller read is faster and can be wrong, which is a choice
// rather than a default.
console.log(`a header of ${neededBytes()} bytes is enough for every format this build knows`);

const dir = mkdtempSync(join(tmpdir(), "magical-js-example-"));
try {
  for (const [name, bytes] of [
    ["photo.png", PNG],
    ["holiday.jpg", Uint8Array.from([0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10])],
    ["archive.zip", Uint8Array.from([0x50, 0x4b, 0x03, 0x04, 0x14, 0x00, 0x00, 0x00])],
    ["notes.txt", ascii("Just some text, with no magic bytes anywhere in it.\n")],
    ["stubborn.webp", PNG],
  ]) {
    writeFileSync(join(dir, name), bytes);
  }

  console.log();
  for (const name of ["photo.png", "holiday.jpg", "archive.zip", "notes.txt", "stubborn.webp"]) {
    const kind = detectPath(join(dir, name));

    // `null` is the honest answer for a file no rule matched, and it is a different
    // answer from a failure to read the file: that one throws, with Node's own
    // `code` on it, so a missing file and an unrecognised one cannot be confused.
    if (kind === null) {
      console.log(`  ${name.padEnd(16)} no signature matched`);
      continue;
    }

    // The three questions detection cannot answer, because it reads bytes. A name
    // is what a person recognises; a MIME type is what a server serves.
    //
    // `mime` answers `null` for the formats with no registered type, never a
    // guess: a wrong MIME type is served to a browser and is indistinguishable
    // from a right one. The fallback belongs here, where the choice is visible.
    const type = mime(kind) ?? "application/octet-stream";
    console.log(
      `  ${name.padEnd(16)} ${kind.padEnd(9)} ${displayName(kind).padEnd(16)} ` +
        `${type.padEnd(26)} .${extension(kind) ?? "(none)"}`,
    );
  }

  // `stubborn.webp` is worth a second look. It is a PNG named `.webp`, and it is
  // reported as `Png`, because detection reads the bytes and never the file name.
  // A `.jpg` holding a PNG is the same case, and it is the one where a caller who
  // trusted the extension was already wrong.
  console.log();
  console.log("the extension is advisory and never read by detection:");
  console.log(`  stubborn.webp is ${describe("WEBP").signatures.length === 0 ? "decided by a function" : "matched by bytes"},`);
  console.log(`  but its bytes are a PNG, so the answer is ${detectPath(join(dir, "stubborn.webp"))}`);
} finally {
  rmSync(dir, { recursive: true, force: true });
}
