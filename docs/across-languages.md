# Across the three languages

One crate, three bindings. They are not three ports of the same API that happen to
look different — parity is a goal, and where it is not reached the difference is
listed here rather than left to be discovered.

If you are porting, this is the page. If you are choosing, the differences below are
the ones that would change your code.

## The table

| | Rust | Python | JavaScript |
| --- | --- | --- | --- |
| Unknown answer | `Option<FileKind>` | `FileKind \| None` | `FileKind \| null` |
| The kind itself | an enum | an enum | a `string` union — the discriminants *are* the ABI |
| Detection call | `FileKind::match_types` | `detect` / `detect_bytes` | `detectPath` / `detectBytes` |
| Data comes first | `bytes.detect()`, behind a flag | `detected(path).kind` | not possible — see below |
| "Is it this format?" | `Detect::is` | `Detected.matches`, or `FileKind.matches` the other way round | `Detected.is`, or `matches(kind, bytes)` |
| "Any of these?" | `Detect::is_any` | `Detected.matches_any` | `Detected.isAny` |
| Did anything match? | `.detect().is_some()` | `bool(detected(path))` | `detected(path).matched` — see below |
| Narrow the window | `match_with_max_read_rule` | `max_bytes_read=` on every call | `options.maxBytesRead`, or `.within()` |
| Report the window | `with_bytes_read()` | `bytes_read()` | `neededBytes()`, or `.window` |
| The per-rule default | `DEFAULT_MAX_BYTES_READ` | `DEFAULT_MAX_BYTES_READ` | `DEFAULT_MAX_BYTES_READ` |
| A rule set lives in | the caller's binary | the caller's objects | compiled into the module, leaked once |
| Rule sets are mutable | no, `&'static` | the list is yours | no, `readonly`, compiled on use |
| Custom kinds | any `Clone` type | any object | a union of the literals you declared |
| `no_std` | yes, levels 1 and 2 | not applicable | not applicable — the module is `wasm32` |
| Async detection | level 4, `magical_async_dyn` | level 4, `AsyncDynMagic` | none; `AsyncDynMagic` is the reason |

## The six that will surprise somebody

### Python's "is this format" is `matches`, and JavaScript's is `is`

`is` is a keyword in Python, so `what.is(FileKind.Png)` is a `SyntaxError` and the
method is spelled `matches`. JavaScript has no such problem, so it spells it `is` —
which is Rust's name and the one the vocabulary comes from.

Neither is a loss, and Python's is not even a compromise: `matches` is the inverse of
the `FileKind.matches(data)` that binding already had, so the two read as a pair.

```python
kind.matches(data)    # kind first, has always existed
data.matches(kind)    # data first, on a `Detected`
```

JavaScript could have taken the same pair — and deliberately did not, because it
*already* exports a free function called `matches(kind, data)` taking the arguments
the other way round. A method reading `detected(p).matches(k)` beside a function
reading `matches(k, d)` would be one name for two things with opposite argument
orders. `is` and `isAny` cannot be confused with either.

### JavaScript cannot ask whether anything matched

`bool(detected(path))` works in Python because `__bool__` returns the answer. A
JavaScript object is always truthy, so `if (detected(path))` is always true and
there is no hook the language offers to change it. `matched` is the stand-in, and it
is a member rather than something to reach for:

```js
if (detected(path).matched) { ... }   // correct
if (detected(path)) { ... }           // always true, whatever the file is
```

### JavaScript has no `toString` on its wrapper, on purpose

Python's `__repr__` reads the file, which is right: a repr that printed the same
thing for every input would be the one thing a repr must not do. A JavaScript
`toString` is not explicit — template literals and string concatenation call it
without the caller asking — so `console.log(`${d}`)` would read a file as a side
effect of logging it. Print a member instead.

### The rule-set lifetime differs, and the boundary is the reason

In Rust and Python your rules are yours and cost nothing to keep. In JavaScript
they cross into a module whose address space you do not own, so **a loop that
compiles a fresh rule set per file grows that module's heap per file.**
`releaseRules` bounds the handles; it does not return the memory. Build fewer,
larger rule sets.

### The kind is an object in two languages and a string in the third

Rust and Python hand back an enum member. JavaScript hands back a string, because
the discriminants cross the wasm boundary as numbers and a union of string literals
is what a TypeScript caller can actually narrow on. The cost is real: `kind === "Png"`
rather than `kind === FileKind.Png`, and no compile error for a typo. `isFileKind`
is the guard for a value from outside your code.

### Custom kinds are typed differently three ways

Rust takes any `Clone` type. Python takes any object. JavaScript takes the union of
the literals you declared, through a `const` type parameter on `as const` — so a
typo in a kind name is a compile error, and so is comparing the result against a
real format that is not one of yours.

```ts
const found = matchTypes(rules, bytes);
if (found === "Jpg") { }   // error: real format, not one of these rules
```

## What is genuinely the same

- **The rule table, and the order detection walks it.** `Ktx2` precedes `Ktx` in all
  three because KTX1's magic is a byte-for-byte prefix of KTX2's.
- **Every answer.** All three bindings call the same Rust code over the same table.
  A format detected in one is detected identically in the others, including the
  formats that read structure rather than a fixed prefix.
- **The numbers.** 36,870 and 2,048 mean the same thing everywhere, and both are
  named rather than inlined.
- **`None`/`null` meaning "no rule matched"** from the unfiltered call, and nothing
  else. Only the window-taking calls can return it for a second reason, and all
  three let you name the window so the two are distinguishable.

## What is not the same, and it is not the same in all three

The *enumeration* order differs between the bindings, which matters only if you
persist an index — and it is exactly the case where persisting one is tempting.

| | Order |
| --- | --- |
| Rust `FileKind as isize`, JS `FileKind`, Python `FileKind` member order | the enum declaration, which is the table order with two deliberate exceptions |
| Detection order, in all three | `SIGNATURE_KIND` |
| `kinds_meta::ALL_KINDS`, Rust | sorted by variant name |

`ALL_KINDS` is the odd one out and it is easy to get wrong: it looks like the
authoritative list of formats and it is sorted alphabetically, so
`ALL_KINDS[47]` is not what `SIGNATURE_KIND[47]` is.
[Concepts](concepts.md#three-orders-exist-and-the-two-that-matter-are-not-the-same-sequence)
has the measured numbers.

The two *introspection* calls agree, and that is the useful part: Rust's
`SIGNATURE_KIND`, Python's `signature_table()` and JavaScript's `signatureTable()`
all report the rules in **detection order**, so "walk the list and stop at the first
match" reproduces detection in all three. `allKinds()` / `ALL_KINDS` do not, and
should not — they are the declaration order, which is what the enum's discriminants
and `describe()` are keyed by.

## Porting notes

**Rust to Python.** `FileKind::match_types(&bytes)` → `detect_bytes(bytes)` or
`detect(path)`. `kind.mime()` → `kind.mime`. `variant_name()` → `kind.value`, and
the Python member's own name (`FileKind.Jpg`) is the Rust variant name.

**Python to Rust.** The `Predicate` classes in `magical_py` are Python callables;
Rust's equivalents are closures behind `magical_dyn`, or `with_fn_matches!` in a
level 2 rule set when the whole set is fixed at compile time. `match_types_custom_all`
has no Rust counterpart at level 2 — it is `CustomMatchRules::AllMatches`, which is
level 5 and `unsafe`.

**Any of them to JavaScript.** Everything is synchronous, so an async call site
becomes a synchronous one. `detect_bytes` takes a `Uint8Array`; a Python `bytes` or
a Rust `&[u8]` becomes `new Uint8Array(...)` at the boundary. Rule sets become
`as const` arrays of plain objects.

## Where the rest of the reference is

- [Rust](api/rust.md) · [Python](api/python.md) · [JavaScript](api/javascript.md)
- [Concepts](concepts.md) — table order, the read window, what `None` means
- [Detection levels](detection-levels.md) — levels 2 to 5, all three languages