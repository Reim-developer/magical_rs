// The public surface: what `index.js` exports, against what `index.d.ts` declares.
//
// The declaration file is hand-written, so nothing stops the two from drifting —
// a function renamed in the implementation is still declared, and TypeScript
// believes it. This compares the name sets in both directions, so either a
// declaration with no implementation or an implementation with no declaration is
// a failing test rather than a mistyped import in somebody's project.

import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

import * as api from "../index.js";

/** Read a file from the package root. */
function source(name) {
  return readFileSync(fileURLToPath(new URL(`../${name}`, import.meta.url)), "utf8");
}

/**
 * Every value `index.d.ts` promises at runtime.
 *
 * Two forms count: `export declare function|const`, for what the file defines, and
 * `export { name } from "..."`, for what it re-exports — which is how `FileKind`
 * reaches the surface, so a scan missing the second form would report a name that
 * is both declared and missing.
 *
 * A regex rather than the compiler, because this test runs under plain Node with
 * no TypeScript installed, and a package that needs a compiler to check its own
 * surface is a package with a toolchain. Comments are stripped first: the
 * declaration file explains *why* it is hand-written and says so in prose, and a
 * naive scan finds the words it is looking for in the explanation.
 */
function declaredValues(text) {
  const code = stripComments(text);
  const names = new Set();
  for (const match of code.matchAll(/^export declare (?:function|const) (\w+)/gm)) {
    names.add(match[1]);
  }
  // `FileKind` is the one re-export, and it is written `export { FileKind };`
  // after a bare `import`, not `export … from`, so the name has no module
  // specifier to pattern-match against.
  for (const match of code.matchAll(/^export \{ (\w+) \};?$/gm)) {
    names.add(match[1]);
  }
  return names;
}

/** Remove line and block comments, so a scan finds code rather than prose. */
function stripComments(source) {
  return source.replace(/\/\*[\s\S]*?\*\//g, " ").replace(/^\s*\/\/.*$/gm, " ");
}

/** Every value the package exports at runtime. */
function exportedValues(module_) {
  return new Set(Object.keys(module_));
}

test("every declared value is exported", () => {
  const declared = declaredValues(source("index.d.ts"));
  const actual = exportedValues(api);
  assert.ok(declared.size >= 12, `only ${declared.size} declarations found — did the regex break?`);
  for (const name of declared) {
    assert.ok(actual.has(name), `index.d.ts declares ${name}, which index.js does not export`);
  }
});

test("every exported value is declared", () => {
  const declared = declaredValues(source("index.d.ts"));
  const actual = exportedValues(api);
  for (const name of actual) {
    assert.ok(declared.has(name), `index.js exports ${name}, which index.d.ts does not declare`);
  }
});

test("the declaration file's types are all defined, and all of them are public", () => {
  // A type cannot be compared against the runtime, so `test/types.ts` is where the
  // types are really checked — by a compiler, against real call sites. What this
  // catches is the cheaper failure: a name left in a signature after the
  // interface it referred to was deleted, which a hand-written file can do and a
  // generated one cannot.
  const code = stripComments(source("index.d.ts"));
  const declaredTypes = new Set(
    [...code.matchAll(/^export (?:type|interface) (\w+)/gm)].map((match) => match[1]),
  );
  // Five: `Signature`, `DetectOptions`, `ReadHeaderOptions`, `MatchRule`,
  // `MatchRules`, plus the re-exported `FileKind`. Stated so that deleting one is a
  // deliberate edit rather than a quiet loss.
  assert.deepEqual(
    [...declaredTypes].sort(),
    ["DetectOptions", "MatchRule", "MatchRules", "ReadHeaderOptions", "Signature"],
  );

  // Every type named in a signature is one of those five, or an imported builtin.
  // PascalCase only, so `DEFAULT_MAX_BYTES_READ` — a value, not a type — is not
  // swept up by the scan looking for dangling references.
  const used = new Set(
    [...code.matchAll(/\b([A-Z][a-z]\w*)\b/g)].map((match) => match[1]),
  );
  for (const name of used) {
    if (["FileKind", "Uint8Array", "ArrayBuffer", "Promise"].includes(name)) continue;
    assert.ok(
      declaredTypes.has(name),
      `index.d.ts mentions ${name}, which it does not declare and does not import`,
    );
  }
});

test("the package exports nothing it did not mean to", () => {
  // An internal module reached by mistake — `_wasm` in particular, whose exports
  // are pointers into linear memory and whose contract nobody documented.
  for (const name of Object.keys(api)) {
    assert.ok(
      !name.startsWith("_") && !name.startsWith("wasm") && !name.startsWith("rules"),
      `internal export leaked into the public surface: ${name}`,
    );
  }
});

test("the barrel is a barrel, and the internals stay internal", async () => {
  // Every public name should come from one of the four internal modules, and
  // nothing outside the package should need to import those. `package.json`'s
  // `exports` map is what enforces that, so it is checked too.
  const pkg = JSON.parse(source("package.json"));
  assert.deepEqual(Object.keys(pkg.exports), [".", "./package.json"]);
  assert.equal(pkg.exports["."].types, "./index.d.ts");
  assert.equal(pkg.exports["."].default, "./index.js");
  assert.equal(pkg.types, "./index.d.ts");
  assert.equal(pkg.main, "./index.js");

  // The internals really are importable by path inside the package — which is
  // what the tests do — but not from outside it, since `exports` is a closed map.
  const internals = ["./_wasm.js", "./_levels.js", "./_signatures.js", "./_kinds.js"];
  for (const name of internals) {
    assert.ok(
      !Object.prototype.hasOwnProperty.call(pkg.exports, name),
      `${name} is in the exports map, so it is part of the public API`,
    );
  }
});

test("the compiled module is inside the files the package ships", () => {
  // `.wasm` is a binary npm will not add on its own. If it is not in `files`, the
  // published package imports fine locally and throws at install time everywhere
  // else — with a message about a missing file, which is a bad first impression
  // and a worse bug report.
  const pkg = JSON.parse(source("package.json"));
  for (const required of [
    "index.js",
    "index.d.ts",
    "_kinds.js",
    "_kinds.d.ts",
    "_levels.js",
    "_signatures.js",
    "_wasm.js",
    "magical_js.wasm",
  ]) {
    assert.ok(pkg.files.includes(required), `package.json files is missing ${required}`);
  }
});

test("the generated kinds files agree with each other", () => {
  const js = source("_kinds.js");
  const dts = source("_kinds.d.ts");
  // Both are generated from the same list in one run, so a name in one and not the
  // other means the generator emitted something different to each file — which is
  // how a name gets a type but no value.
  for (const name of ["FILE_KIND_INDICES", "FILE_KIND_NAMES", "FileKind"]) {
    assert.ok(js.includes(name), `_kinds.js is missing ${name}`);
    assert.ok(dts.includes(name), `_kinds.d.ts is missing ${name}`);
  }
  // Comments stripped, because the generated header explains that the types live
  // in the `.d.ts` *by naming the syntax it is keeping out of the `.js`*.
  const code = stripComments(js);
  assert.ok(
    !/\bas const\b/.test(code),
    "_kinds.js is run by Node, which does not parse `as const`; the types belong in the .d.ts",
  );
  assert.ok(
    !/\bexport type\b/.test(code),
    "_kinds.js is run by Node, which does not parse `export type` either",
  );
  assert.ok(!/\bawait\b/.test(code), "nothing should be asynchronous at import time");
});

test("the loader does its work at import time, not on first call", async () => {
  // A blob that does not match the decoder is a mismatched `.wasm`. Failing while
  // the stack still says which import broke is the difference between a useful
  // error and one that surfaces from inside somebody's `try` block.
  const fresh = await import(`../index.js?cache=${Date.now()}`);
  assert.equal(fresh.detectBytes(new Uint8Array()), null);
});

test("the package has no runtime dependencies", () => {
  const pkg = JSON.parse(source("package.json"));
  assert.equal(pkg.dependencies, undefined, "a zero-dependency binding grew one");
  assert.deepEqual(Object.keys(pkg.devDependencies), ["typescript"]);
});
