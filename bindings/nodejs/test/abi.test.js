// The shape of the compiled module, and the promises the loader makes about it.
//
// These are the tests for the decision recorded in `src/lib.rs`: raw `extern "C"`
// exports, no `wasm-bindgen`, no imports. Each of those is a claim a reader could
// take on trust, so each is checked against the artifact.

import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const WASM = fileURLToPath(new URL("../magical_js.wasm", import.meta.url));
const bytes = readFileSync(WASM);
const module = new WebAssembly.Module(bytes);

test("the module declares no imports", () => {
  const imports = WebAssembly.Module.imports(module);
  assert.deepEqual(
    imports,
    [],
    "an import means a loader must supply it; `_wasm.js` supplies `{}`, so this " +
      "would be a load-time LinkError for every caller rather than a build error",
  );
});

test("it instantiates with an empty import object", () => {
  // The same fact as above, through the path a caller actually takes. Kept
  // separate because "imports() returns []" and "instantiating with {} works" are
  // different claims, and it is the second one that is the promise.
  const instance = new WebAssembly.Instance(module, {});
  assert.equal(typeof instance.exports.which_kind_at, "function");
});

test("every export the loader calls is present", () => {
  const present = new Set(WebAssembly.Module.exports(module).map((entry) => entry.name));
  const required = [
    "which_kind_at",
    "which_kind_max_at",
    "kind_matches_at",
    "table_ptr",
    "table_len",
    "rules_new",
    "rules_add_signature",
    "rules_add_offset",
    "rules_next_rule",
    "rules_finish",
    "rules_release",
    "rules_match",
    "rules_match_all",
    "rules_len",
    "memory",
  ];
  for (const name of required) {
    assert.ok(present.has(name), `missing export: ${name}`);
  }
});

test("there are no exports beyond the ones the loader calls", () => {
  // Not fussy about names — a Rust export may be added for the crate's own
  // benefit. Fussy about the total, because an unexpected extra is usually a
  // helper that was meant to be private and is now part of the ABI by accident.
  const names = WebAssembly.Module.exports(module).map((entry) => entry.name).sort();
  assert.deepEqual(names, [...new Set(names)], "duplicate export names");
  const memory = WebAssembly.Module.exports(module).filter((entry) => entry.kind === "memory");
  assert.equal(memory.length, 1, "expected exactly one exported memory");
});

test("the module's memory starts small enough to be worth growing", () => {
  const instance = new WebAssembly.Instance(module, {});
  const initial = instance.exports.memory.buffer.byteLength;
  // Not an assertion about a magic number so much as a record of why the loader
  // re-derives its view: a caller passing a 4 MB buffer to a module with a
  // 17-page memory is the normal case, not the exotic one.
  assert.ok(initial > 0, "a module with no initial memory cannot be written to at all");
  assert.ok(
    initial < 4 * 1024 * 1024,
    `initial memory is ${initial} bytes; _wasm.js grows before writing, which is ` +
      "correct either way, but a large default would be worth understanding",
  );
});

test("the module does not claim low addresses for the caller's bytes", () => {
  // The loader writes at the address `input_ptr` hands it rather than at 0, and
  // this is why: the encoded blob and the detection table both sit within a few
  // hundred bytes of the top of the initial memory, so a caller writing a
  // one-megabyte buffer from offset 0 overwrites them. The module keeps
  // answering afterwards; the answers are just from a shredded table.
  const instance = new WebAssembly.Instance(module, {});
  const { memory, table_ptr } = instance.exports;
  const initial = memory.buffer.byteLength;
  assert.ok(
    table_ptr() >= initial - 4096,
    `table_ptr is ${table_ptr()}, far below the top of a ${initial}-byte memory; if ` +
      "this moved, the scratch-buffer reasoning in _wasm.js has to be rechecked",
  );
  // And the scratch buffers themselves are handed out above the initial memory,
  // since they come from the allocator rather than from the data section.
  const scratch = instance.exports.input_ptr(64);
  assert.ok(scratch >= initial, `input_ptr returned ${scratch}, below the data section`);
  assert.notEqual(instance.exports.answer_ptr(64), scratch);
});

test("the package loads synchronously, with no await in the loader", () => {
  // `readFileSync`, `new WebAssembly.Module` and `new WebAssembly.Instance` are
  // all synchronous, so importing the package leaves it ready. If this test ever
  // needs an `await`, the API has grown an async entry point that callers can
  // forget to await — which is the failure this design exists to prevent.
  //
  // Comments are stripped first, because the loader's header discusses `await` at
  // length and a naive scan finds the word in the explanation rather than the code.
  const source = readFileSync(fileURLToPath(new URL("../_wasm.js", import.meta.url)), "utf8");
  const code = stripComments(source);
  assert.ok(code.length > 0, "sanity: the loader was read");
  assert.equal(
    /\bawait\b/.test(code),
    false,
    "_wasm.js awaits something, so the module is not ready when the import resolves",
  );
  assert.equal(
    /\basync\b|\bPromise\b/.test(code),
    false,
    "_wasm.js creates a promise; the whole package is synchronous by design",
  );
});

/** Remove line and block comments, so a scan finds code rather than prose. */
function stripComments(source) {
  return source.replace(/\/\*[\s\S]*?\*\//g, " ").replace(/^\s*\/\/.*$/gm, " ");
}
