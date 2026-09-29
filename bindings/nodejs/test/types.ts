// The generic drift test, checked by `tsc --noEmit` (`npm run types`).
//
// `index.d.ts` is hand-written, so nothing makes its generics true. A declaration
// that compiles is the easy half; the half worth testing is that the types stay
// *narrow* when they should, because widening from `"Png" | "GIF" | null` to
// `string | null` compiles perfectly and quietly checks nothing at the call site.
//
// So this file is mostly `@ts-expect-error`. Each one is an assertion that a
// particular widening has *not* happened: if the type widens, the annotated error
// stops being an error and `tsc` fails on the unused directive, which is exactly
// the regression worth catching.

import {
  allKinds,
  detectBytes,
  describe,
  isFileKind,
  matchAllTypes,
  matchTypes,
  matches,
  neededBytes,
  readLimits,
  signatureTable,
  type FileKind,
  type MatchRule,
  type MatchRules,
  type Signature,
} from "../index.js";

// ---------------------------------------------------------------------------
// The first generic: `K extends FileKind` carried into `Signature<K>`.
// ---------------------------------------------------------------------------

const png = describe("Png");
const pngKind: "Png" = png.kind;
void pngKind;

// @ts-expect-error - a rule for one format is not a rule for another. This is the
// narrowing itself: `Signature<"Png">` rather than `Signature<FileKind>`.
const wrongName: "Jpg" = png.kind;
void wrongName;

// The union is the *narrower* type, so a `string` does not flow into it. This is
// the direction a caller hits when a format name arrives from JSON, and it is the
// reason the union is 114 literals rather than `string`.
declare const fromJson: string;
// @ts-expect-error - `string` is wider than the union, so it does not narrow into it.
const narrowedFromString: FileKind = fromJson;
void narrowedFromString;

// The union is still the union when that is what is wanted.
const either: FileKind = describe("Jpg").kind;
void either;

// @ts-expect-error - not a format at all. The whole point of the union is that a
// typo is an error rather than a `null` at runtime.
describe("Jpeg");

// @ts-expect-error - the parameter is constrained to the 114 names.
describe("Png2");

function narrowsFromTheRule(rule: Signature) {
  if (rule.kind === "Woff2") {
    // @ts-expect-error - inside the branch the kind is `"Woff2"` and nothing else.
    const wrong: "Png" = rule.kind;
    void wrong;
  }
}
void narrowsFromTheRule;

// @ts-expect-error - `Signature<K>`'s default is the union, not `string`.
const tooLoose: Signature<"Png" | "Jpg"> = signatureTable()[0];
void tooLoose;

// ---------------------------------------------------------------------------
// The second generic: `const R extends MatchRules`, so the answer is the union
// of the kinds that were declared.
// ---------------------------------------------------------------------------

const PNG_BYTES = new Uint8Array([0x89, 0x50, 0x4e, 0x47]);
const GIF_BYTES = new Uint8Array([0x47, 0x49, 0x46]);

const rules = [
  { kind: "Png", signatures: [PNG_BYTES], offsets: [0] },
  { kind: "GIF", signatures: [GIF_BYTES], offsets: [0] },
] as const;

const found: "Png" | "GIF" | null = matchTypes(rules, PNG_BYTES);
void found;

// @ts-expect-error - "Jpg" was never declared, so the answer cannot be it. This
// is the assertion the `const` type parameter exists for: without it the rules
// array widens to `{ kind: string }[]`, the answer becomes `string | null`, and
// this line compiles — checking nothing.
//
// `"Jpg"` and not a made-up name, because that is the harder case: it is one of
// the 114 real formats, so the only reason the compiler objects is that these two
// rules did not mention it. A name the crate has never heard of would be rejected
// for a reason that has nothing to do with the union.
const impossible: "Png" | "Jpg" | null = matchTypes(rules, PNG_BYTES);
void impossible;

const all: ("Png" | "GIF")[] = matchAllTypes(rules, PNG_BYTES);
void all;

// @ts-expect-error - the same for `matchAllTypes`, which is a separate signature.
const allImpossible: "Jpg"[] = matchAllTypes(rules, PNG_BYTES);
void allImpossible;

// A caller-supplied name is not confined to the 114, and that is deliberate:
// level 2 rules name their own formats.
const custom = [{ kind: "Bananas", signatures: [PNG_BYTES], offsets: [0] }] as const;
const bananas: "Bananas" | null = matchTypes(custom, PNG_BYTES);
void bananas;

// @ts-expect-error - a rule with no offset can never match, so the type forbids
// one. The loader throws a `TypeError` on the same input; the two agree rather
// than one being the safety net for the other's absence.
const incomplete: MatchRule[] = [{ kind: "X", signatures: [PNG_BYTES], offsets: [] }];
void incomplete;

// @ts-expect-error - and a rule with no signature cannot match either.
const noSignature: MatchRule[] = [{ kind: "X", signatures: [], offsets: [0] }];
void noSignature;

// Several signatures and several offsets are the normal case, not a special one,
// so they need no cast either.
const wellFormed: MatchRule[] = [
  { kind: "X", signatures: [PNG_BYTES, GIF_BYTES], offsets: [0, 4, 8] },
];
void wellFormed;

// The no-`as const` path, which is the one most callers write. `const R` is what
// makes it work: the type parameter asks TypeScript to infer the array as a
// readonly tuple of literal-typed entries rather than widening it to
// `{ kind: string }[]`. If `const` were dropped from the signature this line would
// report `string | null` for the answer, and `const direct` below would not
// type-check at all.
const direct: "Png" | "Bananas" | null = matchTypes(
  [
    { kind: "Png", signatures: [PNG_BYTES], offsets: [0] },
    { kind: "Bananas", signatures: [GIF_BYTES], offsets: [0] },
  ],
  PNG_BYTES,
);
void direct;

// @ts-expect-error - and the same literal argument, read as the union it produced.
const notBananas: "Png" | null = matchTypes(
  [
    { kind: "Png", signatures: [PNG_BYTES], offsets: [0] },
    { kind: "Bananas", signatures: [GIF_BYTES], offsets: [0] },
  ],
  PNG_BYTES,
);
void notBananas;

// Rules held in a variable *without* `as const` are refused rather than accepted
// with a worse answer. Pinned because it looks like an inconvenience and is not:
// without `as const` the array widens to `{ kind: string }[]`, so `matchTypes`
// would return `string | null` and none of the narrowing in this file would
// apply. The one-token fix is on the next line.
const widenedRules = [{ kind: "Png", signatures: [PNG_BYTES], offsets: [0] }];
// @ts-expect-error - `widenedRules` is `{ kind: string; … }[]`, not a literal tuple.
const fromWidened: "Png" | null = matchTypes(widenedRules, PNG_BYTES);
void fromWidened;

const pinnedRules = [{ kind: "Png", signatures: [PNG_BYTES], offsets: [0] }] as const;
const fromPinned: "Png" | null = matchTypes(pinnedRules, PNG_BYTES);
void fromPinned;

// The other one-token fix, for a rules array built at runtime from a wider array:
// annotate it as `MatchRules` and the answer is `string | null` — honestly typed,
// and the loader still works.
declare const builtAtRuntime: MatchRules;
const fromAnnotated: string | null = matchTypes(builtAtRuntime, PNG_BYTES);
void fromAnnotated;

// ---------------------------------------------------------------------------
// The parts that are not generic, checked so a signature change is caught too.
// ---------------------------------------------------------------------------

const kindOrNull: FileKind | null = detectBytes(PNG_BYTES);
void kindOrNull;

// @ts-expect-error - the option is `maxBytesRead`, not `maxBytes`.
detectBytes(PNG_BYTES, { maxBytes: 2048 });

// @ts-expect-error - `ArrayBuffer` and `Uint8Array`, not a string.
detectBytes("PNG");

const answer: boolean = matches("Png", PNG_BYTES);
void answer;

// @ts-expect-error - `matches` is narrowed to a format name too.
matches("Png2", PNG_BYTES);

const guard: boolean = isFileKind("Png");
void guard;

// The guard takes `unknown` on purpose, so *any* value may be offered and the
// narrowing happens in the branch. The assertion is that the branch narrows:
// without the guard, `fromJson` stays a `string` and the assignment below fails.
if (isFileKind(fromJson)) {
  const narrowedInBranch: FileKind = fromJson;
  void narrowedInBranch;
}

const needed: number = neededBytes();
void needed;

const bytesNeeded: number = readLimits().bytesRead;
void bytesNeeded;

const everyKind: FileKind[] = allKinds();
void everyKind;

const oneRule: Signature<FileKind> = signatureTable()[0];
void oneRule;

// `readonly` is honoured, so a caller cannot sort the table in place.
const table = signatureTable();
const fixed: readonly number[] = table[0].offsets;
// @ts-expect-error - `offsets` is readonly, not a mutable array.
table[0].offsets.push(1);
void fixed;

// @ts-expect-error - `Signature` has no field this package invented.
void table[0].mimeType;
