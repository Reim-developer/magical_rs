//! Shebang detection.
//!
//! The bare two-byte `#!` is not a usable signature on its own: it claims
//! every file starting with those two characters, including the `#!AMR` audio
//! header. Requiring a path separator in the first line tells the two apart.

// First 128 bytes is far more than enough to hold a shebang line.
const SCAN_LIMIT: usize = 128;

/// Matches a `#!` line that names an interpreter by path.
///
/// A real shebang must contain a `/` in the interpreter path, because the
/// kernel resolves the remainder of the line as a path. That single condition
/// separates `#!/bin/sh` and `#!/usr/bin/env python` from `#!AMR`, which is
/// an audio container header with no path in it.
///
/// The trade-off is that a relative interpreter name such as `#!python` is
/// **not** detected. Such a script is non-portable anyway, and the previous
/// two-byte rule already missed `#!AMR` entirely, so this narrows the false
/// positives without losing any realistic script.
///
/// # Examples
///
/// ```rust
/// use magical_rs::magical::ext_fn::shebang::is_shebang;
///
/// assert!(is_shebang(b"#!/bin/sh\n"));
/// assert!(is_shebang(b"#!/usr/bin/env python3\n"));
/// assert!(is_shebang(b"#!/usr/bin/perl -w\n"));
///
/// // Not a shebang: no path separator, so this is the AMR audio header.
/// assert!(!is_shebang(b"#!AMR\n"));
/// // Not a shebang: relative interpreter name, see the note above.
/// assert!(!is_shebang(b"#!python\n"));
/// // Not a shebang: too short to hold an interpreter path.
/// assert!(!is_shebang(b"#!\n"));
/// assert!(!is_shebang(b"#!"));
/// // Not a shebang: a `#` comment, not `#!`.
/// assert!(!is_shebang(b"# comment\n"));
/// ```
#[must_use]
pub fn is_shebang(bytes: &[u8]) -> bool {
    if bytes.len() < 3 || bytes[0] != b'#' || bytes[1] != b'!' {
        return false;
    }

    let limit = bytes.len().min(SCAN_LIMIT);

    for &byte in &bytes[2..limit] {
        if byte == b'/' {
            return true;
        }
        if byte == b'\n' {
            // End of the shebang line arrived with no path separator.
            return false;
        }
    }

    // No separator within the scanned window. A longer header is not a
    // shebang we can vouch for.
    false
}
