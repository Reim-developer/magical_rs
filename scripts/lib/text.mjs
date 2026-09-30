// Reading and writing text files in this repository, without a dependency.
//
// It exists because this repository is checked out on Windows with
// `core.autocrlf = true` and on Linux without it, so the same tracked file is
// CRLF in one working tree and LF in another. A generator that hardcodes "\n"
// therefore rewrites every file it touches on a Windows checkout, and a
// generator that hardcodes "\r\n" fails CI's "generated files are up to date"
// check, which runs on Linux. Git does not care -- it normalizes on commit --
// so neither choice is *wrong*. What is wrong is a generator that reports it
// wrote four files when it changed nothing, because the next person stops
// believing the output.
//
// So: read the file, notice what it is using, and give it back the same. A file
// that does not exist yet gets "\n", which is what the index holds and therefore
// what every generator run on a clean Linux checkout produces.

import { readFileSync, writeFileSync } from "node:fs";

/** The line ending `text` mostly uses, or "\n" for a file with no line at all. */
export function detectEol(text) {
  let crlf = 0;
  let lf = 0;
  for (let at = 0; at < text.length; at += 1) {
    if (text[at] !== "\n") continue;
    if (at > 0 && text[at - 1] === "\r") crlf += 1;
    else lf += 1;
  }
  if (crlf > lf) return "\r\n";
  return "\n";
}

/**
 * A file's lines, with no terminators and no trailing empty line.
 *
 * `\r?\n` rather than `split("\n")` because a CRLF file read as UTF-8 keeps the
 * `\r` on every line, and it then has to be remembered at every comparison in
 * every script. Stripping it here means a regex like `/^\*\*(.+)\*\*$/` matches
 * on both platforms, which is what makes a parser that reads this repository's
 * own files correct on both.
 */
export function lines(text) {
  const split = text.split(/\r?\n/);
  if (split.at(-1) === "") split.pop();
  return split;
}

/** A file's lines, but `null` when it does not exist. */
export function readLines(path) {
  const text = readTextFile(path);
  return text === null ? null : lines(text);
}

/** A file's text, or `null` when it does not exist. */
export function readTextFile(path) {
  try {
    return readFileSync(path, "utf8");
  } catch (error) {
    // ENOENT is the only absence this module treats as normal. A permission
    // error is a real problem and is reported as one, because silently
    // answering "the file is not there" is how a typo becomes an empty table.
    if (error.code === "ENOENT") return null;
    throw error;
  }
}

/** `text` with every line ending replaced by `eol` and exactly one final newline. */
export function withEol(text, eol) {
  const body = text.replace(/\r?\n/g, "\n").replace(/\n+$/, "");
  return body.split("\n").join(eol) + eol;
}

/**
 * Writes `text` if it differs from what is already there, and says which.
 *
 * Returns "created", "wrote" or "unchanged". The comparison is against the file
 * re-read with the ending the new text uses, so a CRLF file offered LF content
 * that is otherwise identical counts as unchanged rather than as a rewrite.
 */
export function writeIfChanged(path, text, eol = detectEol(readTextFile(path) ?? "")) {
  const current = readTextFile(path);
  if (current === null) {
    writeFileSync(path, withEol(text, eol), "utf8");
    return "created";
  }
  if (withEol(current, eol) === withEol(text, eol)) return "unchanged";
  writeFileSync(path, withEol(text, eol), "utf8");
  return "wrote";
}
