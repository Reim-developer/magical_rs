// `detected(path).is("Png")` — detection with the data first.
//
// `detectBytes` puts the function first and the bytes second, which is the right
// shape for a question asked once and thrown away:
//
//   import { detectBytes } from "@reim-developer/magical-js";
//   detectBytes(bytes); // "Png"
//
// It is the wrong shape for the case this module exists for: a scan that asks about
// every file in a directory, where the answer is usually *no*. There the readable
// form is the one Rust's `magical_fluent` feature gives, with the subject written
// first and the question trailing it:
//
//   for (const path of paths) {
//     if (detected(path).isAny("Png", "GIF", "Jpg")) { ... }
//   }
//
// # Why a wrapper class rather than a method on the bytes
//
// Because JavaScript will not let it, for the same reason Python will not: a
// `Uint8Array` is a built-in, so the spelling that would read best here —
// `bytes.detect()` — is the one this language cannot offer. `Detected` is the wrapper
// standing in for it.
//
// # Why `is` and not `matches`
//
// Because JavaScript has no keyword in the way. `is` is Rust's name for this question
// and is the shortest thing that says it, and `magical_py` can only spell it
// `matches` because `is` is a Python keyword.
//
// Deliberately *not* `matches`, though, because this package already exports a free
// function of that name taking the arguments the other way round: `matches(kind,
// data)`. A method reading `detected(p).matches(k)` beside a function reading
// `matches(k, d)` would be one name for two things, which is the kind of ambiguity
// that costs an afternoon. `is` and `isAny` cannot be confused with either.
//
// # What this is not
//
// It adds no matching. Every member below is a named call to something the binding
// already had — `detectBytes`, `matches`, `describe`, `readHeader`, and the metadata
// tables — and the answers are the ones those give. This is a spelling, not a
// detection level.
//
// # What it costs
//
// The bytes are read once, on first use, and kept. A path is opened exactly once
// however many members are read off the result, and a readable object is read exactly
// once, which matters because a pipe has its bytes only once.
//
// That is also the limit of `within`. It re-classifies what was read; it cannot reach
// bytes that were never read. Narrowing the window always works, because the bytes
// are a superset of a narrower one. Widening works too, but only as far as the read
// went — which is why `window` is a member and not something a caller has to
// remember.
//
// # What JavaScript cannot copy from `magical_py`
//
// `bool(detected(path))` works in Python because `__bool__` returns the answer. An
// object is always truthy in JavaScript, so `if (detected(path))` is **always true**
// and cannot be made otherwise: there is no hook a language offers for it. That is
// what `matched` is for, and it is a member rather than something to reach for.
//
// For the same reason there is no `toString` and no `Symbol.toPrimitive`. Python's
// `__repr__` reads the file, which is right — a repr that printed the same thing for
// every input would be the one thing a repr must not do. A JavaScript `toString` is
// not explicit: template literals and string concatenation call it without the caller
// asking, so `console.log(`${d}`)` would read a file as a side effect of logging it.
// Print a member instead.

import { bytesRead } from "./_signatures.js";
import { displayName, extension, mime } from "./_meta.js";
import { checkSize, describe, detectBytes, matches, readHeader } from "./_levels.js";

const NAME = "@reim-developer/magical-js";

/**
 * A file's format, and the questions about it, with the file written first.
 *
 * Not a value object: it holds the bytes, so it holds memory, and one of these per
 * file across a large scan adds up. It is meant to answer a few questions about one
 * file and then be dropped.
 *
 * ```js
 * import { detected } from "@reim-developer/magical-js";
 *
 * for (const path of directory) {
 *   if (detected(path).isAny("Png", "GIF", "Jpg")) { ... }
 * }
 * ```
 *
 * Nothing is read by the constructor. The read happens on the first member that needs
 * an answer, so building one is free and a caller who only passes it on never touches
 * the disk.
 *
 * `detected(source, options)` is the way to build one; the constructor is public
 * because a class has to have one, and it behaves identically.
 */
export class Detected {
  // `#private` rather than `_underscore`, because the read-once cache is this class's
  // central promise and a naming convention does not enforce it. A caller who reached
  // in and set the cached bytes would not get a wrong answer *from* a member — they
  // would get a wrong answer *instead of* one, with nothing to say so. The fields are
  // invisible from outside, which is the point.
  #source;
  #window;
  #data = null;
  // The window `#kind` was computed in, so a window change asks again rather than
  // answering from the wrong one.
  #kind = null;
  #kindWindow = null;

  /**
   * @param {string | Uint8Array | ArrayBuffer | { read(size: number): Uint8Array }} source
   *   A path, bytes already in memory, or anything with a synchronous `read(size)`.
   * @param {{ maxBytesRead?: number }} [options]
   */
  constructor(source, options = {}) {
    this.#source = source;
    this.#window =
      options.maxBytesRead === undefined || options.maxBytesRead === null
        ? bytesRead()
        : checkSize(options.maxBytesRead, "maxBytesRead");
  }

  /**
   * The format, or `null` if the table does not recognise these bytes.
   *
   * The same answer as `detectBytes`: first match in detection order over the whole
   * table, filtered to rules whose own read size fits the window. Cached, so asking
   * twice reads nothing twice.
   *
   * @returns {import("./_kinds.js").FileKind | null}
   */
  get kind() {
    if (this.#kindWindow !== this.#window) {
      this.#kind = detectBytes(this.#bytes(), { maxBytesRead: this.#window });
      this.#kindWindow = this.#window;
    }
    return this.#kind;
  }

  /**
   * Whether these bytes are *this one format*, ignoring every other rule.
   *
   * Not the same question as `.kind === kind`, and the difference is worth naming.
   * This asks whether that one format's own rule matches, so a file that is both `Ktx`
   * and — by its first bytes — an earlier entry in the table is the earlier entry as
   * far as `kind` is concerned, and is still "yes, it is Ktx" as far as this is
   * concerned.
   *
   * `Ktx` is the case that matters: its magic is a byte-for-byte prefix of `Ktx2`'s,
   * so a KTX2 file is never *reported* as KTX, and this is the only way to ask.
   *
   * @param {import("./_kinds.js").FileKind} kind
   * @returns {boolean}
   */
  is(kind) {
    return matches(kind, this.#bytes());
  }

  /**
   * Whether these bytes are any of *kinds*, one lookup per format.
   *
   * Accepts the arguments of this form or a single iterable, so that both
   * `isAny("Png", "GIF")` and `isAny(["Png", "GIF"])` are correct. That is not
   * cleverness: a caller holding an array should not have to spread it, and a caller
   * holding two literals should not have to bracket them.
   *
   * The formats are tried in the order given, so this is a `boolean` rather than which
   * one matched. An empty list is `false`, since a file cannot be any of nothing.
   *
   * @param {...(import("./_kinds.js").FileKind | Iterable<import("./_kinds.js").FileKind>)} kinds
   * @returns {boolean}
   */
  isAny(...kinds) {
    return flatten(kinds).some((kind) => this.is(kind));
  }

  /**
   * Classify the same bytes in a different window, and return this.
   *
   * Named after the Rust `detect_within`. A window is a claim about how much of the
   * file was read, so lowering it makes every format whose own magic sits past that
   * point unable to match — a statement about the window and not about the file.
   *
   * This does not read again, because it does not have to: the bytes already in hand
   * are a superset of a narrower window, and `detectBytes` filters by the window
   * whatever it is handed. Widening the window beyond the original read cannot recover
   * bytes that were never read, and `window` reports what it actually is so a caller
   * can see that rather than infer it.
   *
   * @param {number} maxBytesRead
   * @returns {this} this, so it can be chained onto the constructor.
   */
  within(maxBytesRead) {
    this.#window = checkSize(maxBytesRead, "maxBytesRead");
    // Dropped rather than trusted: the cache is keyed on the window.
    this.#kindWindow = null;
    return this;
  }

  /**
   * The window in force, in bytes.
   *
   * Defaults to `neededBytes()` — 36,870 — and is *not* `DEFAULT_MAX_BYTES_READ`,
   * which is the crate's 2,048. Both are worth having and they are different numbers,
   * so this says which is in play rather than leaving a caller to remember.
   *
   * @returns {number}
   */
  get window() {
    return this.#window;
  }

  /**
   * Whether anything matched at all.
   *
   * The stand-in for Python's `bool(detected(path))`, which works there because
   * `__bool__` can return the answer. A JavaScript object is always truthy, so
   * `if (detected(path))` is always true whatever this says — use this.
   *
   * @returns {boolean}
   */
  get matched() {
    return this.kind !== null;
  }

  /**
   * What `kind` compares, or `null` when nothing matched.
   *
   * The same value as `describe(kind)`, reached without naming the kind first. It is
   * `null` here rather than throwing as that call does for a name that is not a
   * format, because "nothing matched" is a normal answer for this object and not an
   * exceptional one.
   *
   * @returns {import("./index.js").Signature | null}
   */
  get rule() {
    const kind = this.kind;
    return kind === null ? null : describe(kind);
  }

  /**
   * The media type of `kind`, or `null` if there is none.
   *
   * @returns {string | null}
   */
  get mime() {
    const kind = this.kind;
    return kind === null ? null : mime(kind);
  }

  /**
   * The conventional extension of `kind`, or `null`.
   *
   * @returns {string | null}
   */
  get extension() {
    const kind = this.kind;
    return kind === null ? null : extension(kind);
  }

  /**
   * The human-readable name of `kind`, or `"no match"`.
   *
   * Every format has a display name, so this is a string in both cases — which is
   * what a caller printing a result wants, and why it is not `null` the way `mime` is.
   *
   * @returns {string}
   */
  get displayName() {
    const kind = this.kind;
    return kind === null ? "no match" : displayName(kind);
  }

  /**
   * The bytes of the source, read once and kept.
   *
   * Private because it is an implementation detail of the cache, and reachable from
   * `is` because asking whether one format matches must not be answered by whatever
   * the table happened to reach first.
   *
   * @returns {Uint8Array}
   */
  #bytes() {
    if (this.#data === null) {
      this.#data = readSource(this.#source, this.#window);
    }
    return this.#data;
  }
}

/**
 * Return a `Detected` over *source*.
 *
 * Nothing is read here: the read happens on the first member that needs an answer, so
 * building one is free and a caller who only wants to pass it on never touches the
 * disk.
 *
 * @param {string | Uint8Array | ArrayBuffer | { read(size: number): Uint8Array }} source
 *   A path, bytes already in memory, or anything with a synchronous `read(size)`.
 * @param {{ maxBytesRead?: number }} [options]
 * @returns {Detected}
 */
export function detected(source, options = {}) {
  return new Detected(source, options);
}

/**
 * Accept both `isAny(a, b)` and `isAny([a, b])`.
 *
 * One argument that is iterable is the caller's collection; anything else is the
 * arguments as given. A `FileKind` is a string and a string is iterable as characters,
 * so the string case is checked first — otherwise `isAny("Png")` would test every
 * letter of `"Png"`, which is a bug that passes a casual test.
 *
 * An array out, so a caller passing a generator has it consumed once here rather than
 * once per format.
 *
 * @param {unknown[]} kinds
 * @returns {string[]}
 */
function flatten(kinds) {
  if (kinds.length !== 1) return kinds;

  const only = kinds[0];
  if (typeof only === "string") return kinds;
  if (only !== null && only !== undefined && typeof only[Symbol.iterator] === "function") {
    return [...only];
  }
  return kinds;
}

/**
 * Return *source* as bytes, reading a path or a readable object if it is one.
 *
 * The shapes are told apart here rather than in each member, because they are the same
 * decision several times over and getting one wrong shows up only for that kind of
 * input.
 *
 * @param {string | Uint8Array | ArrayBuffer | { read(size: number): Uint8Array }} source
 * @param {number} window
 * @returns {Uint8Array}
 */
function readSource(source, window) {
  if (source instanceof Uint8Array) return source;
  if (source instanceof ArrayBuffer) return new Uint8Array(source);
  if (typeof source === "string") return readHeader(source, { maxBytes: window });

  // `read` is looked up rather than called straight, so that the error below is a
  // sentence about the three shapes that are accepted instead of a `TypeError` about
  // a missing property.
  const reader = source === null || source === undefined ? undefined : source.read;
  if (typeof reader !== "function") {
    throw new TypeError(
      `${NAME}: detected() takes a path, bytes, or an object with read(size), not ${
        source === null ? "null" : typeof source
      }`,
    );
  }
  return reader.call(source, window);
}