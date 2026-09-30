// Format metadata: what a format is called, what it is served as, what it is
// written as.
//
// # Why this is separate from the detection table
//
// `_signatures.js` decodes the table the crate encodes into the wasm module, and
// everything it answers is a fact about *matching*: which bytes, at which offset,
// how far it reads. None of the three questions this file answers can be answered
// from that module, and keeping them apart is what says so.
//
// The answers are also not facts about the world, which is the more interesting
// reason. `50 4B 03 04` says a file is a zip container; it does not say whether
// that container is a `.jar` or an `.apk`, because the difference is in the bytes
// *inside* it, which is a longer answer than a magic number. So the values here
// are looked up rather than derived, and they come from `formats.json` -- the same
// table that generates the Rust `FileKind::mime` and the Python `FileKind.mime`,
// so a MIME type cannot be one thing in Rust and another in JavaScript.
//
// # Why `null` rather than a guess
//
// A format with no registered MIME type answers `null`. It does not answer
// `application/octet-stream`, because that is a decision -- "this is opaque
// binary, do not render it" -- and baking it in here would make a caller who
// wants a `Content-Type` header get one without ever having chosen it. A wrong
// MIME type is served to a browser; a missing one is a visible `null` the caller
// has to answer for. Same reasoning for `extension`: there is no `bin`.
//
// The one thing that *is* derived is the error for an unknown format name, which
// goes through the same `indexOfKind` every other entry point uses so the message
// and the type guard cannot drift apart.

import { FILE_KIND_DISPLAY_NAME, FILE_KIND_EXTENSION, FILE_KIND_MIME } from "./_kinds.js";
import { indexOfKind } from "./_signatures.js";

/**
 * Validate a format name, or throw an error saying what was asked for.
 *
 * A `Map` would answer `undefined` for a name that is not a format, and a caller
 * writing `mime(await guessFormat(file))` would get `undefined` for two entirely
 * different reasons: "this file is not a format we know" and "this file is a
 * format with no registered MIME type". Those are worth telling apart, and the
 * first is worth a `RangeError` rather than a silent `undefined` that becomes a
 * `Content-Type` of the string "undefined".
 *
 * @param {unknown} kind
 * @returns {import("./_kinds.js").FileKind}
 */
function checked(kind) {
  // The lookup is by string and the result is the same string, so a value that is
  // not a name can never be indexed. `indexOfKind` is what raises, and its message
  // already names the value and points at `allKinds()`.
  if (typeof kind !== "string") {
    throw new TypeError(
      `@reim-developer/magical-js: a format name must be a string, got ${
        kind === null ? "null" : typeof kind
      }. \`allKinds()\` lists the 114 of them.`,
    );
  }
  indexOfKind(kind);
  return /** @type {import("./_kinds.js").FileKind} */ (kind);
}

/**
 * The name this format is written with in documentation and in a file listing.
 *
 * `"PNG"`, not `"Png"`. The two answer different questions and are not
 * interchangeable: the format's *identifier* is `FileKind.Png`, and this is the
 * human-facing spelling of the same thing. It is what the Rust `display_name` and
 * the Python `description` answer, so a table rendered from any of the three
 * bindings reads the same.
 *
 * @param {import("./_kinds.js").FileKind} kind
 * @returns {string}
 */
export function displayName(kind) {
  return FILE_KIND_DISPLAY_NAME[checked(kind)];
}

/**
 * The registered MIME type, or `null` when there is none.
 *
 * `null` means "not registered or not verified", never "unknown" -- the project
 * does not invent a type, because an invented one is served to a browser and is
 * indistinguishable from a real one. A caller that needs a value for a header
 * should fall back itself:
 *
 * ```js
 * const type = mime(kind) ?? "application/octet-stream";
 * ```
 *
 * @param {import("./_kinds.js").FileKind} kind
 * @returns {string | null}
 */
export function mime(kind) {
  return FILE_KIND_MIME[checked(kind)];
}

/**
 * The conventional file extension, or `null` when there is none.
 *
 * Without a leading dot, and advisory: detection never reads a file name, so a
 * `.jpg` holding a PNG is reported as a PNG. This is for choosing what to
 * *write*, which is the one question a magic number cannot answer.
 *
 * It is also not the name this package saves a format under. That is the format
 * name itself, `"Png"`, which is stable across releases; an extension is a
 * convention that a project is free to stop following.
 *
 * @param {import("./_kinds.js").FileKind} kind
 * @returns {string | null}
 */
export function extension(kind) {
  return FILE_KIND_EXTENSION[checked(kind)];
}
