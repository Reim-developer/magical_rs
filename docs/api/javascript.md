# JavaScript API reference

`@reim-developer/magical-js`, for JavaScript and TypeScript. For what the calls
*mean* rather than what they are called, read [Concepts](../concepts.md) first.

## Install

```bash
npm install @reim-developer/magical-js
```

The module is a single `.wasm` with **zero imports**, so there is no glue file and no
import object. Everything is synchronous.

## The public surface

```ts
import {
  // level 1
  detectPath, detectBytes, readHeader, neededBytes, DEFAULT_MAX_BYTES_READ,
  readLimits,
  // level 1, data first
  detected, Detected,
  // the answer type
  FileKind, isFileKind, allKinds,
  // metadata
  displayName, mime, extension,
  // the table
  describe, signatureTable,
  // asking
  matches,
  // level 2
  matchTypes, matchAllTypes, releaseRules,
} from "@reim-developer/magical-js";
```

`FileKind` is a TypeScript **string union** — the discriminants *are* the ABI. That
is the one place this binding cannot match the other two, and it is deliberate: a
union gives a caller `"Png"` narrowed to the literal `"Png"`, where an enum object
would not narrow at all across a `.wasm` boundary.

## Level 1

| Call | Returns | Notes |
| --- | --- | --- |
| `detectPath(path, options?)` | `FileKind \| null` | Synchronous, not a `Promise` |
| `detectBytes(bytes, options?)` | `FileKind \| null` | For bytes you already have |
| `readHeader(path, options?)` | `Uint8Array` | The header itself, if you want to keep it |
| `neededBytes()` | `number` | 36,870 — the window that reaches every format |
| `DEFAULT_MAX_BYTES_READ` | `number` | 2,048 — what one rule usually declares. **Not the same number** |
| `readLimits()` | an object | The smallest window that finds each format |

`readLimits()` returns a plain frozen object rather than a named type — the
declaration inlines the fields, so there is nothing to import. They are
`defaultMaxBytesRead`, `bytesRead`, `defaultOffset`, `isoMaxBytesRead`,
`tarMaxBytesRead`, `isoOffsets` and `tarOffsets`. Python's `read_limits()` returns
a `ReadLimits` class with the snake_case spelling of the same seven; Rust holds them
as `pub const`.

```ts
import { detectPath, detectBytes, neededBytes } from "@reim-developer/magical-js";

detectPath("photo.jpg");          // "Jpg"
detectBytes(new Uint8Array([0x47, 0x49, 0x46, 0x38, 0x39, 0x61]));   // "GIF"
detectPath("notes.txt");          // null
neededBytes();                    // 36870
```

`options` is `{ maxBytesRead?: number }` on the detection calls and
`{ maxBytes?: number }` on `readHeader`. Naming the window is what tells "no rule
matched" apart from "no rule *that fits the window you named* matched" — see
[Concepts](../concepts.md#none-means-one-thing-except-when-it-does-not).

A `null` is one thing here, and it is the same thing it is in the other two: **no
rule matched.** There is no "unknown format" variant, because all 114 are real
formats and a variant carrying one would be claiming knowledge the table does not
have.

## Metadata

| Call | Returns | Example |
| --- | --- | --- |
| `displayName(kind)` | `string` | `"JPEG"` |
| `mime(kind)` | `string \| null` | `"image/jpeg"` |
| `extension(kind)` | `string \| null` | `"jpg"` |

Each is a free function rather than a method because a string union cannot carry
methods in a way that survives crossing the wasm boundary, and pretending otherwise
would mean wrapping every string.

| Call | Returns | Notes |
| --- | --- | --- |
| `allKinds()` | `FileKind[]` | All 114, in the crate's own enum order — **not** the order detection walks. See [Concepts](../concepts.md#three-orders-exist-and-the-two-that-matter-are-not-the-same-sequence) |
| `isFileKind(value)` | `boolean` | A type guard, for a name from a config file or query string |
| `describe(kind)` | `Signature` | One format's entry |
| `signatureTable()` | `Signature[]` | All 114, in detection order. Walking it and stopping at the first match reproduces `detectBytes` |
| `Signature.kind` | `FileKind` | Narrowed to the literal, from `describe("Png")` |
| `Signature.signatures` | `Uint8Array[]` | The bytes compared |
| `Signature.offsets` | `number[]` | Where they are compared |
| `Signature.maxBytesRead` | `number` | What this entry declares |
| `Signature.usesPredicate` | `boolean` | Whether this entry reads structure instead of a fixed prefix |

**`offsets` is a list of positions, not a pairing with `signatures`.** Any
signature matching at any of the offsets is enough — the same rule the Python
binding follows, and the same reason a length mismatch between the two is not an
error. `usesPredicate` is the flag for the entries that cannot be expressed as bytes
at all: `ScriptExecute` and `WEBP` are the two, and they are compiled into the
module rather than compared from `signatures`.

```ts
import { describe, isFileKind } from "@reim-developer/magical-js";

const rule = describe("Png");
rule.kind;                        // "Png", typed as the literal
rule.signatures[0];               // Uint8Array [137, 80, 78, 71, 13, 10, 26, 10]
rule.offsets;                     // [0]

// A name from somewhere you do not control.
function parse(input: string): FileKind | null {
  return isFileKind(input) ? input : null;
}
```

### Walking `signatureTable()` reproduces detection

`signatureTable()` is in **detection order** — the order `detectBytes` walks — so
walking it and stopping at the first rule that matches gives the same answer. That
is what makes it worth the order being stated, and it is checked rather than assumed:
`test/kinds.test.js` tries every `(signature, offset)` pair of every format and
asserts the walk agrees with `detectBytes` on all 143 that produce an answer.

`Ktx2` is the case that makes the order matter. Its magic is
`ab 4b 54 58 20 32 30 bb 0d 0a 1a 0a`, and `Ktx`'s is `ab 4b 54 58 20` — a
byte-for-byte prefix. Ask `Ktx` first and every KTX2 file is reported as KTX1:

```ts
import { detectBytes, matches, signatureTable } from "@reim-developer/magical-js";

const ktx2 = new Uint8Array([0xAB, 0x4B, 0x54, 0x58, 0x20, 0x32, 0x30, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A]);

matches("Ktx2", ktx2);   // true
matches("Ktx", ktx2);    // true — its magic is a prefix, so both rules match

detectBytes(ktx2);                              // "Ktx2"
signatureTable().find((r) => matches(r.kind, ktx2))?.kind;   // "Ktx2"
```

`Qcow2` over `Qcow` (`51 46 49 fb` over `51 46 49`) is the other one. These two are
the whole blast radius of the order: every other format has a magic that nothing
shorter also matches.

**`signatureTable()` is not `allKinds()`.** The first is the detection order; the
second is the `FileKind` declaration order, which is what `_kinds.js` is generated
as and what `describe()` is keyed by. Both are 114 long and both are stable, so
using one where the other belongs compiles, runs, and answers a different question.

## Asking "is it this format?"

| Call | Returns | Notes |
| --- | --- | --- |
| `matches(kind, bytes)` | `boolean` | Ignoring table order — the same question as Rust's `Detect::is` |
| `detected(src).is(kind)` | `boolean` | The same question with the bytes first |

```ts
import { matches } from "@reim-developer/magical-js";

matches("Png", bytes);   // true
```

**There is no free `isAny`,** and there is a method. `isAny` over a kind and some
bytes is a loop over `matches`, which the language already has, so shipping it as a
function would be one more name to keep in step. On a [`Detected`](#detected) it is
not a wrapper over anything the caller can see — the bytes are already read and kept,
so it is a loop that does no I/O.

The deliberate part is the *name*. `detected(p).is(k)` could have been spelled
`matches(k)` to pair with the free `matches(kind, data)`, and is not, because one
name for two things with opposite argument orders is a trap rather than a pair.

```ts
import { detected } from "@reim-developer/magical-js";

detected(src).is("Png");              // true
detected(src).isAny("Png", "GIF");    // varargs
detected(src).isAny(["Png", "GIF"]);  // or one iterable
detected(src).isAny();                // false
```

A single `FileKind` is a **string**, and a string is iterable as characters — so the
one-argument case has to recognise it before treating it as a collection, or
`isAny("Png")` would test `"P"`, `"n"` and `"g"` and find nothing.

## `Detected`

The data-first spelling of `detectBytes` and `detectPath`, for a scan that asks about
every file in a directory: there the subject reads better first and the answer is
usually no.

```ts
import { detected } from "@reim-developer/magical-js";

for (const path of directory) {
  if (detected(path).isAny("Png", "GIF", "Jpg")) { ... }
}
```

`source` is a **path**, **bytes already in memory** (`Uint8Array`, `ArrayBuffer` or a
Node `Buffer`, which is a `Uint8Array`), or **anything with a synchronous
`read(size)`** — the third for a caller holding a stream it wants read once.

| Member | Returns | Notes |
| --- | --- | --- |
| `kind` | `FileKind \| null` | Same answer as `detectBytes`, cached |
| `matched` | `boolean` | Whether anything matched. **Use this, not truthiness** |
| `is(kind)` | `boolean` | One named format, ignoring table order |
| `isAny(...kinds)` | `boolean` | Varargs or one iterable. Empty is `false` |
| `within(maxBytesRead)` | `this` | Re-classifies in another window. Chains |
| `window` | `number` | The window in force, 36,870 by default |
| `rule` | `Signature \| null` | What `kind` compares |
| `mime` / `extension` | `string \| null` | |
| `displayName` | `string` | `"no match"` when nothing matched |

**Nothing is read until a member needs an answer.** Building one is free, and passing
it on never touches the disk. After that the bytes are read once, however many members
you read off it — which matters for a stream, because a pipe has its bytes once.

**`matched`, not truthiness.** A JavaScript object is always truthy, so
`if (detected(path))` is always true and there is no hook the language offers to
change it. `magical_py` writes `bool(detected(path))` because Python has `__bool__`;
this is the one thing it cannot copy, and it is why `matched` is a member rather than
something to reach for.

**There is no `toString` and no `Symbol.toPrimitive`.** Python's `__repr__` reads the
file, which is right — a repr printing the same thing for every input would be the one
thing a repr must not do. A JavaScript `toString` is not explicit: template literals
and string concatenation call it without the caller asking, so `console.log(`${d}`)`
would read a file as a side effect of logging it. Print a member instead.

`within` re-classifies what was read; it cannot reach bytes a first read did not
fetch. Narrowing always works, because what is in hand is a superset. Widening works
too, but only as far as the read went — and for **bytes already in memory** there is
no such limit, because the whole buffer was in hand before `detected` was called.

The internal cache is `#private`, not `_underscore`. The read-once promise is the
class's central guarantee and a naming convention does not enforce it: a caller who
reached in and set the cached bytes would not get a wrong answer *from* a member, they
would get a wrong answer *instead of* one, with nothing to say so.

## Level 2 — custom rules

```ts
import { matchTypes, releaseRules } from "@reim-developer/magical-js";

// `const` type parameter: the answer is the union of the kinds you declared.
const rules = [
  { kind: "Png", signatures: [PNG_BYTES], offsets: [0] },
  { kind: "GIF", signatures: [GIF_BYTES], offsets: [0] },
] as const;

const found = matchTypes(rules, bytes);   // "Png" | "GIF" | null
if (found === "Jpg") { }                  // error: "Jpg" is a real format, just not one of these two
```

| Call | Returns | Notes |
| --- | --- | --- |
| `matchTypes(rules, bytes)` | `K \| null` | The first rule that matches, typed as the union you declared |
| `matchAllTypes(rules, bytes)` | `K[]` | Every rule that matches |
| `releaseRules(rules)` | `boolean` | Drops the compiled handle. `false` if there was nothing to release |

The generics here are the point of this binding: the answer is typed as the kinds
*you* declared, not as all 114. Neither that nor `describe("Png").kind` narrowing
to a literal is expressible in JSDoc, and neither is worth giving up for a `.wasm`
that loads with no glue.

A rule set is a `readonly` array of plain objects, so it can live in a JSON file,
cross a message, or be built at run time. It is compiled into the module's linear
memory on first use.

### `releaseRules` is about handles, not memory

The compiled copy is **leaked once, deliberately**, so the `MagicCustom` values
inside the module can hold a reference to it. `releaseRules` drops the *handle*; it
does not return the memory, and the name implies something stronger than it does.

A caller who builds rule sets once at startup never needs it. A caller building
them in a loop should call it when done with one, and should understand that the
wasm-side allocation is not returned to the allocator — so the honest advice is to
build fewer, larger rule sets rather than many small ones.

## Types

`index.d.ts` is hand-written and is where the generics live. The package ships no
`.d.ts` generated from JSDoc, because the parts worth typing cannot be expressed in
JSDoc at all.

## Notes

**Everything is synchronous, deliberately.** The detection calls return their answer
rather than a `Promise`. Making them `async` would buy nothing but an `await`, and
the alternative — a real async API — means crossing a `.wasm` boundary per call,
which is the one cost this binding's single-threaded synchronous design avoids.

**The kind is a string, not an enum.** See the top of this page for why, and
[Across the three languages](../across-languages.md) for what that costs a caller
porting from the other two.