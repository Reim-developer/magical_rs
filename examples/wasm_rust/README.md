# `magical_rs` on WebAssembly, without a binding

`magical_rs` is a library, and a library that runs on `wasm32-unknown-unknown`
needs nothing from anyone. This example is the smallest thing that can prove it:
four exports, no glue file, no import object, no bundler.

```sh
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
node load.mjs
```

```
module: 32847 bytes, 4 exports, 0 imports
  no imports, so no glue file and no import object
  exports: memory, display_name_bytes, display_name_len, which_kind_at

    a PNG header           PNG (discriminant 0)
    a GIF header           GIF (discriminant 36)
    a ZIP header           Zip / JAR / APK (discriminant 4)
    a WASM header          WebAssembly (discriminant 15)
    a PDF header           PDF (discriminant 38)
    an ELF header          ELF (discriminant 19)
    a tar header at 257    Tar (discriminant 5)
    an ISO header at 32769 ISO 9660 (discriminant 10)
    not a format           no signature matched

9 fixtures, all correct.
```

The expected name is in the fixture table, so a wrong answer is a non-zero exit and
`make examples` goes red. That is the difference between this being an example and
being a check, and it is worth having here specifically: a crate whose wasm module
compiled and then answered "PNG" for a GIF would pass every other gate in this
repository.

Two of those rows are load-bearing rather than decorative:

- **`a tar header at 257`** — the buffer is 512 bytes and the signature is at 257,
  so a matcher reading from offset zero would find nothing. It is the only row that
  proves an *offset* is honoured rather than a length being large enough.
- **`an ISO header at 32769`** — 40,000 bytes, so it forces the module's memory to
  grow. A loader that cached its `memory.buffer` view would answer `null` from that
  row onwards, which is exactly the failure this file exists to be able to show.

## Why this exists

`make build-wasm` at the repository root proves the crate *compiles* for
`wasm32-unknown-unknown`, and its own comment is careful about how little that
means: a wasm32 `std` has a file system, a process API, sockets and threads that
all compile and then fail at runtime. What that build catches is target-gated `std`
— `std::os::unix` and friends, which resolve on a host that is one of them.

It does not catch "the module runs and answers correctly". That is what `load.mjs`
is for, and `make examples` runs it in CI.

## 32,847 bytes against 44,616

The npm binding's module is 44,616 bytes. This one is 32,847. The 12 KB difference
is everything a *binding* is and a *library* is not:

| | this example | `bindings/asm` |
| --- | --- | --- |
| exports | 4 | 17 |
| detection table in the module | no — it calls the crate's matcher | yes — 114 entries encoded into linear memory at import |
| level 2 runtime rules | no | yes — `rules_new` through `rules_match_all` |
| introspection API | no | yes — `describe`, `signatureTable`, `readLimits` |
| versioned ABI | no | yes, and released |
| npm package | no | `@reim-developer/magical-js` |

Neither is better. This one is what you would write if you were embedding detection
in your own wasm module and already had a memory ABI; the binding is what you would
install if you did not.

## The one thing you cannot pass

A `&str`. Rust strings are `(pointer, length)` with no terminator, so handing
`str::as_ptr` across the boundary gives the caller no way to know where it ends.
This example returns the pointer *and* the length:

```rust
pub extern "C" fn display_name_len(kind: u32) -> u32 { /* … */ }
pub extern "C" fn display_name_bytes(kind: u32) -> *const u8 { /* … */ }
```

and the loader reads exactly that many bytes. The alternative is a NUL-terminated
copy into a buffer the host provides, which works and costs a buffer. Neither is
free, and the choice belongs to whoever owns the boundary.

## `as u32` is the ABI

`FileKind` is a fieldless enum, so `kind as u32` *is* its discriminant, and that
number is what every binding's name table is indexed by. This is why the order of
the declaration in `crates/magical_rs/src/magical/magic.rs` is load-bearing rather
than a detail, and why `bindings/nodejs/test/kinds.test.js` compares all 114 names
against the discriminants a compiled module actually reports.

`from_discriminant` goes through `ALL_KINDS` rather than transmuting the `u32` back.
A transmute of an arbitrary integer into an enum is undefined behaviour when the
value is out of range, and this is an `extern "C"` function — the last place in a
program you want that. A scan of 114 entries costs nothing and turns an out-of-range
value into `None`.

## `std`, and what it is doing here

`which_kind_at` uses `std::slice::from_raw_parts`, so this module is a `std` module.
That is not the crate's requirement: `FileKind::match_types` works under `no_std`,
and `make build-nostd` proves it by building `magical_rs` for
`thumbv7em-none-eabi`. A wasm32 host is a `std` host anyway — wasm32 ships a `std`
whose file-system and socket calls all compile and then fail at runtime — so there
is nothing to gain here by pretending otherwise.

## Not a binding

There is no public API here, no introspection, no custom rules and no compatibility
promise across versions. Do not build on it.
[`bindings/asm`](../../bindings/asm) is the thing with a released ABI, and
[`bindings/nodejs`](../../bindings/nodejs) is the package.
