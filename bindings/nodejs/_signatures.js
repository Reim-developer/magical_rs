// The detection table, decoded once at import, and the introspection API over it.
//
// # Why the whole table crosses the boundary as one blob
//
// The Rust side encodes `SIGNATURE_KIND` and the crate's read limits into a
// single buffer and hands over a pointer and a length. The obvious alternative —
// reading `Magic` structs straight out of linear memory — would mean depending on
// `#[repr(Rust)]` field order, which is not a stable layout and would break
// silently on a compiler update instead of loudly. A hand-written encoding costs
// one decoder and buys a layout that only changes when this repository changes it.
//
// # Why it is decoded eagerly
//
// At import, not on first use. A blob that does not match what this decoder
// expects is a mismatched `.wasm`, and the useful moment to say so is while the
// stack still says which import failed. Decoding lazily would spread the same
// failure across whichever call happened to need the data, which for the answer
// buffer is the one already inside a `try` somewhere in a caller's code.
//
// # The generic, and what it buys
//
// `describe<K extends FileKind>(kind: K): Signature<K>` infers `K` from the
// argument and carries it into the result, so `describe("Png").kind` has the type
// `"Png"` rather than `FileKind`. That is a real narrowing, not decoration: it
// makes `const kind: "Png" = describe("Png").kind` compile, and it makes
// `if (rule.kind === "Png")` a check the compiler can use to narrow the rest of
// the object. See `index.d.ts` for the second generic, over custom rule sets.

import { FILE_KIND_INDICES, FILE_KIND_NAMES } from "./_kinds.js";
import { readTableBlob } from "./_wasm.js";

/** Slot index into the blob's constant block. Mirrors `mod constant` in lib.rs. */
const CONSTANT = {
  DEFAULT_MAX_BYTES_READ: 0,
  BYTES_READ: 1,
  DEFAULT_OFFSET: 2,
  ISO_MAX_BYTES_READ: 3,
  TAR_MAX_BYTES_READ: 4,
};

const BLOB_MAGIC = "MTB1";

/**
 * A cursor over the blob.
 *
 * Every read is bounds-checked and throws with the byte offset it failed at.
 * A decoder that returns `NaN` or a short read instead would produce a rule
 * table that is wrong in a way nobody can trace back to the file that decoded it.
 */
class Reader {
  constructor(bytes) {
    this.bytes = bytes;
    this.at = 0;
  }

  u8() {
    if (this.at + 1 > this.bytes.length) {
      throw new RangeError(`@reim-developer/magical-js: table blob ended at byte ${this.at} reading a u8`);
    }
    return this.bytes[this.at++];
  }

  u32() {
    if (this.at + 4 > this.bytes.length) {
      throw new RangeError(`@reim-developer/magical-js: table blob ended at byte ${this.at} reading a u32`);
    }
    // A fresh view rather than a DataView: one per field would be an allocation
    // per field, and this runs 114 times at import. Little endian is what the
    // encoder writes on every platform wasm runs on.
    const value = new DataView(this.bytes.buffer, this.bytes.byteOffset + this.at, 4).getUint32(0, true);
    this.at += 4;
    return value;
  }

  slice(length) {
    if (this.at + length > this.bytes.length) {
      throw new RangeError(
        `@reim-developer/magical-js: table blob ended at byte ${this.at} reading ${length} bytes`,
      );
    }
    const view = this.bytes.subarray(this.at, this.at + length);
    // Copied, not a view: the blob's buffer is the module's, and the next call
    // into wasm may grow memory and detach it.
    this.at += length;
    return new Uint8Array(view);
  }

  offsets() {
    const count = this.u32();
    const out = new Array(count);
    for (let i = 0; i < count; i++) out[i] = this.u32();
    return out;
  }
}

function decode() {
  const bytes = readTableBlob();
  const r = new Reader(bytes);

  const magic = String.fromCharCode(...bytes.subarray(0, 4));
  if (magic !== BLOB_MAGIC) {
    throw new Error(
      `@reim-developer/magical-js: the table blob starts with ${JSON.stringify(magic)}, expected ` +
        `${JSON.stringify(BLOB_MAGIC)}. The compiled module and this decoder are from ` +
        "different builds.",
    );
  }
  r.at = 4;

  const constantCount = r.u32();
  const constants = new Array(constantCount);
  for (let i = 0; i < constantCount; i++) constants[i] = r.u32();
  const constant = (name) => {
    const index = CONSTANT[name];
    if (index === undefined) throw new Error(`@reim-developer/magical-js: no constant slot named ${name}`);
    if (index >= constantCount) {
      throw new Error(
        `@reim-developer/magical-js: constant slot ${name} is ${index}, but the module carries only ` +
          `${constantCount} constants`,
      );
    }
    return constants[index];
  };

  const limits = {
    defaultMaxBytesRead: constant("DEFAULT_MAX_BYTES_READ"),
    bytesRead: constant("BYTES_READ"),
    defaultOffset: constant("DEFAULT_OFFSET"),
    isoMaxBytesRead: constant("ISO_MAX_BYTES_READ"),
    tarMaxBytesRead: constant("TAR_MAX_BYTES_READ"),
    isoOffsets: r.offsets(),
    tarOffsets: r.offsets(),
  };
  const count = r.u32();
  // Two orderings, kept separately, because they are two orderings.
  //
  // `byDiscriminant` is indexed by the discriminant each entry *carries*, which is
  // what makes `describe()` correct: 71 of the 114 entries are not in declaration
  // order, measured rather than assumed, because `ScriptExecute` (18) sits at
  // position 17 and `RAR` (17) at position 18. Indexing by position would attribute
  // every rule to the wrong format.
  //
  // `byPosition` is indexed by where the entry sits in the blob, and the blob is
  // written by iterating `SIGNATURE_KIND`, so this one *is* the detection order.
  //
  // Keeping only the first and then rebuilding a list by walking it is what left
  // `signatureTable()`'s rows in enum order while its docstring claimed the
  // crate's order. The two differ by up to 55 places, and for the two formats whose
  // magic shadows another's — `Ktx2` and `Qcow2` — walking that list answers `Ktx`
  // and `Qcow` where `detectBytes` answers `Ktx2` and `Qcow2`. See issue #24.
  const byDiscriminant = new Array(count);
  const byPosition = new Array(count);
  for (let position = 0; position < count; position++) {
    const kind = r.u32();
    const flags = r.u32();
    const maxBytesRead = r.u32();
    const offsets = r.offsets();
    const signatureCount = r.u32();
    const signatures = new Array(signatureCount);
    for (let s = 0; s < signatureCount; s++) {
      const length = r.u32();
      signatures[s] = r.slice(length);
    }
    if (kind >= count) {
      throw new Error(
        `@reim-developer/magical-js: table entry ${position} claims discriminant ${kind}, but there are ` +
          `only ${count} formats. The module and _kinds.js are from different builds.`,
      );
    }
    if (byDiscriminant[kind] !== undefined) {
      throw new Error(
        `@reim-developer/magical-js: two table entries both claim discriminant ${kind}. Every format ` +
          "must appear exactly once, or one of them is invisible.",
      );
    }
    byPosition[position] = kind;
    byDiscriminant[kind] = {
      usesPredicate: (flags & 1) !== 0,
      maxBytesRead,
      offsets,
      signatures,
    };
  }

  const missing = byDiscriminant.indexOf(undefined);
  if (missing !== -1) {
    throw new Error(
      `@reim-developer/magical-js: no table entry claims discriminant ${missing} (${
        FILE_KIND_NAMES[missing]
      }). Every format must appear, or \`describe\` would answer for some formats and ` +
        "not others with nothing to tell the two apart.",
    );
  }

  if (r.at !== bytes.length) {
    // The Rust test `the_blob_is_exactly_as_long_as_its_contents_claim` walks the
    // same bytes from the other end and asserts the same thing, so a failure here
    // means one of the two decoders drifted from the encoder.
    throw new Error(
      `@reim-developer/magical-js: the table blob has ${bytes.length - r.at} trailing bytes the decoder ` +
        "did not account for. The module and this decoder are from different builds.",
    );
  }

  return { limits, byDiscriminant, byPosition };
}

const { limits, byDiscriminant, byPosition } = decode();

/** The default header size, matching the crate's `DEFAULT_MAX_BYTES_READ`. */
export const DEFAULT_MAX_BYTES_READ = limits.defaultMaxBytesRead;

/** The header size needed to classify any supported format. */
export function bytesRead() {
  return limits.bytesRead;
}

/**
 * Every read limit the crate reports, in one object.
 *
 * A fresh object, and fresh offset arrays, per call. The numbers only make sense
 * together — `isoOffsets` says where the `Iso*` family is read and `isoMaxBytesRead`
 * says how far — so they are handed over as one value rather than five.
 *
 * `tarMaxBytesRead` (262) and `isoMaxBytesRead` (36,870) are **not** the read sizes
 * of the detection rules for those formats. `describe("Tar").maxBytesRead` is 2048
 * and `describe("ISO").maxBytesRead` is 36,870; the tar figure is how far into a
 * tarball the crate reads when it walks the archive, which has nothing to do with
 * recognising one. Measured, and the two being different is the kind of thing that
 * reads as a contradiction until it is written down.
 */
export function readLimits() {
  return {
    defaultMaxBytesRead: limits.defaultMaxBytesRead,
    bytesRead: limits.bytesRead,
    defaultOffset: limits.defaultOffset,
    isoMaxBytesRead: limits.isoMaxBytesRead,
    tarMaxBytesRead: limits.tarMaxBytesRead,
    isoOffsets: [...limits.isoOffsets],
    tarOffsets: [...limits.tarOffsets],
  };
}

/**
 * The index of a format name, or a thrown error naming what was asked for.
 *
 * Validation lives here rather than in Rust so the message can say
 * `"Foo"` is not a file kind and point at `allKinds()`. A discriminant that
 * simply matched nothing would be indistinguishable from a file that is not the
 * format you thought it was, which is the worst possible failure for a function
 * whose whole job is to answer.
 */
export function indexOfKind(kind) {
  const index = FILE_KIND_INDICES.get(kind);
  if (index === undefined) {
    throw new RangeError(
      `@reim-developer/magical-js: ${JSON.stringify(kind)} is not a file kind. There are ` +
        `${FILE_KIND_INDICES.size} of them; \`allKinds()\` lists them.`,
    );
  }
  return index;
}

/**
 * The public shape of one rule, built from a decoded entry.
 *
 * Shared by the two lookups below so that `describe` and `signatureTable` cannot
 * drift apart in what a row contains — they differ only in which row they reach,
 * never in how it is shaped.
 */
function row(discriminant) {
  const entry = byDiscriminant[discriminant];
  const name = FILE_KIND_NAMES[discriminant];
  if (entry === undefined || name === undefined) {
    throw new RangeError(
      `@reim-developer/magical-js: the table reports a rule for discriminant ${discriminant}, which is not ` +
        `a file kind. The module and _kinds.js are from different builds; run \`npm run gen\`.`,
    );
  }
  return {
    kind: name,
    // Every array is copied, because the decoded table is shared by every caller and
    // a `sort()` on a returned `offsets` would reorder it for everybody.
    signatures: entry.signatures.map((signature) => new Uint8Array(signature)),
    offsets: [...entry.offsets],
    maxBytesRead: entry.maxBytesRead,
    usesPredicate: entry.usesPredicate,
  };
}

/**
 * One format's detection rule, looked up by discriminant.
 *
 * This is what `describe()` wants: a format named, and the rule that names it. It
 * says nothing about position, and must not — 71 of the 114 entries sit at a
 * position that is not their discriminant.
 *
 * `signatures` is empty exactly when the format is decided by a function rather
 * than by bytes, which is what `usesPredicate` says. The two are not the same
 * thing: `ScriptExecute` carries `#!` as a prefilter *and* a predicate on what
 * follows it, so it reports both. That is worth saying because the Python binding
 * blanks the signature of every predicate entry, and this one does not — the bytes
 * are there, and hiding them would make the table a worse description of how the
 * crate actually matches.
 */
export function signatureForIndex(index) {
  return row(index);
}

/**
 * One format's detection rule, looked up by position in the detection order.
 *
 * This is what `signatureTable()` wants. It is a different lookup from
 * `signatureForIndex` and the difference is the whole point: `byPosition` is the
 * order `detectBytes` walks, and `byDiscriminant` is the enum's.
 */
export function signatureAtPosition(position) {
  const discriminant = byPosition[position];
  if (discriminant === undefined) {
    throw new RangeError(
      `@reim-developer/magical-js: no rule at detection position ${position}. There are ` +
        `${byPosition.length}; \`signatureTable()\` returns all of them.`,
    );
  }
  return row(discriminant);
}

/** How many entries the table has. Not part of the public API. */
export function tableLength() {
  return byPosition.length;
}
