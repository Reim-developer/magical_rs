use crate::magical::signatures::SIGNATURE_KIND;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileKind {
    Png,
    Bitmap,
    Gzip,
    Bzip,
    PkgZip,
    Tar,
    MSDOS,
    Jpg,
    Class,
    MP3,
    ISO,
    RPM,
    SQLite,
    XML,
    ICO,
    WASM,
    Deb,
    RAR,
    ScriptExecute,
    ELF,
    OGG,
    _8BPS,
    BLENDER,
    TrueTypeFont,
    OpenTypeFont,
    ModuleForEvenvironmentModules,
    WindowImagingFormat,
    Slob,
    SerializedJavaData,
    CreativeVoiceFile,
    AuAudioFileFormat,
    OpenGLIrisPerformer,
    NoodlesoftHazel,
    VBScriptEncoded,
    WEBP,
    AppleIconImage,
    GIF,
    JPEG2000,
    PDF,
    AppleDiskImage,
    Cabinet,
    MatroskaMediaContainer,
    RichTextFormat,
    PhotoCapTemplate,
    AceCompressed,
    FlashVideo,
}

impl FileKind {
    /// Attemps to determine the file type by matching the given bytes slice against known magic
    /// signatures.
    ///
    /// This function compares the input bytes with a list of predefined
    /// file signatures (magic numbers) at specified.
    /// It returns the most appropriate [`FileKind`] if a match is found.
    /// Otherwise, return [`None`] if no signature matches.
    ///
    /// # Parameters
    ///
    /// * `bytes` - A slice of bytes, typicaly the header (prefix) of a file, to be analyzed.
    ///
    /// # Returns
    ///
    /// `Self` (`FileKind`) - The detected file type. This is:
    /// - The `kind` of the first matching signature in `SIGNATURE_KIND`, or
    /// - [`None`] if no match is found.
    ///
    /// # Matching Logic
    ///
    /// For each signature entry in `SIGNATURE_KIND`:
    /// - Checks if the byte sequence (`signature`) appears at any of the specified `offsets`.
    /// - A valid match requires:
    ///   1. The input `bytes` is long enough to cover the range `[offset..offset + signature.len()]`.
    ///   2. The slice `&bytes[offset..offset + signature.len()]` exactly matches `signature`.
    ///
    /// The search is performed in the order of `SIGNATURE_KIND`, and the first successful match wins.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use magical_rs::magical::magic::FileKind;
    ///
    /// let png_header = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    /// let kind = FileKind::match_types(png_header).unwrap();
    ///
    /// assert_eq!(kind, FileKind::Png);
    /// ```
    ///
    /// ```rust
    /// use magical_rs::magical::magic::FileKind;
    ///
    /// let unknown_bytes = &[0x00, 0x01, 0x02, 0x03];
    /// let kind = FileKind::match_types(unknown_bytes);
    ///
    /// assert!(kind.is_none());
    /// ```
    ///
    /// ```rust
    /// use magical_rs::magical::{magic::FileKind, bytes_read::{read_file_header, DEFAULT_MAX_BYTES_READ}};
    /// let bytes = read_file_header("tests/1.png", DEFAULT_MAX_BYTES_READ).unwrap();
    ///
    /// assert_eq!(FileKind::match_types(&bytes).unwrap(), FileKind::Png);
    /// ```
    ///
    /// # Notes
    ///
    /// - Order matters: Since the function returns on the first match, the order of entries
    ///   in `SIGNATURE_KIND` can affect the result (e.g., in case of ambiguous or overlapping signatures).
    /// - Performance: The function short-circuits on the first match. Worst-case time complexity
    ///   is linear in the number of signatures and offsets.
    /// - Buffer size: Ensure that `bytes` is at least as large as required by [`with_bytes_read`],
    ///   otherwise some signatures may not be detectable.
    ///
    /// [`with_bytes_read`]: https://docs.rs/magical_rs/0.1.0/magical_rs/magical/bytes_read/fn.with_bytes_read.html
    ///
    /// # Safety
    ///
    /// This function performs safe slice comparisons and bounds checks (`bytes.len() >= offset_end`),
    /// so it will not panic or cause memory access violations even with small or empty input.
    #[must_use]
    #[inline]
    pub fn match_types(bytes: &[u8]) -> Option<Self> {
        SIGNATURE_KIND
            .iter()
            .find(|magic| magic.matches(bytes))
            .map(|magic| magic.kind)
    }

    /// Detects the file type by matching against built-in signatures, but only considers rules
    /// where `max_bytes_read` is less than or equal to `allowed_max_read`.
    ///
    /// This function is designed for environments where the amount of data read from a file
    /// is limited (e.g., embedded systems, streaming, or memory-constrained contexts).
    ///
    /// It ensures that only signature checks compatible with the provided read limit are performed,
    /// preventing false negatives due to insufficient data.
    ///
    /// # Parameters
    /// - `bytes`: The raw byte slice to analyze (e.g., file header).
    /// - `allowed_max_read`: The maximum number of bytes that were read from the file.
    ///   Rules requiring more bytes to validate will be **excluded** from the check.
    ///
    /// # Returns
    /// - `Self`: The detected `FileKind` if a matching rule is found and within the read limit.
    /// - [`None`]: If no rule matches or all matching rules require more bytes than allowed.
    ///
    /// # Why This Matters
    ///
    /// Some file formats (like ISO, TAR, or RPM) have magic bytes at **large offsets** (e.g., 32KB+).
    /// If you read only 1024 bytes, those formats **cannot be detected** — and this function
    /// respects that limitation by filtering them out.
    ///
    /// This avoids misleading results and ensures detection reliability based on actual input size.
    ///
    /// # Critical Constants
    ///
    /// The behavior of this function depends on built-in constants defined in `no_std_bytes_read.rs`:
    ///
    /// ```rust
    /// use magical_rs::magical::bytes_read::max_bytes;
    ///
    /// pub const DEFAULT_MAX_BYTES_READ: usize = 2048;
    /// pub const DEFAULT_OFFSET: usize = 0;
    /// pub const ISO_OFFSETS: &[usize] = &[32769, 34817, 36865];
    /// pub const TAR_OFFSETS: &[usize] = &[257];
    /// pub const ISO_MAX_BYTES_READ: usize = max_bytes(ISO_OFFSETS, b"CD001"); // ~32774
    /// pub const TAR_MAX_BYTES_READ: usize = max_bytes(TAR_OFFSETS, b"ustar"); // 262
    /// ```
    ///
    /// For example:
    /// - PNG, JPG, GIF, etc..: require `allowed_max_read >= 2048` to be considered.
    /// - TAR: requires `allowed_max_read >= 262`.
    /// - ISO: requires `allowed_max_read >= 32774`.
    ///
    /// If `allowed_max_read < 2048`, even common formats like PNG will be **excluded** from detection.
    ///
    /// # Example
    ///
    /// ```rust
    /// use magical_rs::magical::magic::FileKind;
    ///
    /// let png_bytes = [
    ///     0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
    ///     0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    /// ];
    ///
    /// let kind =  FileKind::match_with_max_read_rule(&png_bytes, 2048).unwrap();
    /// assert_eq!(kind, FileKind::Png);
    ///
    /// let kind = FileKind::match_with_max_read_rule(&png_bytes, 100);
    /// assert!(kind.is_none());
    /// ```
    ///
    /// # `no_std` Compatibility
    ///
    /// - Zero allocation
    /// - No dependency on `std`
    /// - `#[must_use]` and `#[inline]` for performance
    /// - Safe to use in embedded, kernel, or WASM environments
    #[must_use]
    #[inline]
    #[cfg(not(feature = "std"))]
    pub fn match_with_max_read_rule(bytes: &[u8], allowed_max_read: usize) -> Option<Self> {
        SIGNATURE_KIND
            .iter()
            .filter(|magic| magic.max_bytes_read <= allowed_max_read)
            .find(|magic| magic.matches(bytes))
            .map(|magic| magic.kind)
    }

    /// Detects the file type by matching against built-in signatures, without enforcing per-rule `max_bytes_read` limits.
    ///
    /// This function checks all known signatures against the provided byte slice,
    /// as long as the input `bytes` is at least `allowed_max_read` in length.
    /// It does **not** filter rules based on their individual `max_bytes_read` value.
    ///
    /// # Parameters
    /// - `bytes`: The raw byte slice to analyze (e.g., file header).
    /// - `allowed_max_read`: The number of bytes that were read from the file.
    ///   This is used to ensure the input is long enough for meaningful checks.
    ///
    /// # Returns
    /// - `Self`: The detected `FileKind` if a matching rule is found.
    /// - [`None`]: If the input is too short, or no signature matches.
    ///
    /// # Why This Matters
    ///
    /// Unlike `no_std_match_with_max_read_rule`, this function **does not exclude** any signature
    /// based on its `max_bytes_read`. Instead, it assumes the caller has already read enough data
    /// (at least `allowed_max_read` bytes), and attempts to match **all** possible formats.
    ///
    /// This is useful when:
    /// - You have read a sufficient amount of data (e.g., 2048+ bytes).
    /// - You want to maximize detection chances, even for formats with large offsets.
    /// - You are in a `no_std` context but control the read size.
    ///
    /// # Example
    ///
    /// ```rust
    /// use magical_rs::magical::magic::FileKind;
    ///
    /// let png_bytes = [
    ///     0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
    ///     0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    /// ];
    ///
    /// let kind = FileKind::match_with_custom_max_read(&png_bytes, 16).unwrap();
    /// assert_eq!(kind, FileKind::Png);
    ///
    /// let tiny = [0x89, 0x50];
    /// let wrong_kind = FileKind::match_with_custom_max_read(&tiny, 16);
    /// assert!(wrong_kind.is_none());
    /// ```
    ///
    /// # Note
    /// - This function only requires that `bytes.len() >= allowed_max_read`
    ///   to proceed with matching — it does not enforce `magic.max_bytes_read`.
    /// - All signature checks (including high-offset ones like ISO, TAR) will be attempted
    ///   as long as the input buffer is large enough.
    ///
    /// # `no_std` Compatibility
    /// - Zero allocation
    /// - No dependency on `std`
    /// - Safe to use in embedded, kernel, or WASM environments
    #[must_use]
    #[inline]
    #[cfg(not(feature = "std"))]
    pub fn match_with_custom_max_read(bytes: &[u8], allowed_max_read: usize) -> Option<Self> {
        if bytes.len() < allowed_max_read {
            return None;
        }

        SIGNATURE_KIND
            .iter()
            .find(|magic| magic.matches(bytes))
            .map(|magic| magic.kind)
    }
}
