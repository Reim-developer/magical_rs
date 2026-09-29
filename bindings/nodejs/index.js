// @reim-developer/magical-js - JavaScript and TypeScript bindings for `magical_rs`.
//
// This file is the package's whole public surface, and it is a barrel on purpose.
// Every name below is re-exported from exactly one internal module, so the shape
// of the package is decided here and nowhere else: `_kinds.js` is generated,
// `_wasm.js` is the memory ABI, `_signatures.js` decodes the table, and
// `_levels.js` is the API. None of those four is imported by name from outside —
// not by a test, not by the README's examples — so a change inside one of them
// is invisible from out here, which is what makes the barrel worth having.
//
// The types are hand-written in `index.d.ts`, and the generics there are the
// point of this binding: `describe("Png").kind` is typed `"Png"`, and
// `matchTypes` on a literal rule array answers with the union of the kinds you
// declared. Neither is expressible in JavaScript's own JSDoc annotations, and
// neither is worth giving up for a `.wasm` that loads with no glue.
//
// Everything is synchronous. See `_wasm.js` for why that is not an
// approximation.

export { FileKind } from "./_kinds.js";

export {
  DEFAULT_MAX_BYTES_READ,
  allKinds,
  detectBytes,
  detectPath,
  describe,
  isFileKind,
  matchAllTypes,
  matchTypes,
  matches,
  neededBytes,
  readHeader,
  releaseRules,
  signatureTable,
} from "./_levels.js";

export { readLimits } from "./_signatures.js";
