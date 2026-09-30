# Examples

Runnable, and run by `npm run gates` — so a broken one is a failing check rather
than a file in a directory.

```sh
node examples/01_detect_a_file.mjs
node examples/02_detect_bytes.mjs
node examples/03_list_supported_formats.mjs
node examples/04_scan_a_directory.mjs
node examples/05_custom_rules.mjs
node --experimental-strip-types examples/06_typed.ts
npm run types      # type-checks 06_typed.ts as well as the declarations
```

The imports are relative (`../index.js`) rather than bare
(`@reim-developer/magical-js`) so the examples run from a checkout with nothing
installed. In a project of your own they are the bare specifier.

Nothing here is a fixture file. Every header is written out from its magic bytes,
because a `.png` in the repository is a binary nobody reviewing a diff can read and
one that has to be regenerated whenever the detection table changes. Writing the
signature longhand says exactly what is being matched.

| | |
| --- | --- |
| `01_detect_a_file.mjs` | `detectPath`, and the three things detection cannot answer: name, media type, extension. Why a `.webp` holding a PNG is reported as a PNG. |
| `02_detect_bytes.mjs` | `detectBytes`, and `maxBytesRead` — the one option here that changes *answers* rather than their cost. The crate's own read limits, and a 4 MB buffer, because a cached `memory.buffer` view is a bug that looks like "no files recognised". |
| `03_list_supported_formats.mjs` | `allKinds`, tallied by media type. Which formats answer `null` and why nothing is invented. How each rule matches, which is a different question from what the table holds. |
| `04_scan_a_directory.mjs` | Walking a tree, with a `FileKind` as the tally key so no parsing or dictionary keyed by media type is needed. |
| `05_custom_rules.mjs` | Level 2: `matchTypes` against `matchAllTypes`, why a half-built rule set is rejected, what `releaseRules` does and does not free, and `matches` answering a different question from `detectBytes`. |
| `06_typed.ts` | What the declarations buy. Every `@ts-expect-error` is a claim that a particular widening has *not* happened; if one ever does, `tsc` fails on the unused directive. |

## Two `tsconfig.json`s, and why

`06_typed.ts` is type-checked by `examples/tsconfig.json`, which extends the one at
the package root and adds `"dom"` to `lib` — for `console`.

The root project sets `"types": []` and `lib: ["es2023"]` on purpose: the public
surface is a `Uint8Array`, a `number`, a `string` or `null`, and a binding that
needed `@types/node` would not also run in a browser. Widening the package's own
project to accommodate an example would quietly weaken the property it asserts, so
the example gets its own. Two config files, no build step, and the package's type
surface stays free of both.

## What an example is for

Each of these was written to be read, so the comments answer "why would I write it
this way" rather than restating the line below. Where a number appears it is
measured rather than asserted — `readLimits()` is called instead of writing 36,870,
and `allKinds().length` instead of 114 — so an example cannot go quietly out of
date the way a hard-coded value in a README does.

Two of them also correct a mistake a reader would otherwise make:

- `01` — a file's *name* is never read. A `.jpg` holding a PNG is a PNG, and that
  is not an edge case.
- `02` — `maxBytesRead: 2048` makes ISO 9660 undetectable even when the buffer
  contains its magic. The bytes are present; the rule is not allowed to look.

## Not here

A browser example. The module declares no imports and has no file-system access, so
it works in a page, but a page cannot be run by `npm run gates` and an example
nothing executes is documentation with a directory around it. The claims that would
be checked there — zero imports, every export present — are checked at build time by
[`scripts/build.mjs`](../scripts/build.mjs) instead.
