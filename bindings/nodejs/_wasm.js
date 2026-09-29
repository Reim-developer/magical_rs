// The WebAssembly module, loaded and spoken to.
//
// # Why this file is not generated
//
// There is no wasm-bindgen and no `bg.js`. The module declares no imports, so
// `new WebAssembly.Instance(module, {})` is the whole loader, and the memory ABI
// below is the only thing standing between a caller and the Rust code. That was
// a deliberate choice over wasm-bindgen: it keeps a toolchain out of CI, it ships
// no generated file next to the module, and it leaves the type surface — which
// for this binding is hand-written, generics and all — entirely ours.
//
// # Why everything here is synchronous
//
// `new WebAssembly.Module(bytes)` and `new WebAssembly.Instance(module, {})` are
// synchronous, and `readFileSync` is too, so the module is ready by the time this
// file's top-level code finishes. There is no `await` anywhere in the package and
// no async entry point to forget to await: `detectBytes` returns a `FileKind`,
// not a promise for one. Anyone porting a promise-based wasm library will find
// that difference the first time they forget the `await` and get a kind instead
// of a promise, which is the good direction to be surprised in.
//
// # The two mistakes this file exists to prevent
//
// 1. **Caching the memory view.** `memory.buffer` is replaced whenever memory
//    grows, which detaches every view onto the old one. A `Uint8Array` captured
//    at load time becomes zero-length after the first `memory.grow`. Every read
//    here goes through `view()`, which re-derives it.
//
// 2. **Writing at offset 0.** Address 0 is not free. Linear memory holds the
//    module's static data and its allocator's heap as well as the caller's bytes,
//    and the `table_ptr` export reports 1,114,120 in a module whose memory starts
//    at 1,114,112 bytes — the detection table and the encoded blob sit within a
//    few hundred bytes of the top of the initial memory, and a caller's one-
//    megabyte file written from offset 0 lands in the middle of them. The
//    corruption is silent, because the module keeps answering; the answers are
//    just answers from a shredded table. And it only appears for buffers large
//    enough to reach the data, so a test suite built from short fixtures never
//    sees it and the failure arrives instead in a production directory scan.
//
//    So the module allocates. `input_ptr` and `answer_ptr` say where to write, and
//    both are asked again on every call, because a cached address is a pointer
//    into freed memory the moment a differently-sized file comes through.
//
// The third, subtler one is handled on the Rust side: an empty `Uint8Array` is
// handed over as a zero-length slice, which `from_raw_parts` rejects even for a
// null pointer. `input_ptr` therefore always allocates at least one byte, and
// `borrow` returns `&[]` for a null pointer regardless, so the call is defined
// behaviour rather than undefined behaviour that happens to work.

import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

/**
 * The value the module uses for "nothing matched".
 *
 * **-1, not 4294967295**, which is the part that will bite anyone who assumes the
 * two sides speak the same number. Rust declares the sentinel as `u32::MAX` and
 * that is the right type on its side, but WebAssembly's JavaScript API converts
 * an `i32` *result* with `ToInt32` — that is, as signed. So `u32::MAX` arrives
 * here as `-1`, and comparing against `0xffffffff` silently never matches, which
 * would turn "no match" into "the 4294967295th format" and return `undefined`
 * instead of `null`. Every value this package reads out of the module is small,
 * so nothing else is affected.
 *
 * The Rust side documents the same fact in `NO_MATCH`; the two comments exist
 * because either one read alone is enough to get this wrong.
 */
export const NO_MATCH = -1;

/** A WebAssembly page, in bytes. The growth unit, so not negotiable. */
const PAGE = 65536;

/**
 * The compiled module's exports, or the reason it could not be compiled.
 *
 * Thrown at import time on purpose. A missing or truncated `.wasm` is a broken
 * install, and every later symptom of it would be a wrong answer for a file that
 * is perfectly readable. Failing at the import that needed it names the cause.
 */
function instantiate() {
  const url = new URL("./magical_js.wasm", import.meta.url);
  let bytes;
  try {
    bytes = readFileSync(fileURLToPath(url));
  } catch (cause) {
    throw new Error(
      `magical-js: cannot read ${url.href}.\n` +
        "The compiled module is built by `npm run build` and shipped inside the\n" +
        "package. If you are running from a source checkout, build it first; if you\n" +
        "installed from npm, this is a corrupt install and reinstalling will fix it.",
      { cause },
    );
  }

  try {
    return new WebAssembly.Instance(new WebAssembly.Module(bytes), {});
  } catch (cause) {
    throw new Error(`magical-js: the WebAssembly module did not compile: ${cause.message}`, {
      cause,
    });
  }
}

const instance = instantiate();
const wasm = instance.exports;

/**
 * The module's linear memory, read fresh.
 *
 * Never cache the result. Memory growth replaces `buffer` and detaches every view
 * onto the old one, so a saved `Uint8Array` silently reads as empty — the kind of
 * bug that only shows up once a large buffer has been through, which is to say
 * in production and never in the tests that would have caught it.
 */
function view() {
  return new Uint8Array(wasm.memory.buffer);
}

/**
 * Grow linear memory if `bytes` would not fit.
 *
 * Kept even though the module now allocates its own scratch buffers, because
 * `Vec` growth goes through `memory.grow` too and the caller still has to
 * re-derive its view afterwards. It is not a safety net so much as the statement
 * of the rule: **nothing here may assume memory did not move.**
 */
function growIfNeeded(bytes) {
  const current = wasm.memory.buffer.byteLength;
  if (bytes > current) {
    wasm.memory.grow(Math.ceil((bytes - current) / PAGE));
  }
}

/**
 * Copy `data` into the module's own input buffer, then run `call` with its address.
 *
 * The module allocates the buffer (`input_ptr`) and may reallocate it on the way,
 * so the address is read once and the view is re-derived after. `call` receives
 * the address rather than a view: nothing outside this function should hold one
 * across a call, because building a rule set allocates.
 */
function withInput(data, call) {
  const ptr = wasm.input_ptr(data.length);
  growIfNeeded(ptr + data.length);
  view().set(data, ptr);
  return call(ptr);
}

/**
 * Copy a buffer out of linear memory.
 *
 * A copy rather than a view on purpose: the bytes belong to the module and the
 * next call may overwrite them, so a view would be a value that changes under the
 * caller after it was handed over.
 */
export function readBlobAt(ptr, len) {
  return view().slice(ptr, ptr + len);
}

// ---------------------------------------------------------------------------
// Level 1
// ---------------------------------------------------------------------------

/**
 * The crate's `FileKind` discriminant for these bytes, or [`NO_MATCH`].
 *
 * A signed `i32` on the JavaScript side; see the note on [`NO_MATCH`].
 */
export function whichKindAt(data) {
  return withInput(data, (ptr) => wasm.which_kind_at(ptr, data.length));
}

/**
 * As `whichKindAt`, skipping every rule whose own read size is too large.
 *
 * Also a signed `i32` result, for the same reason.
 */
export function whichKindMaxAt(data, maxBytesRead) {
  return withInput(data, (ptr) => wasm.which_kind_max_at(ptr, data.length, maxBytesRead));
}

/**
 * Whether one format's own rule matches, ignoring the rest of the table.
 *
 * `!== 0` rather than the value itself: Rust's `bool` crosses the boundary as an
 * `i32`, so the module hands back `0` or `1` and returning that would leak a
 * number into a predicate. `0` is the only falsy answer the module can produce
 * here, so the comparison is exact rather than merely convenient.
 */
export function kindMatchesAt(kind, data) {
  return withInput(data, (ptr) => wasm.kind_matches_at(kind, ptr, data.length)) !== 0;
}

// ---------------------------------------------------------------------------
// The table blob
// ---------------------------------------------------------------------------

/** The encoded table and read limits, copied out of the module. */
export function readTableBlob() {
  return readBlobAt(wasm.table_ptr(), wasm.table_len());
}

// ---------------------------------------------------------------------------
// Level 2
// ---------------------------------------------------------------------------

export function rulesNew() {
  return wasm.rules_new();
}

export function rulesAddSignature(handle, data) {
  withInput(data, (ptr) => wasm.rules_add_signature(handle, ptr, data.length));
}

export function rulesAddOffset(handle, offset) {
  wasm.rules_add_offset(handle, offset);
}

export function rulesNextRule(handle) {
  wasm.rules_next_rule(handle);
}

export function rulesFinish(handle) {
  return wasm.rules_finish(handle);
}

export function rulesRelease(handle) {
  wasm.rules_release(handle);
}

export function rulesLen(handle) {
  return wasm.rules_len(handle);
}

/**
 * The index of the first rule that matched, or [`NO_MATCH`].
 *
 * A signed `i32` on the JavaScript side; see the note on [`NO_MATCH`]. A real
 * index is a rule's position, so it is a small positive number and nothing else.
 */
export function rulesMatch(handle, data) {
  return withInput(data, (ptr) => wasm.rules_match(handle, ptr, data.length));
}

/**
 * Every rule that matched, as indices into the rule set.
 *
 * The answer buffer is the module's, sized from `rules_len` — the upper bound on
 * how many rules can match — rather than from a guess, and it is a different
 * buffer from the input. That separation is the point: a rule set with a
 * four-kilobyte signature would otherwise be able to match the bytes that are
 * really its own result list.
 *
 * The count is a signed `i32` on arrival, like every other answer; see
 * [`NO_MATCH`]. The indices themselves are read as `u32`, which is correct
 * because a rule index is never negative. The view is re-derived after the call
 * because `rules_match_all` allocates, and allocation moves memory.
 */
export function rulesMatchAll(handle, data) {
  const input = wasm.input_ptr(data.length);
  growIfNeeded(input + data.length);
  view().set(data, input);
  const answers = wasm.answer_ptr(wasm.rules_len(handle));
  const written = wasm.rules_match_all(handle, input, data.length, answers);
  if (written === NO_MATCH) return [];
  return Array.from(new Uint32Array(wasm.memory.buffer, answers, written));
}
