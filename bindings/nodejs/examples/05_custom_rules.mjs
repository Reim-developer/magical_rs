// Rules you build at runtime, for formats this package has never heard of.
//
// Run it with:
//
//   node examples/05_custom_rules.mjs
//
// This is level 2 in the crate, not a reimplementation of it: a completed rule set
// is compiled once by the crate's own matcher and called through it, so the two
// cannot disagree about what matches.

import { matchAllTypes, matchTypes, matches, releaseRules } from "../index.js";

/** Bytes of an ASCII string, without going through UTF-8 encoding. */
function ascii(text) {
  const out = new Uint8Array(text.length);
  for (let i = 0; i < text.length; i++) out[i] = text.charCodeAt(i) & 0xff;
  return out;
}

const PNG = Uint8Array.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
const GIF = ascii("GIF89a");
const ZIP = Uint8Array.from([0x50, 0x4b, 0x03, 0x04]);

const tarred = (() => {
  const out = new Uint8Array(512);
  out.set(ascii("ustar"), 257);
  return out;
})();

// A rule is "any signature, at any offset". `kind` is whatever you want it to be:
// level 2 exists for formats this package has never heard of, and the answer comes
// back under the name you gave rather than under a `FileKind`.
const rules = [
  { kind: "Screenshot", signatures: [PNG], offsets: [0] },
  { kind: "AnimatedImage", signatures: [GIF], offsets: [0] },
  // Two rules that both match a PNG on purpose, so that "the first match" and
  // "every match" are visibly different questions rather than a distinction the
  // reader has to take on trust.
  { kind: "Image", signatures: [PNG, GIF], offsets: [0] },
  // A renamed format usually has more than one signature, and a file that moved
  // usually has more than one offset. Both arrays are allowed to be that long.
  { kind: "Archive", signatures: [ascii("PK"), ascii("ustar")], offsets: [0, 257] },
];

console.log("a rule set naming four formats this package does not know:");
for (const rule of rules) {
  console.log(
    `  ${rule.kind.padEnd(13)} ${rule.signatures.length} signature(s) at offset(s) ${rule.offsets.join(", ")}`,
  );
}

console.log();
console.log("matchTypes answers with the FIRST rule that matched, in declaration order:");
console.log(`  a PNG header       ${matchTypes(rules, PNG)}`);
console.log(`  a GIF header       ${matchTypes(rules, GIF)}`);
console.log(`  a ZIP container    ${matchTypes(rules, ZIP)}`);
console.log(`  a tar header       ${matchTypes(rules, tarred)}`);
console.log(`  something else     ${matchTypes(rules, ascii("not a format"))}`);

console.log();
console.log("matchAllTypes asks the same rules the same way, and returns every match:");
console.log(`  a PNG header       [${matchAllTypes(rules, PNG).join(", ")}]`);
console.log(`  a GIF header       [${matchAllTypes(rules, GIF).join(", ")}]`);
console.log("  order is the rules array's, so putting a general rule first hides the specific one.");

// A rule that can never match is rejected, by the types and by the loader. A rule
// with no signature, or no offset to look at one, is indistinguishable from a
// correct one at the call site, because both of them simply do not fire.
console.log();
console.log("a half-built rule set is refused rather than kept:");
for (const broken of [
  ["1 signature, no offset", { kind: "Broken", signatures: [PNG], offsets: [] }],
  ["no signature, 1 offset", { kind: "Broken", signatures: [], offsets: [0] }],
]) {
  try {
    matchTypes([broken[1]], PNG);
  } catch (error) {
    console.log(`  ${broken[0]}`);
    console.log(`    ${error.constructor.name}: ${error.message}`);
  }
}
console.log("  both of them would compile, match nothing, and look correct.");

// The compiled copy is cached on the array you passed, so a rule set used in a loop
// compiles once and only once. `releaseRules` hands the handle back to the module;
// it does not free the memory, because the crate's `MagicCustom` needs a `&'static`
// reference to keep matching with. The boolean is what distinguishes "already
// released" from "never compiled", which `undefined` could not.
console.log();
console.log("the compiled handle, and giving it back:");
console.log(`  still matches:                ${matchTypes(rules, PNG)}`);
console.log(`  releaseRules(rules):          ${releaseRules(rules)}`);
console.log(`  releaseRules(rules) again:    ${releaseRules(rules)}   <- false, so "already" and "never" differ`);
console.log(`  matches again, recompiled:    ${matchTypes(rules, PNG)}`);

// The other question a rule set can be asked: one format's *own* rule, ignoring the
// rest of the table. `detectBytes` stops at the first table entry that matched, so a
// format whose signature is a prefix of another's is unreachable through it. Two
// pairs in the table are like this, measured rather than assumed, and both are
// listed in the crate's own tests.
const ktx2 = Uint8Array.from([0xab, 0x4b, 0x54, 0x58, 0x20, 0x32, 0x30, 0xbb, 0x0d, 0x0a, 0x1a, 0x0a]);
console.log();
console.log("`matches` asks about one format, ignoring the rest of the table:");
console.log(`  the same 12 bytes, as a Ktx2 header`);
console.log(`    matches("Ktx2", bytes)   ${matches("Ktx2", ktx2)}`);
console.log(`    matches("Ktx",  bytes)   ${matches("Ktx", ktx2)}`);
console.log(
  "  Ktx's magic is a prefix of Ktx2's, so both answers are true and they answer",
);
console.log(
  "  different questions. Nothing is broken: the table lists the longer signature",
);
console.log(
  "  first, so a real KTX2 file detects as Ktx2. This is the call for when you want",
);
console.log("  to ask the other question anyway. Qcow and Qcow2 are the second such pair.");
