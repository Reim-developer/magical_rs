// What the TypeScript declarations buy, in a file a compiler reads.
//
// Run it with:
//
//   node --experimental-strip-types examples/06_typed.ts
//
// and type-check it with:
//
//   npm run types
//
// The second command is the one that makes this an example. Running the first shows
// that the code works. Type-checking shows that the compiler *objected to code that
// reads fine*, which is the only property of a type system anyone can observe.
//
// Every `@ts-expect-error` in this file is inside `neverCalled`, because a line the
// compiler rejects is usually a line that also throws — `describe("Jpeg")` is a
// `RangeError` at runtime for exactly the reason it is an error at compile time.
// Keeping the two kinds apart is the difference between an example that runs and one
// that only type-checks.
//
// A `@ts-expect-error` is an assertion that a particular widening has *not* happened:
// if `matchTypes` ever answered `string | null` rather than the union of the kinds
// you declared, these directives would stop being errors, `tsc` would fail on the
// unused directive, and this file would say so.

import {
  allKinds,
  describe,
  detectBytes,
  displayName,
  extension,
  isFileKind,
  matchTypes,
  mime,
  signatureTable,
  type FileKind,
  type MatchRule,
  type Signature,
} from "../index.js";

const PNG = Uint8Array.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
const GIF = Uint8Array.from([0x47, 0x49, 0x46, 0x38, 0x39, 0x61]);

// A format name that arrived from somewhere untyped - a config file, a JSON API
// response - is a `string` and nothing more. Annotated rather than cast, because the
// value genuinely has no better type, and pretending otherwise is what makes a
// type system a decoration.
const fromJson = JSON.parse('"Png"') as string;

// ---------------------------------------------------------------------------
// The first generic: a format name carried into the result.
// ---------------------------------------------------------------------------

// `describe("Png")` is a `Signature<"Png">`, not a `Signature<FileKind>`. That is a
// real narrowing rather than decoration: it makes this line compile, and it gives a
// `switch (rule.kind)` somewhere to narrow to.
const png: "Png" = describe("Png").kind;
console.log(`describe("Png").kind is ${png}, typed as the literal "Png"`);

// The union is the union where that is what is wanted, so a caller who has not
// narrowed and does not want to gets the honest answer rather than an error.
const either: FileKind = describe("Jpg").kind;
console.log(`the union is available where it is wanted: ${either}`);

// ---------------------------------------------------------------------------
// The second generic: `const R`, so the answer is the union of *your* kinds.
// ---------------------------------------------------------------------------

const rules = [
  { kind: "Png", signatures: [PNG], offsets: [0] },
  { kind: "GIF", signatures: [GIF], offsets: [0] },
] as const;

// The answer's type is the union of the kinds that were declared, so the branch you
// write for it is a branch the compiler checked.
const found: "Png" | "GIF" | null = matchTypes(rules, PNG);
console.log(`matchTypes answered ${found}, typed "Png" | "GIF" | null`);

// A name you invent is not confined to the 114. Level 2 rules name their own
// formats, and refusing that would make the feature useless.
const custom = [{ kind: "Bananas", signatures: [PNG], offsets: [0] }] as const;
const bananas: "Bananas" | null = matchTypes(custom, PNG);
console.log(`a name of your own: ${bananas}`);

// ---------------------------------------------------------------------------
// The parts that are not generic, checked so a signature change is caught too.
// ---------------------------------------------------------------------------

const kindOrNull: FileKind | null = detectBytes(PNG);
console.log(`detectBytes answered ${kindOrNull}`);

// `mime` is `string | null` and not `string`, which is the whole reason a caller has
// to write a fallback at all. Were it `string`, the line below would compile and the
// `??` would be dead code TypeScript believed in.
const pngType: string | null = mime("Png");
console.log(`and the fallback is the caller's to write: mime("Png") ?? "application/octet-stream"`);
console.log(`  is ${pngType ?? "application/octet-stream"}`);

// `allKinds()` is typed `FileKind[]` and not `readonly FileKind[]`, so a caller's
// `sort()` on it compiles -- deliberately. It hands out a fresh array per call, so
// sorting one is your own business and cannot reorder the table for the next caller.
// Claiming otherwise would be a type that lies about a convenience it does not
// forbid. What *is* `readonly` is the decoded detection table, and the difference is
// worth being able to state.
const mine = allKinds();
const stillOrdered: readonly FileKind[] = allKinds();
mine.sort();
console.log(`sorting your own copy is allowed: ${mine[0]}, and the next call is still ${stillOrdered[0]}`);


// The rule table is the opposite: `readonly` all the way, because it is shared.
const [pngRule] = signatureTable();
// @ts-expect-error - `Signature.offsets` is `readonly number[]`, and this one really is
// an error. It is the same array for every caller, so a `sort()` would reorder the
// detection table for everybody - which is why `describe()` copies before handing out.
pngRule.offsets.sort();

// The narrowing a guard buys, which is the part that makes iterating untyped input
// bearable without a cast at every step.
if (isFileKind(fromJson)) {
  const name: string = displayName(fromJson);
  const type: string | null = mime(fromJson);
  const suffix: string | null = extension(fromJson);
  console.log(`narrowed in one branch: ${name} / ${type} / .${suffix}`);
}

// The formats where `null` is the honest answer rather than a gap in the table.
console.log(`ELF has no verified media type:   ${mime("ELF")}`);
console.log(`and no conventional extension:    ${extension("ELF")}`);
console.log(`Zlib has neither - which is why neither is a made-up "bin"`);

// ---------------------------------------------------------------------------
// The claims that are about the compiler, in a function nothing calls.
// ---------------------------------------------------------------------------

/** Never called. Every line below is a compile-time claim, not a runtime step. */
function neverCalled(): void {
  // @ts-expect-error - a rule for one format is not a rule for another. This *is* the
  // narrowing: without it `png.kind` would be `FileKind` and this would compile.
  const wrongName: "Jpg" = describe("Png").kind;

  // @ts-expect-error - `string` is wider than the 114-name union, so a name from JSON
  // does not narrow into `FileKind`. The direction a caller actually hits.
  const narrowed: FileKind = fromJson;

  // @ts-expect-error - not a format at all. The point of the union is that a typo is
  // a compile error rather than a `null` at runtime.
  describe("Jpeg");

  // @ts-expect-error - the parameter is constrained to the 114 names.
  displayName("Png2");

  // @ts-expect-error - `Signature`'s default is the union, and it cannot be narrowed
  // after the fact. `signatureTable()` rows are `Signature<FileKind>`.
  const tooTight: Signature<"Png"> = signatureTable()[0];

  // @ts-expect-error - "Jpg" was never declared, so the answer cannot be it. This is
  // what the `const` type parameter exists for: without it the array widens to
  // `{kind: string}[]`, the answer becomes `string | null`, and this line compiles --
  // checking nothing.
  //
  // "Jpg" and not a made-up name, because it is the harder case: it is one of the 114
  // real formats, so the only reason the compiler objects is that these two rules did
  // not mention it.
  const impossible: "Png" | "Jpg" | null = matchTypes(rules, PNG);

  // @ts-expect-error - a rule with no offset can never match, so `MatchRule` forbids
  // one. The annotation is what makes it a check: a bare array literal is inferred
  // as `{kind: string; offsets: never[]}[]` and nothing is ever asked whether it fits.
  // The loader throws a `TypeError` on the same input, so the type and the runtime
  // agree rather than one being the safety net for the other's absence.
  const noOffset: MatchRule[] = [{ kind: "X", signatures: [PNG], offsets: [] }];

  // @ts-expect-error - and a rule with no signature cannot match either.
  const noSignature: MatchRule[] = [{ kind: "X", signatures: [], offsets: [0] }];

  // @ts-expect-error - the option is `maxBytesRead`, not `maxBytes`.
  detectBytes(PNG, { maxBytes: 2048 });

  // @ts-expect-error - a `Uint8Array` or an `ArrayBuffer`, not a string.
  detectBytes("PNG");

  // @ts-expect-error - `string` is wider than the nullable answer. A `mime` that
  // answered `string` would make this compile and make every `??` fallback dead.
  const alwaysThere: string = mime("Png");

  // @ts-expect-error - the same for `extension`, which is `null` for 11 formats.
  const alwaysAnExtension: string = extension("Png");

  // Keep every binding "used" so `noUnusedLocals` would not be the thing that fails.
  void [
    wrongName,
    narrowed,
    tooTight,
    impossible,
    noOffset,
    noSignature,
    alwaysThere,
    alwaysAnExtension,
  ];
}

// Referenced so the function above is not itself dead code in the reader's eyes.
void neverCalled;
