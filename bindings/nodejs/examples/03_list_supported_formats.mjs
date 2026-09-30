// What is detectable, and what each answer carries.
//
// `allKinds()` is the authoritative list of what this build recognises — it is
// generated from the same table that generates the Rust enum and the Python enum,
// so it cannot be out of step with either.
//
// Run it with:
//
//   node examples/03_list_supported_formats.mjs

import { allKinds, describe, extension, isFileKind, mime } from "../index.js";

/** Formats grouped by top-level media type, most common first. */
function byMediaType() {
  const tally = new Map();
  for (const kind of allKinds()) {
    const type = mime(kind);
    if (type === null) continue;
    const top = type.split("/")[0];
    tally.set(top, (tally.get(top) ?? 0) + 1);
  }
  return [...tally].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]));
}

/** The formats whose `attribute` is `null`, sorted. */
function without(attribute) {
  const read = { mime, extension };
  return allKinds()
    .filter((kind) => read[attribute](kind) === null)
    .sort();
}

console.log(`${allKinds().length} formats are detectable by this build.`);

console.log();
console.log("By top-level media type:");
for (const [name, count] of byMediaType()) {
  console.log(`  ${name}: ${count}`);
}

const noMime = without("mime");
console.log();
console.log(
  `${noMime.length} formats have no registered media type, which is null rather than a guess:`,
);
console.log(`  ${noMime.join(", ")}`);
console.log(
  "  an invented type would be served to a browser and would look exactly like a right one.",
);

const noExtension = without("extension");
console.log();
console.log(`${noExtension.length} formats have no conventional extension:`);
console.log(`  ${noExtension.join(", ")}`);
console.log("  there is no `bin`: an extension is a convention, and inventing one invents a claim.");

// The identifier and the display name are the same format written two ways, and
// the difference matters the moment one is printed and the other is stored.
console.log();
console.log("A format has two names, and they answer different questions:");
console.log(`  the identifier:  ${"Png".padEnd(4)} -> a value, a key, a matchTypes answer`);
console.log(`  the display one: ${"PNG".padEnd(4)} -> what goes in a file listing`);
console.log();
console.log(`  isFileKind("Png")   ${isFileKind("Png")}`);
console.log(`  isFileKind("PNG")   ${isFileKind("PNG")}  <- a display name is not an identifier`);
console.log(`  isFileKind("png")   ${isFileKind("png")}`);

// `describe` is the other half of the same table: not what a format is called, but
// what the crate matches it on. Two entries are decided by a function rather than by
// bytes. Only one of the 114 carries no bytes at all, so the two counts are 113
console.log();
console.log("How the table matches, which is a different question from what it holds:");
const withBytes = allKinds().filter((kind) => describe(kind).signatures.length > 0);
const byPredicate = allKinds().filter((kind) => describe(kind).usesPredicate);
const noBytesAtAll = allKinds().filter((kind) => describe(kind).signatures.length === 0);
console.log(`  ${withBytes.length} rules compare bytes`);
console.log(`  ${byPredicate.length} call a host function: ${byPredicate.join(", ")}`);
console.log(`  ${noBytesAtAll.length} carries no bytes at all: ${noBytesAtAll.join(", ")}`);
console.log(
  "  ScriptExecute carries `#!` as a prefilter *and* a predicate, so it is in both",
);
console.log("  lists. This binding does not blank the signature of a predicate entry:");
console.log("  the bytes are there and hiding them would make this a worse description.");
