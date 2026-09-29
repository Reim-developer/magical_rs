# @reim-developer/magical-js

JavaScript and TypeScript bindings for [`magical_rs`](../../readme.md), a
zero-dependency file type detection library. 114 formats, 42 KB of WebAssembly
(17 KB gzipped), no glue file, no toolchain, and hand-written types with real
generics.

This is not a port of `file-type` or `python-magic`. The API is designed for
JavaScript, and the reason to use it over either is in the next section.

```js
import { detectPath, describe } from "@reim-developer/magical-js";

detectPath("photo.png");          // "Png"
describe("Png").signatures[0];    // Uint8Array [137, 80, 78, 71, 13, 10, 26, 10]
```

## Install

```sh
npm install @reim-developer/magical-js
```

The scope is there because `magical-js` on npm is a different package that was
published and then unpublished in 2023; the name is still held, and the registry
will not release it back.

Node 22 or newer. The package is ESM-only, and the reason is one line of code:
`_wasm.js` finds the compiled module with `new URL("./magical_js.wasm",
import.meta.url)`, which needs `import.meta.url`, which needs `"type": "module"`.
A CommonJS build would mean a second copy of the loader and a second path for
everything to go wrong.

The floor is 22 rather than 20 because `node --test` grew glob support in 22, and
the test script needs it: a quoted glob is expanded by Node rather than by the
shell, which is the only form that works on Windows — `cmd.exe` does not expand
globs, so an unquoted `test/*.test.js` finds the files on Linux and nothing at all
on a Windows contributor's machine. Listing the six files by hand instead would
mean editing two files every time a test is added, and the file that gets missed
is the one nobody runs.

## Everything is synchronous

```js
const kind = detectPath("photo.png");    // "Png", not a Promise
```

`readFileSync`, `new WebAssembly.Module` and `new WebAssembly.Instance` are all
synchronous, so the module is instantiated while this package loads and there is
nothing to await — anywhere, ever. There is no `detectBytesAsync` and no
`ready` promise to forget.

The consequence worth stating plainly: a caller porting from a promise-based wasm
library will write `await detectBytes(…)`, and get a kind back instead of a
promise. That is a `TypeError` at the first `.ext` on the result, not a silent
bug, and the error names the function.

## The types

`index.d.ts` is hand-written. That is unusual for a wasm binding, and it is the
point: the interesting types are about *your* arguments, and no tool that reads a
`.wasm` can write them.

```ts
import { describe, matchTypes } from "@reim-developer/magical-js";

// `describe` carries the name you asked about into the rule it returns, so the
// answer is `"Png"` rather than the 114-way union.
const png = describe("Png");
const name: "Png" = png.kind;    // compiles

// `matchTypes` answers with the union of the kinds *you* declared, because of the
// `const` type parameter. `as const` on a rules array is what makes the array a
// literal tuple; an inline array literal does not need it.
const rules = [
  { kind: "Png", signatures: [PNG_BYTES], offsets: [0] },
  { kind: "GIF", signatures: [GIF_BYTES], offsets: [0] },
] as const;

const found = matchTypes(rules, bytes);
found;        // "Png" | "GIF" | null
if (found === "Jpg") { }    // error: "Jpg" is a real format, just not one of these two
```

That last line is the reason this file exists. Without the `const` type parameter
the array widens to `{ kind: string }[]`, the answer becomes `string | null`, and
the comparison compiles while being a branch that can never be taken. Rules held
in a variable **must** be declared `as const`; without it they are rejected
rather than accepted with a worse answer, and `test/types.ts` pins both halves of
that.

`test/types.ts` is the drift test for this file, and it is mostly
`@ts-expect-error`. Each one asserts that a particular widening has *not* happened:
if a type widens, the directive stops being an error and `tsc` fails on the unused
directive. `npm run types` runs it.

## What is here, and what is not

The crate offers five levels of API. Two are reachable from JavaScript.

| Level | What it is | Here |
| --- | --- | --- |
| 1 | `detect`, `from_path`, `from_buffer` | Yes — `detectBytes`, `detectPath` |
| 2 | Build your own rules at runtime | Yes — `matchTypes`, `matchAllTypes` |
| 3 | Bring your own `WithFn` | **No** |
| 4 | Custom match rules with a function | **No** |
| 5 | Raw `magic_rs` pointers | **No** |

Levels 3 and 4 need a *host* function: Rust calls out to JavaScript, which means
the wasm module has to declare an **import**. This module declares none — that is
the design, and it is what lets `_wasm.js` instantiate it with `{}` and no glue
file. One import would mean every caller has to supply an import object, and
`build.mjs` fails the build if a dependency ever introduces one.

So the crate's two predicate formats are reported but not called:
`describe("WEBP").usesPredicate` is `true` and `matches("WEBP", bytes)` answers
correctly, because the predicate is compiled *into* the module and called from
inside it. What cannot cross the boundary is a predicate you write in JavaScript.

Level 5 is meaningless in wasm: it hands out `magic_rs` pointers, and there is no
`magic_rs` address space on the other side of a `WebAssembly.Instance`. The Rust
in this package *does* offer that, to Rust callers, because it is built as an
`rlib` as well as a `cdylib`.

## The API

### Detection

```js
detectBytes(data, options?)   // FileKind | null
detectPath(path, options?)    // FileKind | null
readHeader(path, options?)    // Uint8Array
neededBytes()                 // 36870
```

`options.maxBytesRead` narrows the table to rules that read no further than that.
A format whose own `maxBytesRead` is larger cannot be returned, **even when the
buffer is long enough for its signature** — this is the crate's
`match_with_max_read_rule`, and it is the one option here that changes answers
rather than just their cost. In `detectPath` it also decides how much is read off
disk, so a small window is a small read.

### Introspection

```js
describe("Png")      // Signature<"Png">
signatureTable()     // Signature[] — 114 rows, in the crate's own order
allKinds()           // FileKind[] — 114 names
isFileKind(value)    // type guard
readLimits()         // the crate's read limits, as one object
```

`signatureTable()` is not a live view of the crate's `Magic` values. The table
crosses the boundary as one hand-written TLV blob, because `Magic` is
`#[repr(Rust)]` and its field order is not part of any promise. The blob stores
each entry's discriminant explicitly, and it has to: 71 of the 114 entries are not
in declaration order, and the two formats that share a signature prefix are
swapped relative to it. A decoder that indexed by position would attribute every
rule to the wrong format.

`Signature` has a fifth field the Python binding does not: `usesPredicate`, and
`ScriptExecute` reports *both* `usesPredicate: true` and a real `#!` signature. The
Python binding blanks the signature of every predicate entry. `#!` is a genuine
prefilter the crate uses, so hiding it would make this a worse description of how
detection actually works.

### Custom rules

```js
matchTypes(rules, data)       // the first rule's kind, or null
matchAllTypes(rules, data)    // every matching rule's kind, in order
releaseRules(rules)           // drop the compiled copy
```

A rule is `{ kind, signatures, offsets }`: any signature at any offset is a match.
`kind` is any string, not one of the 114 — level 2 is for formats this package has
never heard of.

A rule with no signature or no offset is rejected, by the types (`readonly
[T, ...T[]]`) and by the loader (`TypeError`). A rule that can never match is
indistinguishable from a correct one at the call site, and "the rule set was
silently half-built" is not a bug worth rediscovering in production.

`matchAllTypes` does not exist in the crate. It is the crate's own matcher called
once per rule over a one-rule slice, rather than a second implementation of what a
match is — so the two functions cannot disagree about what matches.

Rule sets are compiled on first use and cached on the array you pass, so the same
`rules` in a loop compiles once. `releaseRules` drops the handle. It does not free
the memory: the compiled copy is leaked once so `MagicCustom` can hold a
reference to it, which is a deliberate trade against allocating a fresh `Vec` per
call, and it is recorded in `src/lib.rs`. A caller building rule sets in a loop
rather than at startup should call it when done with one.

## How it works

`src/lib.rs` is a `cdylib` of raw `extern "C"` exports. No `wasm-bindgen`, no
`wasm-pack`, no `bg.js` beside the module, and no toolchain in CI. The memory ABI
is documented at the top of `_wasm.js` and it is short:

- The module allocates its own scratch buffers (`input_ptr`, `answer_ptr`) and the
  loader writes where it is told. Address 0 is **not** free: the encoded table sits
  within a few hundred bytes of the top of the initial memory, so a caller writing
  a one-megabyte file from offset 0 overwrites the detection table. The module keeps
  answering afterwards; the answers are just answers from a shredded table.
- `memory.buffer` is replaced whenever memory grows, detaching every view onto the
  old one, so nothing caches a `Uint8Array`.
- An `i32` *result* arrives signed, so Rust's `u32::MAX` "no match" sentinel is
  `-1` in JavaScript, not `4294967295`.
- A `bool` result arrives as `0` or `1`.

`scripts/build.mjs` asserts the first and third of those against the built
artifact, and checks that every export the loader calls is present. Nothing in the
Rust build would notice the no-imports claim going false; adding a dependency that
needs an import compiles perfectly well, and the symptom would appear later as a
`LinkError` in somebody else's project.

## Two things this binding does not do

**No MIME types or extensions.** The Python binding parses `readme.md` for its
table; this one reports signatures and read limits, and `describe` does not
answer "what should I call this file". A follow-up is possible, and the format
count is pinned at 114 in both `scripts/gen_kinds.mjs` and `test/kinds.test.js`,
so adding a format is a deliberate edit.

**No CommonJS.** See "Install" above.

## Building from source

```sh
npm run gen      # regenerate _kinds.js and _kinds.d.ts from src/magical/magic.rs
npm run build    # cargo build --target wasm32-unknown-unknown, install, and verify
npm test         # build, then node --test
npm run types    # tsc --noEmit over index.d.ts and test/types.ts
npm run gates    # all of the above, in the order CI runs them
```

`npm run build` needs the `wasm32-unknown-unknown` target
(`rustup target add wasm32-unknown-unknown`) and nothing else — no `wasm-opt`, no
`wasm-pack`, no `twiggy`. The release profile is `opt-level = "s"` with LTO,
`codegen-units = 1` and `panic = "abort"`; `s` over `z` because the measured
difference is 243 bytes and `s` keeps the matching loop faster for a caller
scanning a directory.

`_kinds.js` and `_kinds.d.ts` are generated and checked in. The names and their
order come from `pub enum FileKind` in `src/magical/magic.rs`, because the order
*is* the ABI: a format crosses the wasm boundary as its Rust discriminant. They
are committed because `test/kinds.test.js` compares them against the compiled
module, and a generated file nobody can diff is a generated file nobody can
review.

## Licence

MIT, the same as the crate this wraps. See [LICENSE](../../LICENSE). There is no
second copy of the licence file beside this README on purpose: one file, one place
to change it, and the `license` field in `package.json` is what npm shows.
