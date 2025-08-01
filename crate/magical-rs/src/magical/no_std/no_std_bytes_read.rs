use crate::magical::no_std::nostd_signature::SIGNATURE_KIND;

/// Default bytes to read of file.
pub const DEFAULT_MAX_BYTES_READ: usize = 2048;
/// Default offset of file.
pub const DEFAULT_OFFSET: usize = 0;
/// ISO file offset.
pub const ISO_OFFSETS: &[usize] = &[32769, 34817, 36865];
/// TAR file offset.
pub const TAR_OFFSETS: &[usize] = &[257];
/// ISO file max bytes to read.
pub const ISO_MAX_BYTES_READ: usize = no_std_max_bytes(ISO_OFFSETS, b"CD001");
/// TAR file max bytes to read.
pub const TAR_MAX_BYTES_READ: usize = no_std_max_bytes(TAR_OFFSETS, b"ustar");

/// Returns the maximum number of bytes needed to read the file headers for all known signature types.
///
/// This value is the largest `max_bytes_read` among all entries in `SIGNATURE_KIND`.
/// It ensures that even file formats with signatures at high offsets (e.g., ISO, TAR) can be detected.
///
/// # Why This Matters
///
/// Some file formats (e.g., ISO, RPM, TAR) have magic bytes at **large offsets** (e.g., 32KB+).
/// If you read fewer bytes than required, those formats will not be detected.
///
/// Always use this function to determine the read size.
///
/// Never assume `2048` or `4096` is enough.
///
/// # Returns
///
/// * `usize` - The number of bytes that must be read from the start of a file
///   to ensure all signature checks can succeed.
///
/// # Panics
///
/// This function **does not panic**, even if `SIGNATURE_KIND` is empty.
/// It returns `DEFAULT_MAX_BYTES_READ` as a safe fallback.
///
/// # `no_std` Compatibility
///
/// This function is designed to be used in `no_std` environments:
/// - It does not use `std`, `Vec`, `alloc`, or any heap allocation.
/// - It only depends on `SIGNATURE_KIND` and `DEFAULT_MAX_BYTES_READ`.
/// - It is `const`-friendly and can be evaluated at compile time in many cases.
///
/// The `no_std_` prefix emphasizes that this function is part of the `no_std`-safe API surface,
/// and can be used in embedded systems, kernels, or any environment without `std`.
///
/// # Examples
///
/// ```rust
/// use magical_rs::magical::no_std::no_std_bytes_read::no_std_with_bytes_read;
///
/// let buffer_size = no_std_with_bytes_read();
/// assert!(buffer_size >= 1024);
/// ```
#[must_use]
#[inline]
pub fn no_std_with_bytes_read() -> usize {
    SIGNATURE_KIND
        .iter()
        .map(|magic| magic.max_bytes_read)
        .max()
        .unwrap_or(DEFAULT_MAX_BYTES_READ)
}

/// Calculates the minimum number of bytes that must be read to validate a signature at given offsets.
///
/// This is used to determine the required buffer size for file type detection.
/// The result is `max(offsets) + signature.len()`.
///
/// # Parameters
/// - `offsets`: List of possible offsets where the signature may appear.
/// - `signature`: The byte sequence to match.
///
/// # Returns
/// The minimum number of bytes to read from the start of a file to ensure the signature
/// can be checked at the furthest offset.
///
/// # Example
/// ```rust
/// use magical_rs::magical::no_std::no_std_bytes_read::no_std_max_bytes;
///
/// const OFFSETS: &[usize] = &[0, 257];
/// const SIG: &[u8] = b"ustar";
/// assert_eq!(no_std_max_bytes(OFFSETS, SIG), 257 + 5); // 262
/// ```
///
/// # `const` Context
/// This function is `const fn`, so it can be evaluated at compile time.
#[must_use]
#[inline]
pub const fn no_std_max_bytes(offsets: &[usize], signature: &[u8]) -> usize {
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
