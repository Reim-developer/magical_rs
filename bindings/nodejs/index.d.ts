// The hand-written type surface for magical-js.
//
// Why hand-written rather than generated: the generics below are the reason this
// binding exists in TypeScript, and no generator produces them. A tool that reads
// a `.wasm` and writes `.d.ts` writes `string` where `matchTypes` needs to answer
// `"Png" | "GIF" | null`, because the answer's type comes from *your* array, not
// from the module. So the module's ABI is hand-written in `_wasm.js`, and this
// file is hand-written over the top of it.
//
// `test/types.ts` is the drift test. It type-checks this file and the generics
// against real call sites, including `@ts-expect-error` cases that fail if a type
// silently widens — a declaration that compiles is easy; one that stays narrow
// when it should is the thing worth pinning.

// `FileKind` exists twice on purpose: a type that is the union of 114 names, and a
// value that maps each of them to itself so `FileKind.Png` reads well. One import
// brings both meanings, and one export sends both on — which is also why this
// cannot be `export type { … }` followed by `export { … }`: that is a duplicate
// identifier, because the plain re-export already carries the type with it.
import { FileKind } from "./_kinds.js";

/** Every format the built-in detection table can return. */
export { FileKind };

// ---------------------------------------------------------------------------
// Introspection
// ---------------------------------------------------------------------------

/**
 * One format's detection rule.
 *
 * `K` is the format this describes, and it is narrowed rather than widened: the
 * `kind` field of `describe("Png")` is typed `"Png"`, not the 114-way union. That
 * makes `const k: "Png" = describe("Png").kind` compile and gives a
 * `switch (rule.kind)` somewhere to narrow to.
 */
export interface Signature<K extends FileKind = FileKind> {
  /**
   * The format this rule detects.
   *
   * `"Png"` for `describe("Png")`, `FileKind` for `signatureTable()`'s rows.
   */
  readonly kind: K;

  /**
   * The byte patterns, any one of which is enough — at any one of `offsets`.
   *
   * Empty exactly when the format is decided by a predicate function rather than
   * by bytes, which `usesPredicate` reports. The two are not the same thing:
   * `ScriptExecute` carries `#!` as a prefilter *and* a predicate on what follows
   * it, so it reports both. (The Python binding blanks the signature of every
   * predicate entry; this one does not, because the bytes are there and hiding
   * them would make this a worse description of how the crate actually matches.)
   */
  readonly signatures: readonly Uint8Array[];

  /** Where each signature is looked for. Usually `[0]`. */
  readonly offsets: readonly number[];

  /** How far into the file this format's rule reads. */
  readonly maxBytesRead: number;

  /** Whether a host function decides this format rather than a byte signature. */
  readonly usesPredicate: boolean;
}

/**
 * The detection rule for one format, with its name carried through.
 *
 * ```ts
 * const png = describe("Png");
 * png.kind; //    ^? "Png", not FileKind
 * png.offsets; //  readonly [0]
 * ```
 */
export declare function describe<K extends FileKind>(kind: K): Signature<K>;

/**
 * Every rule in the table, in the crate's own order.
 *
 * A fresh array of fresh objects per call, so sorting or splicing the result does
 * not reorder the table for the next caller.
 */
export declare function signatureTable(): Signature[];

/** Every format name, in the crate's order. A fresh array per call. */
export declare function allKinds(): FileKind[];

/**
 * Whether `value` is one of the format names.
 *
 * A type guard, so an untyped `string` narrows to `FileKind` without a cast.
 */
export declare function isFileKind(value: unknown): value is FileKind;

/**
 * Every read limit the crate reports.
 *
 * Returned as one object rather than five functions because the numbers only make
 * sense together: `isoOffsets` are where the offsets in the `Iso*` family are
 * read, and `isoMaxBytesRead` is how far.
 */
export declare function readLimits(): {
  readonly defaultMaxBytesRead: number;
  readonly bytesRead: number;
  readonly defaultOffset: number;
  readonly isoMaxBytesRead: number;
  readonly tarMaxBytesRead: number;
  readonly isoOffsets: readonly number[];
  readonly tarOffsets: readonly number[];
};

/**
 * The header size every format needs to be distinguishable: 36,870 bytes.
 *
 * Reading less than this means some formats cannot be detected at all, which is
 * what `maxBytesRead` on `detectBytes` is for when you know which formats you
 * care about. This is the crate's `with_bytes_read()`.
 */
export declare function neededBytes(): number;

/** The crate's `DEFAULT_MAX_BYTES_READ`: 2,048 bytes. */
export declare const DEFAULT_MAX_BYTES_READ: number;

// ---------------------------------------------------------------------------
// Level 1 — the built-in table
// ---------------------------------------------------------------------------

export interface DetectOptions {
  /**
   * Narrow the table to rules that read no further than this.
   *
   * A format whose own `maxBytesRead` is larger cannot be returned, *even when
   * the buffer is long enough for it*. That is the crate's
   * `match_with_max_read_rule`, and it is the one knob here that changes answers
   * rather than just their cost, so it is an option and not a second argument.
   */
  readonly maxBytesRead?: number;
}

export interface ReadHeaderOptions {
  /** Bytes to read from the front of the file. Defaults to `neededBytes()`. */
  readonly maxBytes?: number;
}

/**
 * Detect the format of an in-memory buffer, or `null` if it is not recognised.
 *
 * Synchronous: the module is instantiated from bytes already on disk while this
 * package loads, so there is nothing to await.
 */
export declare function detectBytes(
  data: Uint8Array | ArrayBuffer,
  options?: DetectOptions,
): FileKind | null;

/**
 * Detect the format of a file by reading its header, or `null`.
 *
 * `maxBytesRead` does two jobs here that it does not do in `detectBytes`: it also
 * decides how much is read off disk, so a small window is a small read rather
 * than a large read of which only part is used.
 */
export declare function detectPath(
  path: string,
  options?: DetectOptions,
): FileKind | null;

/**
 * Read up to `maxBytes` bytes from the front of a file, detecting nothing.
 *
 * Shorter than `maxBytes` when the file is shorter, which is not an error: a file
 * smaller than the header size is still a file worth classifying. `ENOENT` and
 * the other `errno` failures are left to Node's own error, so `error.code` still
 * works.
 */
export declare function readHeader(
  path: string,
  options?: ReadHeaderOptions,
): Uint8Array;

/**
 * Whether this one format's rule matches, ignoring the rest of the table.
 *
 * Two pairs of formats share a signature *prefix*: `Ktx` matches on `«KTX␣` and
 * `Ktx2` on `«KTX␣20»»»`, and `Qcow` on `QFI` and `Qcow2` on `QFI»`. Nothing is
 * broken by that — the table lists the longer signature first, so a real KTX2
 * file detects as `Ktx2` — but it means the answer to "is this *that* format" and
 * the answer to "what is this" are different questions, and this is the one that
 * asks the first.
 *
 * ```ts
 * detectBytes(ktx2); // "Ktx2"
 * matches("Ktx", ktx2); // true — Ktx's own rule matches too
 * ```
 */
export declare function matches<K extends FileKind>(
  kind: K,
  data: Uint8Array | ArrayBuffer,
): boolean;

// ---------------------------------------------------------------------------
// Level 2 — rules you build at runtime
// ---------------------------------------------------------------------------

/**
 * One custom detection rule: any `signatures` at any `offsets`.
 *
 * Both arrays are non-empty tuples rather than `readonly T[]`, and that is a
 * deliberate claim rather than decoration. A rule with no signature, or no offset
 * to look at one, can never match anything — and a rule that can never match is
 * indistinguishable from a correct one at the call site, because both of them
 * simply do not fire. The types make `offsets: []` a compile error and the loader
 * makes it a `TypeError`; the two agree so a caller who sees one has not been told
 * something the other will contradict.
 *
 * A rule with *several* signatures is the normal case for a format that was
 * renamed, and several offsets is the normal case for a format that is found in
 * more than one place. Both are `[T, ...T[]]`, so both are allowed and neither
 * requires a cast.
 */
export interface MatchRule<K extends string = string> {
  /** Whatever name you want the answer to be. Not restricted to `FileKind`. */
  readonly kind: K;

  /** Any one of these, at any one of `offsets`, is a match. */
  readonly signatures: readonly [Uint8Array, ...Uint8Array[]];

  /** Where to look for them. `0` unless you have a reason. */
  readonly offsets: readonly [number, ...number[]];
}

/**
 * A rule set whose `kind` values are known only at runtime.
 *
 * This is the honest type for a rules array assembled from JSON or computed rather
 * than written out: `matchTypes` accepts one and answers `string | null`. Reach for
 * it deliberately, not by default — it is the type you get *without* the union, and
 * the whole reason this file is hand-written is that the union is available.
 */
export type MatchRules = readonly MatchRule[];

/**
 * The first rule that matches these bytes, or `null`.
 *
 * The answer's type is the union of the `kind` literals *you declared*, which is
 * what the `const` type parameter buys:
 *
 * ```ts
 * const rules = [
 *   { kind: "Png", signatures: [PNG], offsets: [0] },
 *   { kind: "GIF", signatures: [GIF], offsets: [0] },
 * ] as const;
 *
 * const found = matchTypes(rules, bytes);
 * found; //    ^? "Png" | "GIF" | null
 * ```
 *
 * Without `const` the array would widen to `{ kind: string }[]` and the answer
 * would be `string | null`, which checks nothing. With it, `if (found === "Jpg")`
 * is an error the compiler catches rather than a branch that quietly never runs —
 * and it is an error about the *union*, not about the name: `"Jpg"` is one of the
 * 114, which is exactly what makes it a good probe.
 *
 * A rule with no signature, or no offset, is rejected here rather than kept. A
 * rule that can never match is indistinguishable from a correct one at the call
 * site, and "the rule set was silently half-built" is not a bug worth
 * rediscovering in production.
 *
 * The compiled rule set is cached on the array you pass, so the same `rules` in a
 * loop compiles once. `releaseRules` drops it if you are building rule sets
 * rather than holding them.
 *
 * ## The array has to be `as const`, and the type says so
 *
 * ```ts
 * const rules = [{ kind: "Png", signatures: [PNG], offsets: [0] }] as const;
 * matchTypes(rules, bytes); // "Png" | "GIF" | null — for these two rules
 * ```
 *
 * Drop the `as const` and the array widens to `{ kind: string; … }[]`, which
 * `MatchRule` rejects rather than accepts-with-a-worse-answer: the answer would be
 * `string | null`, and `if (found === "Jpg")` would compile while being a branch
 * that can never be taken. Passing the array literal *inline* needs no `as const` —
 * `const R` makes TypeScript infer a literal tuple either way — so the cast is only
 * needed when the rules are held in a variable. For a rules array built at runtime,
 * annotate it as `MatchRules` and expect `string | null`; that is the honest answer
 * for kinds nobody declared.
 */
export declare function matchTypes<const R extends MatchRules>(
  rules: R,
  data: Uint8Array | ArrayBuffer,
): R[number]["kind"] | null;

/**
 * Every rule that matches these bytes, in declaration order.
 *
 * An empty array rather than `null` for no match, so a caller can iterate without
 * a guard. The crate has no "match all" of its own; this is the same rules asked
 * one at a time rather than a second implementation of what a match is.
 */
export declare function matchAllTypes<const R extends MatchRules>(
  rules: R,
  data: Uint8Array | ArrayBuffer,
): R[number]["kind"][];

/**
 * Drop the compiled rule set for a rules array.
 *
 * Releases the handle, not the memory — the compiled copy is leaked once so
 * `MagicCustom` can hold a reference to it, and that is a documented trade in
 * `src/lib.rs`. A caller building rule sets in a loop rather than at startup
 * should call this when done with one. Returns whether there was anything to
 * release, so "already released" and "never compiled" are distinguishable.
 */
export declare function releaseRules(rules: MatchRules): boolean;
