// The README's claims, checked against the module.
//
// This file exists because the README was wrong once already. It documented
// `detectPath("photo.heic")` as answering `"Heif"`, which is a format name this
// crate does not have — it answers `IsoMedia`, because a `.heic` is an ISO
// base-media container. Nothing caught it, because the tests assert what the code
// does and the README is prose about what the code does, and the two were never
// compared.
//
// So: every number and every name below is a sentence from the README, and each
// one is asserted here. Where a claim is an exact value it is asserted exactly.
// Where it is a size, which moves with the compiler version, it is asserted as an
// order of magnitude and the README is quoted — pinning 43,216 bytes exactly
// would mean a test that fails the day someone upgrades Rust, which is the kind
// of failure people learn to delete.

import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync, statSync, mkdtempSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { gzipSync } from "node:zlib";

import * as api from "../index.js";

const README = readFileSync(fileURLToPath(new URL("../README.md", import.meta.url)), "utf8");
const WASM = fileURLToPath(new URL("../magical_js.wasm", import.meta.url));
const PKG = JSON.parse(
  readFileSync(fileURLToPath(new URL("../package.json", import.meta.url)), "utf8"),
);

test("the README says the package's real name, and never an old one", () => {
  // The name is the one thing in a package that is wrong everywhere at once: the
  // install line, every import, the headings, and the prefix on 25 error messages.
  // Nothing about a rename makes any of those fail, so the tests all stay green
  // over a README that tells people to install a package that does not exist.
  //
  // It was not hypothetical. `magical-js` — the name this binding was written with
  // — is held on npm by a different package that was published and then
  // unpublished in 2023, so the rename to a scope was forced, and every one of
  // those places needed changing by hand.
  assert.equal(PKG.name, "@reim-developer/magical-js");

  // The install line, exactly. `npm i` is not accepted as a synonym here because
  // the README only uses the long form and a synonym would make the assertion
  // pass on a line nobody writes.
  assert.match(README, new RegExp(`^npm install ${escape(PKG.name)}$`, "m"));
  assert.doesNotMatch(README, /^npm install magical-js$/m);

  // The heading is the name a reader sees first, on npmjs.com and in a file
  // tree alike.
  assert.match(README, new RegExp(`^# ${escape(PKG.name)}$`, "m"));

  // Every import in the README resolves to this package, or to a Node builtin.
  // This is the assertion that would have caught the rename, because an unscoped
  // import line is a copy-paste that still looks right. Nothing else is allowed
  // because this README documents one package; if it ever grows an example that
  // imports a third-party module, this list is where that gets admitted.
  const specifiers = new Set([...README.matchAll(/from "([^"]+)"/g)].map((m) => m[1]));
  assert.ok(specifiers.size > 0, "the scan no longer finds any import in the README");
  for (const specifier of specifiers) {
    assert.ok(
      specifier === PKG.name || specifier.startsWith("node:"),
      `the README imports "${specifier}", which is neither this package nor a builtin`,
    );
  }

  // And the error prefix carries the name too, because a message that names
  // `magical-js` now points at somebody else's package. Checked in both
  // directions: the scoped prefix is present, and the old unscoped one is gone
  // from every file that throws.
  //
  // The unscoped prefix is a *substring* of the scoped one, so the scoped form is
  // cut out before the search. Testing `"magical-js: " in source` directly would
  // pass on a file that had done nothing, and fail on a file that was entirely
  // correct — the assertion would be about the test rather than the code.
  const prefix = `${PKG.name}: `;
  for (const file of ["_levels.js", "_signatures.js", "_wasm.js"]) {
    const source = readFileSync(fileURLToPath(new URL(`../${file}`, import.meta.url)), "utf8");
    assert.ok(source.includes(prefix), `${file} no longer prefixes errors with the package name`);
    const unscoped = source.split(prefix).join("");
    assert.ok(
      !unscoped.includes("magical-js: "),
      `${file} still prefixes errors with the unscoped name, which is another package`,
    );
  }
});

test("the counts the README quotes are the ones the module reports", () => {
  // "114 formats", "114 rows", "114 names", "the 114-way union", "the format
  // count is pinned at 114".
  assert.equal(api.allKinds().length, 114);
  assert.equal(api.signatureTable().length, 114);
  assert.match(README, /\b114\b/, "the README no longer mentions 114 at all");

  // `neededBytes()  // 36870`.
  assert.equal(api.neededBytes(), 36870);
  assert.match(README, /neededBytes\(\)\s*\/\/\s*36870/);
});

test("the first example in the README is what it says it is", () => {
  // ```js
  // detectPath("photo.png");          // "Png"
  // describe("Png").signatures[0];    // Uint8Array [137, 80, 78, 71, 13, 10, 26, 10]
  // ```
  //
  // A real file on disk rather than a fixture, because `detectPath` is what the
  // example calls and a path it cannot read is not the claim.
  const dir = mkdtempSync(join(tmpdir(), "magical-js-readme-"));
  try {
    const path = join(dir, "photo.png");
    const bytes = Uint8Array.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
    writeFileSync(path, bytes);
    assert.equal(api.detectPath(path), "Png");
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }

  assert.deepEqual(
    [...api.describe("Png").signatures[0]],
    [137, 80, 78, 71, 13, 10, 26, 10],
  );
  assert.match(README, /detectPath\("photo\.png"\)/);
});

test("the sizes in the first paragraph are the right order of magnitude", () => {
  // "42 KB of WebAssembly (17 KB gzipped)". Exact bytes move with the compiler,
  // so the README is read for its claim and the module for its measurement, and
  // they have to agree to within a factor of two rather than to the byte.
  const raw = statSync(WASM).size;
  const gzipped = gzipSync(readFileSync(WASM)).length;
  const claimedRaw = claimedKilobytes(/(\d+) KB of WebAssembly/);
  const claimedGzipped = claimedKilobytes(/\((\d+) KB gzipped\)/);
  assert.ok(claimedRaw, "the README no longer states a raw size");
  assert.ok(claimedGzipped, "the README no longer states a gzipped size");
  assert.ok(
    Math.abs(raw - claimedRaw * 1024) < claimedRaw * 1024,
    `raw is ${raw} bytes, the README says ${claimedRaw} KB`,
  );
  assert.ok(
    Math.abs(gzipped - claimedGzipped * 1024) < claimedGzipped * 1024,
    `gzipped is ${gzipped} bytes, the README says ${claimedGzipped} KB`,
  );
  // And the two claims are about different things, which is worth keeping true:
  // gzip must actually be much smaller, or the README is quoting a number that
  // means nothing. Measured, it is 2.5x; a factor of two is the floor that
  // distinguishes "gzipped" from "re-stored with a different extension".
  assert.ok(gzipped < raw / 2, `gzip saved only ${gzipped}/${raw}`);
});

test("every format name the README puts in a result position is a real one", () => {
  // The class of error the `Heif` incident was: a plausible name, written where
  // the module answers with one, that the crate does not have. The scan is
  // deliberately narrow — a *double-quoted* capitalised word, which is the shape
  // that mistake took (`// "Heif"`). A wider scan over every backticked
  // capitalised word finds `TypeError`, `Uint8Array`, `readFileSync` and every
  // other name in the file, and then the allowlist has to grow every time anyone
  // mentions an API, which is a test that gets deleted instead of fixed.
  //
  // The handful of ordinary words in double quotes are listed rather than
  // pattern-matched around, because four is fewer than the number of ways to say
  // "not a format name" and this list is readable. One-character strings are not
  // scanned at all: `extern "C"` and `opt-level = "s"` are not format names and
  // no format is called `C`.
  const notFormats = new Set([
    "Install", "abort", "module", "type", // "panic = \"abort\"", "type": "module", headings
  ]);
  const known = new Set(api.allKinds());
  const claimed = new Set();
  for (const match of README.matchAll(/"([A-Z][A-Za-z0-9]+)"/g)) {
    if (notFormats.has(match[1])) continue;
    claimed.add(match[1]);
  }

  // `"Jpg"` is the README's *negative* example — `if (found === "Jpg")` is meant
  // not to compile — so it is a real format that these particular rules did not
  // declare. Checking it against `allKinds()` is the right check, and it is worth
  // saying why the example uses a real name: a made-up one would be rejected by
  // the compiler for a reason that has nothing to do with the union, which is the
  // point the example is making.
  for (const name of claimed) {
    assert.ok(
      known.has(name),
      `the README puts "${name}" where a format name goes, and it is not one of ` +
        `the ${known.size} formats`,
    );
  }
  // Sanity, in both directions. A regex that stopped matching would make this
  // test pass forever, and so would an allowlist that grew to everything.
  assert.ok(claimed.has("Png"), "the scan no longer finds the example that is there");
  assert.ok(claimed.has("WEBP"), "the scan no longer finds the predicate format");
  assert.ok(claimed.size < 20, `${claimed.size} matches: the scan is too wide now`);
});

test("the levels the README says are present are the ones that are", () => {
  // "| 1 | ... | Yes |" through "| 5 | ... | No |". The claim is that levels 1
  // and 2 work and 3, 4 and 5 do not, and the reason for the last three is
  // structural — a level 3 or 4 predicate needs a wasm *import* — rather than an
  // omission, so the table is not going to change quietly.
  for (const present of ["detectBytes", "detectPath", "matchTypes", "matchAllTypes"]) {
    assert.equal(typeof api[present], "function", `${present} is missing but the README lists it`);
  }
  // And the README's own table says so, in both directions: two Yes and three No.
  assert.match(README, /\| 1 \|.*\| Yes /);
  assert.match(README, /\| 2 \|.*\| Yes /);
  const noRows = [...README.matchAll(/^\| [345] \|.*\| \*\*No\*\* \|$/gm)];
  assert.equal(noRows.length, 3, "the table no longer says 3, 4 and 5 are absent");
  // An async entry point would be the one that breaks the package's central
  // claim, and the README says there is none.
  assert.equal(api.detectBytesAsync, undefined);
  for (const name of Object.keys(api)) {
    assert.ok(
      !/Async$/.test(name),
      `${name} is asynchronous, and the README says the whole package is synchronous`,
    );
  }
});

/** Read a `NN KB` claim out of the README, in kilobytes. */
function claimedKilobytes(pattern) {
  const match = README.match(pattern);
  return match ? Number(match[1]) : 0;
}

/** Quote a package name for a regular expression; the scope is not a pattern. */
function escape(text) {
  return text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
