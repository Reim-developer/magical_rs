// Levels 1 and 2: detection against the built-in table, and against rules you
// build at runtime.
//
// # What is not here, and why
//
// Levels 3 and 4 of the crate decide a format by calling back into the host
// language. This binding does not offer them. The module has no imports at all —
// `WebAssembly.Module.imports` returns `[]` — and calling JavaScript from Rust
// requires an import, so adding levels 3 and 4 would trade away the one property
// that makes this module loadable from any byte array with no glue. Level 5 is raw
// pointers into host memory, which has no meaning in WebAssembly, where caller and
// callee share one address space.
//
// Level 2 *is* here, and it is the crate's level 2 rather than a reimplementation:
// `MagicCustom` wants `&'static` slices, so a completed rule set is leaked once
// into a handle table. See `lib.rs` for why that is the right trade, and
// `releaseRules` for what a caller who builds them in a loop should do about it.

import { closeSync, fstatSync, openSync, readSync } from "node:fs";

import { FILE_KIND_INDICES, FILE_KIND_NAMES } from "./_kinds.js";
import {
  DEFAULT_MAX_BYTES_READ,
  bytesRead,
  indexOfKind,
  signatureAtPosition,
  signatureForIndex,
  tableLength,
} from "./_signatures.js";
import {
  NO_MATCH,
  kindMatchesAt,
  rulesAddOffset,
  rulesAddSignature,
  rulesFinish,
  rulesMatch,
  rulesMatchAll,
  rulesNew,
  rulesNextRule,
  rulesRelease,
  whichKindAt,
  whichKindMaxAt,
} from "./_wasm.js";

/** Every format the built-in table can return, in the crate's own order. */
export function allKinds() {
  return [...FILE_KIND_NAMES];
}

/**
 * Whether `value` is one of the 114 format names.
 *
 * A type guard rather than a plain predicate, so a `string` from somewhere
 * untyped narrows to `FileKind` without a cast.
 *
 * @param {unknown} value
 */
export function isFileKind(value) {
  return typeof value === "string" && FILE_KIND_INDICES.has(/** @type {never} */ (value));
}

/**
 * The rule for one format.
 *
 * Direct rather than a search: the discriminant already is the table index, which
 * is the whole reason `_kinds.js` is generated from the crate's declaration order.
 */
export function describe(kind) {
  return signatureForIndex(indexOfKind(kind));
}

/**
 * Every rule in the table, in detection order.
 *
 * That is the order `detectBytes` walks, so walking this array and stopping at the
 * first rule that matches reproduces `detectBytes` exactly — which is what makes it
 * worth the order being stated rather than assumed. It was the enum order instead
 * until issue #24, and the two differ for 71 of the 114 entries, by up to 55 places.
 * For `Ktx2` and `Qcow2` — the two whose magic shadows a shorter neighbour's — the
 * walk answered `Ktx` and `Qcow`.
 *
 * Not the same as `allKinds()`, which is the `FileKind` declaration order. Both are
 * 114 long and both are stable, so treating one as the other compiles, runs, and
 * answers a different question.
 *
 * A fresh array of fresh objects every call. `BUFFER` uses this to build its own
 * values, so handing out the internal ones would let a caller's `sort()` reorder
 * the table for everybody.
 */
export function signatureTable() {
  const out = new Array(tableLength());
  for (let i = 0; i < tableLength(); i++) out[i] = signatureAtPosition(i);
  return out;
}

/**
 * The header size a file needs for every format to be distinguishable.
 *
 * `bytesRead()` under the name the rest of this package uses. Both are exported
 * because the crate calls it `with_bytes_read()` and the Python binding calls it
 * `bytes_read()`, and neither spelling is obviously the right one to a reader who
 * has only seen the other.
 */
export function neededBytes() {
  return bytesRead();
}

/**
 * Validate an optional byte count.
 *
 * Integer-only, because a fractional read size would be silently floored somewhere
 * deeper and a negative one would compare oddly against every rule's real size.
 *
 * @param {unknown} value
 * @param {string} name
 */
function checkSize(value, name) {
  if (value === undefined || value === null) return undefined;
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0) {
    throw new RangeError(`@reim-developer/magical-js: ${name} must be a non-negative safe integer, got ${value}`);
  }
  return value;
}

/**
 * Narrow a caller's buffer to a `Uint8Array` without copying it.
 *
 * `Buffer` is a `Uint8Array`, so it passes straight through, which matters on Node
 * where `readFileSync` has already produced one. An `ArrayBuffer` is wrapped. A
 * string is rejected rather than encoded: guessing an encoding would be guessing
 * at the bytes being detected.
 *
 * @param {unknown} data
 * @param {string} where
 */
function toBytes(data, where) {
  if (data instanceof Uint8Array) return data;
  if (data instanceof ArrayBuffer) return new Uint8Array(data);
  throw new TypeError(
    `@reim-developer/magical-js: ${where} needs a Uint8Array or an ArrayBuffer, got ` +
      `${data === null ? "null" : typeof data}`,
  );
}

/**
 * Detect the format of an in-memory buffer.
 *
 * `maxBytesRead` narrows the table to the rules whose own read size fits inside
 * that window, which is the crate's `match_with_max_read_rule`. A window smaller
 * than a format needs means that format cannot be returned *even when the buffer
 * is long enough to hold it*, so the default is the size that suits every rule.
 *
 * @param {Uint8Array | ArrayBuffer} data
 * @param {{ maxBytesRead?: number }} [options]
 * @returns {import("./_kinds.js").FileKind | null}
 */
export function detectBytes(data, options = {}) {
  const bytes = toBytes(data, "detectBytes");
  const maxBytesRead = checkSize(options.maxBytesRead, "maxBytesRead");

  const index =
    maxBytesRead === undefined ? whichKindAt(bytes) : whichKindMaxAt(bytes, maxBytesRead);
  return index === NO_MATCH ? null : FILE_KIND_NAMES[index];
}

/**
 * Read a file's header and detect its format.
 *
 * `maxBytesRead` does two jobs here that it does not do in `detectBytes`: it also
 * decides how much is read off disk, so a small window is a small `readSync`
 * rather than a large read of which only part is used.
 *
 * @param {string} path
 * @param {{ maxBytesRead?: number }} [options]
 * @returns {import("./_kinds.js").FileKind | null}
 */
export function detectPath(path, options = {}) {
  const maxBytesRead = checkSize(options.maxBytesRead, "maxBytesRead");
  const header = readHeader(path, { maxBytes: maxBytesRead ?? bytesRead() });
  return detectBytes(header, maxBytesRead === undefined ? {} : { maxBytesRead });
}

/**
 * Read up to `maxBytes` bytes from the front of a file, detecting nothing.
 *
 * The read happens here, in JavaScript, rather than in the module: the module has
 * no `std::fs`, because a WebAssembly module with filesystem access is a module
 * that cannot load in a browser. `detectBytes` is the whole of the detection and
 * this is the convenience around it.
 *
 * A short read is normal and is not an error: a file smaller than the header size
 * is still a file worth classifying, and the returned array is shorter than asked
 * for. Only a real failure throws.
 *
 * @param {string} path
 * @param {{ maxBytes?: number }} [options]
 * @returns {Uint8Array}
 */
export function readHeader(path, options = {}) {
  if (typeof path !== "string") {
    throw new TypeError(`@reim-developer/magical-js: readHeader needs a path string, got ${typeof path}`);
  }
  const maxBytes = checkSize(options.maxBytes, "maxBytes") ?? bytesRead();

  // Left to throw as-is from here down. `ENOENT` and `EACCES` each carry a `code`
  // a caller can branch on, and rewrapping them would replace that with a message
  // that says less. The path is already in the error.
  const fd = openSync(path, "r");
  try {
    const stats = fstatSync(fd);
    // One failure is manufactured rather than inherited, because Node's own
    // behaviour here is platform-dependent: `openSync` on a directory throws
    // `EISDIR` on Linux and succeeds on Windows, where the directory reports a
    // size of 0. Left alone, the same call would return an empty array on one
    // platform and throw on the other, and a caller doing `detectPath(dir)` would
    // get `null` — "not a recognised format" — for a path that cannot be a file at
    // all. The `code` is the one Node uses, so a caller that already handles
    // directories keeps working.
    if (stats.isDirectory()) {
      const error = new Error(
        `@reim-developer/magical-js: ${path} is a directory, not a file.`,
      );
      error.code = "EISDIR";
      throw error;
    }
    const wanted = Math.min(maxBytes, stats.size);
    const out = new Uint8Array(wanted);
    let filled = 0;
    while (filled < wanted) {
      const read = readSync(fd, out, filled, wanted - filled, filled);
      if (read === 0) break;
      filled += read;
    }
    return filled === wanted ? out : out.subarray(0, filled);
  } finally {
    closeSync(fd);
  }
}

/**
 * Whether one format's own rule matches these bytes.
 *
 * Ignores the rest of the table, which is the reason it exists. Two pairs of
 * formats share a signature *prefix* — measured, not assumed: `Ktx` matches on
 * `«KTX␣` (5 bytes) and `Ktx2` on `«KTX␣20»»»` (12), and likewise `Qcow` on `QFI`
 * and `Qcow2` on `QFI»`. So `matches("Ktx", data)` answers whether `Ktx` matched
 * even where `detectBytes` answers `"Ktx2"` — a question about one format rather
 * than about the file.
 *
 * The overlap costs nothing to the ordinary path: `SIGNATURE_KIND` lists the
 * longer signature first, so a real KTX2 file still detects as `Ktx2`. This is
 * the call for when you want to ask the other question anyway.
 *
 * @param {import("./_kinds.js").FileKind} kind
 * @param {Uint8Array | ArrayBuffer} data
 */
export function matches(kind, data) {
  return kindMatchesAt(indexOfKind(kind), toBytes(data, "matches"));
}

// ---------------------------------------------------------------------------
// Level 2: rules built at runtime
// ---------------------------------------------------------------------------

/**
 * A rule set compiled once per array and cached on that array.
 *
 * A `WeakMap` keyed by the array the caller passed, so the same rules used in a
 * loop compile once, and a rules array the caller dropped becomes collectable.
 *
 * What is *not* collected is the Rust side's copy: `rules_finish` leaks the rule
 * set so `MagicCustom` can hold a `&'static` reference, and releasing the handle
 * does not give that memory back. That is the documented trade in `lib.rs`, and it
 * is why `releaseRules` exists rather than being a nicety.
 *
 * @type {WeakMap<readonly object[], number>}
 */
const compiled = new WeakMap();

/**
 * Validate one rule and hand back its parts.
 *
 * Checked here as well as in Rust. Rust rejects an incomplete rule set because a
 * rule that can never match is indistinguishable from a correct one at the call
 * site. JavaScript checks first because it can say *which* rule and *which* kind,
 * and "rules[3], whose kind is MyFormat, has no offset" is a sentence the Rust
 * side cannot construct — it only ever sees indices by then.
 *
 * @param {unknown} rule
 * @param {number} index
 */
function checkRule(rule, index) {
  if (rule === null || typeof rule !== "object") {
    throw new TypeError(
      `@reim-developer/magical-js: rules[${index}] must be an object, got ${rule === null ? "null" : typeof rule}`,
    );
  }
  const { kind, signatures, offsets } = /** @type {Record<string, unknown>} */ (rule);
  if (typeof kind !== "string" || kind === "") {
    throw new TypeError(`@reim-developer/magical-js: rules[${index}].kind must be a non-empty string`);
  }
  if (!Array.isArray(signatures) || signatures.length === 0) {
    throw new TypeError(
      `@reim-developer/magical-js: rules[${index}] (${kind}) needs at least one signature; a rule with ` +
        "none can never match",
    );
  }
  if (!Array.isArray(offsets) || offsets.length === 0) {
    throw new TypeError(
      `@reim-developer/magical-js: rules[${index}] (${kind}) needs at least one offset; a rule with none ` +
        "can never match",
    );
  }
  offsets.forEach((offset, position) => {
    checkSize(offset, `rules[${index}].offsets[${position}]`);
  });
  return {
    kind,
    signatures: signatures.map((signature, position) =>
      toBytes(signature, `rules[${index}].signatures[${position}]`),
    ),
    offsets,
  };
}

/**
 * Compile a rule array into a wasm handle, memoised on the array.
 *
 * @param {readonly object[]} rules
 */
function handleFor(rules) {
  if (!Array.isArray(rules)) {
    throw new TypeError(`@reim-developer/magical-js: rules must be an array, got ${typeof rules}`);
  }
  if (rules.length === 0) {
    throw new RangeError(
      "@reim-developer/magical-js: rules is empty. A rule set with no rules answers null for everything, " +
        "which a caller cannot tell apart from a format that was not recognised.",
    );
  }

  const cached = compiled.get(rules);
  if (cached !== undefined) return cached;

  const checked = rules.map(checkRule);
  const handle = rulesNew();
  for (const rule of checked) {
    for (const signature of rule.signatures) rulesAddSignature(handle, signature);
    for (const offset of rule.offsets) rulesAddOffset(handle, offset);
    rulesNextRule(handle);
  }

  if (!rulesFinish(handle)) {
    // Rust is the backstop for the checks above, so reaching this means one of the
    // two validators is wrong, not the caller.
    rulesRelease(handle);
    throw new Error(
      "@reim-developer/magical-js: the compiled rule set was rejected. That is a bug in magical-js " +
        "rather than in the rules you passed, because the JavaScript checks accepted them.",
    );
  }

  compiled.set(rules, handle);
  return handle;
}

/**
 * Drop the compiled rule set for a rules array, if it has one.
 *
 * Releases the handle, not the memory — see the note on `compiled`. Returns whether
 * there was anything to release, so a caller can tell "already released" from
 * "never compiled".
 *
 * @param {readonly object[]} rules
 */
export function releaseRules(rules) {
  const handle = compiled.get(/** @type {readonly object[]} */ (rules));
  if (handle === undefined) return false;
  compiled.delete(/** @type {readonly object[]} */ (rules));
  rulesRelease(handle);
  return true;
}

/**
 * The first rule that matches, or `null`.
 *
 * The result type is the union of the `kind` literals you declared, which is what
 * the `const` type parameter buys: with a literal array the answer is
 * `"Png" | "Gif" | null` rather than `string | null`, so the compiler checks the
 * branch you wrote for it.
 *
 * @template {readonly { readonly kind: string }[]} R
 * @param {R} rules
 * @param {Uint8Array | ArrayBuffer} data
 * @returns {R[number]["kind"] | null}
 */
export function matchTypes(rules, data) {
  const bytes = toBytes(data, "matchTypes");
  const index = rulesMatch(handleFor(rules), bytes);
  if (index === NO_MATCH) return null;
  return /** @type {any} */ (rules[index].kind);
}

/**
 * Every rule that matches, in declaration order.
 *
 * The crate has no "match all" of its own, so this is the same rules asked one at a
 * time rather than a second implementation of what a match is. Returns a fresh
 * array, and an empty one rather than `null`, so a caller can iterate without a
 * guard.
 *
 * @template {readonly { readonly kind: string }[]} R
 * @param {R} rules
 * @param {Uint8Array | ArrayBuffer} data
 * @returns {R[number]["kind"][]}
 */
export function matchAllTypes(rules, data) {
  const bytes = toBytes(data, "matchAllTypes");
  const indices = rulesMatchAll(handleFor(rules), bytes);
  return indices.map((index) => /** @type {any} */ (rules[index].kind));
}

export { DEFAULT_MAX_BYTES_READ };
