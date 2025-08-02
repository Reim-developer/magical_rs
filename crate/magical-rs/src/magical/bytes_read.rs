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
use {
    std::fs::File,
    std::io,
    std::io::{BufReader, Read},
};
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
/// * `Ok(Vec<u8>)` - A vector containing the bytes read from file.
/// * `Err(io::Error)` - An I/O error if the file could not be append or read.
///
/// # Errors
/// This function returns an error in the following cases:
///
/// * The file does not exists or cannot be opened. (e.g., due to permission issues).
/// * There is error while reading from the file. (e.g., disk I/O error).
pub fn read_file_header(file_path: &str, max_bytes: usize) -> Result<Vec<u8>, io::Error> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);

    let max_bytes_read = max_bytes;
    let mut buffer = vec![0u8; max_bytes_read];

    let mut total_read: usize = 0;

    while total_read < max_bytes_read {
        match reader.read(&mut buffer[total_read..]) {
            Ok(0) => break, /* EOF */
            Ok(index) => total_read = total_read.saturating_add(index),
            Err(error) => return Err(error),
        }
    }

    Ok(buffer)
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
