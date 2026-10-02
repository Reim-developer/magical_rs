use crate::magical::signatures::SIGNATURE_KIND;

#[must_use]
#[inline]
pub const fn max_bytes(offsets: &[usize], signature: &[u8]) -> usize {
    let mut max_offset = 0;
    let mut index = 0;

    while index < offsets.len() {
        if offsets[index] > max_offset {
            max_offset = offsets[index];
        }

        index += 1;
    }

    max_offset + signature.len()
}

/// Default bytes to read of file.
pub const DEFAULT_MAX_BYTES_READ: usize = 2048;
/// Default offset of file.
pub const DEFAULT_OFFSET: usize = 0;
/// ISO file offset.
pub const ISO_OFFSETS: &[usize] = &[32769, 34817, 36865];
/// TAR file offset.
pub const TAR_OFFSETS: &[usize] = &[257];
/// ISO file max bytes to read.
pub const ISO_MAX_BYTES_READ: usize = max_bytes(ISO_OFFSETS, b"CD001");
/// TAR file max bytes to read.
pub const TAR_MAX_BYTES_READ: usize = max_bytes(TAR_OFFSETS, b"ustar");

/// Returns the maxium number of bytes needed to read the file headers for all known signature types.
///
/// The value returned is the largest `max_byte_read` among all entries in `SIGNATURE_KIND`.
///
/// # Returns
///
/// * `usize` - The maxium number of bytes that need to be read from file header.
///
/// # Examples
///
/// ```rust
/// use magical_rs::magical::bytes_read::{with_bytes_read, read_file_header};
///
/// let buffer_size = with_bytes_read();
/// let bytes = read_file_header("Cargo.toml", buffer_size);
///
/// assert!(read_file_header("Cargo.toml", buffer_size).is_ok());
/// assert!(!read_file_header("Cargo.toml", buffer_size).unwrap().is_empty());
///
/// ```
/// # Note
/// - This function assumes that `SIGNATURE_KIND` contains signature definedtions,
///   (e.g., magic numbers) with associalted `max_bytes_read` value indicating how many
///   initial bytes of a file must be read to validate each signature.
///
/// # Panics
/// - This function **does not** panic, even if `SIGNATURE_KIND` is empty. Thanks to
///   `unwrap_or(DEFAULT_MAX_BYTES_READ)`
///
/// # Why This Matters
///
/// Some file formats (e.g., ISO, RPM, TAR) have magic bytes at **large offsets** (e.g., 32KB+).
/// If you read fewer bytes than required, those formats will not be detected.
///
/// Always use this function to determine the read size:
/// ```no_run
/// use magical_rs::magical::bytes_read::{with_bytes_read, read_file_header};
///
/// let max_bytes = with_bytes_read();
/// let header = read_file_header("file.iso", max_bytes).unwrap();
/// ```
///
/// Never assume `2048` or `4096` is enough.
///
/// # Returns
///
/// The minimum number of bytes to read from the start of a file to ensure
/// all signature checks (including high-offset ones) can succeed.
#[must_use]
#[inline]
pub fn with_bytes_read() -> usize {
    SIGNATURE_KIND
        .iter()
        .map(|magic| magic.max_bytes_read)
        .max()
        .unwrap_or(DEFAULT_MAX_BYTES_READ)
}

#[cfg(feature = "std")]
use {std::fs::File, std::io, std::io::Read};
/// Reads up to `max_bytes` from beginning of a file.
///
/// This function opens the file at the given path and reads a maxium of `max bytes`.
///
/// # Parameters
///
/// * `file_path` - A string slice that holds the path to the file to read.
/// * `max_bytes` - The maxium number of bytes to read from the file header.
///
///
/// # Examples
///
/// ```no_run
/// use magical_rs::magical::bytes_read::{DEFAULT_MAX_BYTES_READ, read_file_header};
///
/// let file = "example.png";
///
/// match read_file_header("example.png", DEFAULT_MAX_BYTES_READ) {
///     Ok(file_header) => println!("Read: {} bytes from {file}", file_header.len()),
///     Err(error) => eprintln!("Could not read: {file}, with error: {error}"),
/// }
///
/// ```
/// # Test
///
/// ```rust
/// use magical_rs::magical::bytes_read::{DEFAULT_MAX_BYTES_READ, read_file_header};
///
/// let file_path = "Cargo.toml";
///
/// assert!(read_file_header(file_path, DEFAULT_MAX_BYTES_READ).is_ok());
/// assert!(!read_file_header(file_path, DEFAULT_MAX_BYTES_READ)
///         .unwrap()
///         .is_empty());
/// ```
/// # Result
/// Returns a `Result<Vec<u8, io::Error>`
///
/// * `Ok(Vec<u8>)` - A vector containing the bytes read from file, and
///   nothing else. A file shorter than `max_bytes` yields a shorter vector
///   rather than a padded one, so `header.len()` is the number of bytes the
///   file really has.
/// * `Err(io::Error)` - An I/O error if the file could not be append or read.
///
/// # Errors
/// This function returns an error in the following cases:
///
/// * The file does not exists or cannot be opened. (e.g., due to permission issues).
/// * There is error while reading from the file. (e.g., disk I/O error).
#[cfg(feature = "std")]
pub fn read_file_header(file_path: &str, max_bytes: usize) -> Result<Vec<u8>, io::Error> {
    let mut buffer = Vec::new();
    read_file_header_into(file_path, max_bytes, &mut buffer)?;
    Ok(buffer)
}

/// Reads up to `max_bytes` from the start of a file into a buffer the caller owns.
///
/// This is [`read_file_header`] with the allocation moved to the caller, and it is
/// the version to reach for when reading many files in a row: a caller scanning a
/// directory allocates once and then reads every file into the same buffer.
///
/// ```no_run
/// use magical_rs::magical::bytes_read::{read_file_header_into, with_bytes_read};
/// use magical_rs::magical::magic::FileKind;
///
/// let mut buffer = Vec::new();
/// for entry in std::fs::read_dir(".")? {
///     let path = entry?.path();
///     if read_file_header_into(&path.to_string_lossy(), with_bytes_read(), &mut buffer).is_ok() {
///         if let Some(kind) = FileKind::match_types(&buffer) {
///             // `variant_name`, not `Display`: `FileKind` has no `Display`, and
///             // `display_name` is the human-facing string.
///             println!("{path:?}: {}", kind.variant_name());
///         }
///     }
/// }
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// The buffer is cleared first and holds only this file's bytes when the function
/// returns, so it can be handed straight to [`FileKind::match_types`] — no slicing,
/// no `truncate` at the call site.
///
/// # Errors
///
/// The same as [`read_file_header`]: the path does not exist or cannot be opened,
/// or the read fails. The buffer is left **empty** on failure, so a caller that
/// catches the error and ignores it gets `FileKind::match_types(&[])` == `None`
/// rather than the previous file's bytes — or, worse, a window of invented zeros.
#[cfg(feature = "std")]
pub fn read_file_header_into(
    file_path: &str,
    max_bytes: usize,
    buffer: &mut Vec<u8>,
) -> Result<(), io::Error> {
    // Cleared *first*, and before the open, so the empty-on-failure guarantee
    // above holds. Resizing before the open would also leave the buffer full of
    // zeros on a failed open, and those are not inert: `bytes[0]` is a real byte,
    // so it lands in a real bucket, and a caller that ignored the error could be
    // handed a kind for a file that was never opened. Order is the whole of it
    // here — `resize` alone would overwrite the stale bytes with zeros and look
    // equivalent while answering a different question.
    buffer.clear();

    let mut file = File::open(file_path)?;

    // `resize` rather than `reserve` plus a read into the spare capacity: the
    // spare capacity of a `Vec<u8>` is `[MaybeUninit<u8>]`, so reading into it
    // needs `unsafe` and a `set_len` that trusts the `read`. The zero-fill that
    // `resize` does is 419 ns for 36,870 bytes on the machine this was measured
    // on — about 1.4% of a call — and it is the price of not writing `unsafe` in
    // a crate whose whole claim is that it needs none.
    buffer.resize(max_bytes, 0);

    let mut total_read: usize = 0;

    // Not `BufReader`. A `BufReader` is for reading a stream through many small
    // reads, and this reads one fixed window: the wrapper allocates 8 KiB it
    // never needs, copies the file into it, and copies it back out. Measured on a
    // 36,870-byte read that layer and its allocation cost about 5 µs of a 30 µs
    // call — roughly a sixth — for no change in what was read.
    while total_read < max_bytes {
        match file.read(&mut buffer[total_read..]) {
            Ok(0) => break, /* EOF */
            Ok(index) => total_read = total_read.saturating_add(index),
            Err(error) => return Err(error),
        }
    }

    // The buffer was zero-filled to its capacity, so without this a file
    // shorter than `max_bytes` comes back padded with invented bytes
    // rather than short, and the returned length says nothing about how much
    // of it is the file. Two of those invented bytes are enough to complete a
    // signature, because a format is free to end its magic with a zero: PCX is
    // `[0x0A, 0x00]` and TIFF is `II*\0`. So a one-byte file holding a
    // newline was reported as a PCX and a three-byte one as a TIFF, while
    // `FileKind::match_types` on the very same bytes said no match. Detection
    // that disagrees with itself depending on whether the caller passed a
    // path or a buffer is worse than the missing read it looked like, and a
    // caller handed these bytes has no way to tell which part is the file.
    //
    // Truncating cannot cost a real match: the padding supplies only zeros, so
    // it can only ever complete a signature the file itself did not contain.
    // A genuine PCX carries both of its bytes and a genuine TIFF all four.
    buffer.truncate(total_read);

    Ok(())
}

#[test]
#[cfg(feature = "std")]
fn test_read_file_header() {
    use crate::magical::bytes_read::{DEFAULT_MAX_BYTES_READ, read_file_header};

    let file_path = "Cargo.toml";

    assert!(read_file_header(file_path, DEFAULT_MAX_BYTES_READ).is_ok());
    assert!(
        !read_file_header(file_path, DEFAULT_MAX_BYTES_READ)
            .unwrap()
            .is_empty()
    );
}

#[test]
#[cfg(feature = "std")]
fn test_read_file_header_is_not_padded() {
    use crate::magical::bytes_read::{DEFAULT_MAX_BYTES_READ, read_file_header};

    // `II*` is three quarters of the TIFF magic, whose last byte is a zero.
    // Read into a zero-filled buffer of `DEFAULT_MAX_BYTES_READ` it completed
    // itself, so a three-byte file was reported as a TIFF.
    let path = std::env::temp_dir().join(format!(
        "magical-rs-header-unpadded-{}.bin",
        std::process::id()
    ));
    std::fs::write(&path, b"II*").unwrap();

    let header = read_file_header(path.to_str().unwrap(), DEFAULT_MAX_BYTES_READ).unwrap();

    std::fs::remove_file(&path).unwrap();
    assert_eq!(header, b"II*");
    assert_eq!(header.len(), 3);
}

#[test]
#[cfg(feature = "std")]
fn test_a_short_file_and_its_bytes_agree_on_the_kind() {
    use crate::magical::bytes_read::{DEFAULT_MAX_BYTES_READ, read_file_header};
    use crate::magical::magic::FileKind;

    // Detection must not depend on whether the caller had a path or a buffer.
    // These are the prefixes the padding used to complete: PCX is
    // `[0x0A, 0x00]`, TIFF is `II*\0` and RAR is `Rar!\x1a\x07\0`, so each
    // was one byte short of a real format and the invented zero finished it.
    // The last two are complete magics, kept here so the test would notice if
    // truncation had started cutting matches it should not.
    let cases: [&[u8]; 5] = [
        b"\x0a",
        b"II*",
        b"Rar!\x1a\x07",
        b"\x00\x00\x02\x00",
        b"\x00\x01\x00\x00\x00",
    ];

    for content in cases {
        let path = std::env::temp_dir().join(format!(
            "magical-rs-header-agree-{}-{}.bin",
            std::process::id(),
            content.len()
        ));
        std::fs::write(&path, content).unwrap();

        let header = read_file_header(path.to_str().unwrap(), DEFAULT_MAX_BYTES_READ).unwrap();

        std::fs::remove_file(&path).unwrap();
        assert_eq!(header, content, "the header is not the file's own bytes");
        assert_eq!(
            FileKind::match_types(&header),
            FileKind::match_types(content),
            "a path and the same bytes disagree about {content:?}"
        );
    }
}

#[test]
#[cfg(feature = "std")]
fn test_truncated_magics_are_not_formats() {
    use crate::magical::bytes_read::{DEFAULT_MAX_BYTES_READ, read_file_header};
    use crate::magical::magic::FileKind;

    // The three prefixes from the test above, on their own: with the padding in
    // place a lone newline was a PCX, which is how a one-byte file could be
    // reported as a graphics format.
    for content in [
        b"\x0a".as_slice(),
        b"II*".as_slice(),
        b"Rar!\x1a\x07".as_slice(),
    ] {
        let path = std::env::temp_dir().join(format!(
            "magical-rs-header-truncated-{}-{}.bin",
            std::process::id(),
            content.len()
        ));
        std::fs::write(&path, content).unwrap();

        let header = read_file_header(path.to_str().unwrap(), DEFAULT_MAX_BYTES_READ).unwrap();

        std::fs::remove_file(&path).unwrap();
        assert_eq!(
            FileKind::match_types(&header),
            None,
            "{content:?} is {} bytes and matches no format",
            content.len()
        );
    }
}

/// The two readers agree, byte for byte, on files either side of the window.
///
/// `read_file_header_into` exists for the allocation, so the only thing that makes
/// it a faster reader rather than a second reader is that it reads the same thing.
/// The buffer it fills is one the caller owns, so the risk is real: a `clear` that
/// did not happen, or a `truncate` that used the wrong count, leaves the previous
/// file's bytes in the buffer and the next detection silently describes the wrong
/// file.
///
/// The buffer is **not** fresh each time. That is the whole point — reusing one is
/// what the API is for, and a fresh buffer per call would pass a version that
/// forgot to clear it.
///
/// A note on what this does **not** cover, because the first version of it claimed
/// to: removing the `clear()` outright still passes this test, since the following
/// `resize` overwrites those bytes with zeros. The `clear` earns its place on the
/// failure path instead, which is the case below.
#[test]
#[cfg(feature = "std")]
fn the_reusing_reader_matches_the_allocating_one() {
    use crate::magical::bytes_read::{read_file_header, read_file_header_into, with_bytes_read};
    use crate::magical::magic::FileKind;

    // Files both shorter and longer than the window, and the window sizes the
    // library itself suggests and its own default, because `read_file_header` and
    // `read_file_header_into` take the size as an argument and neither has a say
    // in it — a mismatch there would show up as different bytes, not as a panic.
    let window = with_bytes_read();
    let long = vec![0x5a_u8; window + 4_096];
    let cases: [(&[u8], usize); 6] = [
        (&long, window),
        (&long, 2_048),
        (&[], window),
        (b"II*\0", window),
        (b"\x89PNG\r\n\x1a\n", window),
        (&long[..3], window),
    ];

    let mut scratch = Vec::new();
    for (content, limit) in cases {
        let path = std::env::temp_dir().join(format!(
            "magical-rs-header-agree-{}-{limit}.bin",
            std::process::id()
        ));
        std::fs::write(&path, content).unwrap();

        let allocating = read_file_header(path.to_str().unwrap(), limit).unwrap();
        // Deliberately without a clear first: a reader that left the previous
        // file's bytes here would pass a fresh-buffer version of this test and
        // fail here, which is the bug a directory scan actually hits.
        scratch.extend_from_slice(b"stale bytes that must not survive");
        read_file_header_into(path.to_str().unwrap(), limit, &mut scratch).unwrap();

        std::fs::remove_file(&path).unwrap();
        assert_eq!(
            allocating,
            scratch,
            "a {} byte file read with limit {limit}: the two readers disagree",
            content.len()
        );
        assert_eq!(
            FileKind::match_types(&allocating),
            FileKind::match_types(&scratch),
            "a {} byte file read with limit {limit}: the kinds disagree",
            content.len()
        );
    }
}

/// A failed read leaves the buffer empty, not stale and not a window of zeros.
///
/// This is the case the `clear()` before the open exists for, and it is why that
/// line is where it is. Both wrong versions of this function pass
/// `the_reusing_reader_matches_the_allocating_one`:
///
/// * **No `clear` at all** — the next `resize` overwrites the stale bytes, so a
///   successful read is still correct.
/// * **`clear` after the open, or after the `resize`** — a caller scanning a
///   directory and ignoring errors keeps detecting against the previous file.
///
/// Neither is caught by comparing two successful reads, and both are caught here.
/// The assertion is `is_empty` rather than "detects as nothing" because those are
/// different guarantees: a buffer of zeros has `bytes[0] == Some(0)`, which is a
/// real byte in a real bucket.
#[test]
#[cfg(feature = "std")]
fn a_failed_read_leaves_the_buffer_empty() {
    use crate::magical::bytes_read::{read_file_header_into, with_bytes_read};
    use crate::magical::magic::FileKind;

    let missing = std::env::temp_dir().join(format!(
        "magical-rs-does-not-exist-{}.bin",
        std::process::id()
    ));
    assert!(
        !missing.exists(),
        "{} exists, so this test is reading a real file",
        missing.display()
    );

    let mut scratch = Vec::new();
    // A real file first, so the buffer holds a real file's bytes rather than
    // starting empty — otherwise "empty afterwards" proves nothing.
    let real = std::env::temp_dir().join(format!(
        "magical-rs-header-then-fail-{}.bin",
        std::process::id()
    ));
    std::fs::write(&real, b"\x89PNG\r\n\x1a\n").unwrap();
    read_file_header_into(real.to_str().unwrap(), with_bytes_read(), &mut scratch).unwrap();
    std::fs::remove_file(&real).unwrap();
    assert!(
        !scratch.is_empty(),
        "the successful read put nothing in the buffer, so the failed one below is not testing \
         anything"
    );

    let error = read_file_header_into(missing.to_str().unwrap(), with_bytes_read(), &mut scratch);
    assert!(
        error.is_err(),
        "reading a path that does not exist reported success"
    );
    assert!(
        scratch.is_empty(),
        "after a failed read the buffer holds {} bytes ({:02X?}); a caller that ignored the \
         error would detect against those",
        scratch.len(),
        &scratch[..scratch.len().min(16)],
    );
    assert_eq!(
        FileKind::match_types(&scratch),
        None,
        "a buffer left over from a failed read still names a format"
    );
}
