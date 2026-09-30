// Detecting a buffer you already have, and the one option that changes answers.
//
// Run it with:
//
//   node examples/02_detect_bytes.mjs

import { DEFAULT_MAX_BYTES_READ, detectBytes, neededBytes, readLimits } from "../index.js";

/** Bytes of an ASCII string, without going through UTF-8 encoding. */
function ascii(text) {
  const out = new Uint8Array(text.length);
  for (let i = 0; i < text.length; i++) out[i] = text.charCodeAt(i) & 0xff;
  return out;
}

// `maxBytesRead` is the one option in this package that changes *answers* rather
// than just their cost, which is worth seeing rather than being told.
//
// It narrows the table to rules that read no further than the window. A format
// whose own `maxBytesRead` is larger cannot be returned — *even when the buffer is
// long enough to hold its signature* — because the crate will not compare bytes it
// has promised not to read. That is `match_with_max_read_rule` in the crate, and
// it is the crate's own behaviour rather than something this binding adds.
const iso = (() => {
  // ISO 9660 puts its magic at offset 32,765, which is the largest offset any rule
  // in the table uses. The buffer here is longer than that, so the *bytes are
  // present*; the only question is whether the rule is allowed to look at them.
  const out = new Uint8Array(40000);
  out.set(ascii("CD001"), 32769);
  return out;
})();

// The default is the crate's `DEFAULT_MAX_BYTES_READ`, 2,048 bytes. It is a
// deliberate floor rather than a maximum: 113 of the 114 rules declare 2,048 or
// more, so a *smaller* window drops nearly everything rather than a little.
console.log(`the default window is ${DEFAULT_MAX_BYTES_READ} bytes`);
console.log(`a window big enough for every format is ${neededBytes()} bytes`);

console.log();
console.log("an ISO 9660 header, 40,000 bytes long, whose magic sits at 32,769:");
console.log(`  detectBytes(iso)                     ${detectBytes(iso)}`);
console.log(`  detectBytes(iso, {maxBytesRead: 2048}) ${detectBytes(iso, { maxBytesRead: DEFAULT_MAX_BYTES_READ })}`);
console.log("  the second is null: the rule needs 36,870 bytes and the window is 2,048.");

// A buffer that is genuinely too short is a different thing, and it is worth
// telling apart. ISO 9660's magic really is at 36,769 here, so no window that
// reaches it can miss it; a buffer truncated *before* the magic is a file that no
// window can classify, which is not the same claim.
const truncated = iso.slice(0, 1000);
console.log();
console.log("the same header truncated to 1,000 bytes, before its magic:");
console.log(`  detectBytes(truncated) ${detectBytes(truncated)}`);

// What the read sizes actually are. They are the crate's `pub const` values, and
// they only make sense together — `isoOffsets` says where the ISO family is read
// and `isoMaxBytesRead` says how far — so they arrive as one object rather than
// five separate functions.
const limits = readLimits();
console.log();
console.log("the crate's own read limits:");
console.log(`  defaultMaxBytesRead  ${limits.defaultMaxBytesRead}`);
console.log(`  bytesRead            ${limits.bytesRead}`);
console.log(`  isoMaxBytesRead      ${limits.isoMaxBytesRead}`);
console.log(`  tarMaxBytesRead      ${limits.tarMaxBytesRead}`);
console.log(
  `  isoOffsets           [${limits.isoOffsets.join(", ")}]` +
    `  <- the largest is 36,865, and CD001 is five bytes, which is where 36,870 comes from`,
);

// A buffer big enough to reach the module's own data is the case that breaks a
// naive loader, and it is worth doing here rather than only in a test: the encoded
// detection table sits near the top of the module's initial memory, so a write at
// offset 0 of a large buffer is fine but a *zero-length* view cached at import is
// not. `detectBytes` grows memory and re-derives its view on every call, which is
// why this works and why the next call works too.
const big = new Uint8Array(4 * 1024 * 1024);
big.set(Uint8Array.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]));
console.log();
console.log("a 4 MB buffer, past the point where the module's memory has to grow:");
console.log(`  detectBytes(big)   ${detectBytes(big)}`);
console.log(`  detectBytes(again) ${detectBytes(big)}   <- the previous call did not break this one`);
