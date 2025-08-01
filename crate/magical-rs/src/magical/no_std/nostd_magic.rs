use crate::magical::no_std::nostd_signature::SIGNATURE_KIND;

#[derive(Clone, Debug, Copy, PartialEq, Eq)]

pub enum NoStdFileKind {
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
    Unknown,
}

impl NoStdFileKind {
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
    /// - `Self::Unknown`: If no rule matches or all matching rules require more bytes than allowed.
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
    /// use magical_rs::magical::no_std::no_std_bytes_read::no_std_max_bytes;
    /// pub const DEFAULT_MAX_BYTES_READ: usize = 2048;
    /// pub const DEFAULT_OFFSET: usize = 0;
    /// pub const ISO_OFFSETS: &[usize] = &[32769, 34817, 36865];
    /// pub const TAR_OFFSETS: &[usize] = &[257];
    ///
    /// pub const ISO_MAX_BYTES_READ: usize = no_std_max_bytes(ISO_OFFSETS, b"CD001"); // ~32774
    /// pub const TAR_MAX_BYTES_READ: usize = no_std_max_bytes(TAR_OFFSETS, b"ustar"); // 262
    /// ```
    ///
    /// For example:
    /// - **PNG, JPG, GIF, etc.**: require `allowed_max_read >= 2048` to be considered.
    /// - **TAR**: requires `allowed_max_read >= 262`.
    /// - **ISO**: requires `allowed_max_read >= 32774`.
    ///
    /// If `allowed_max_read < 2048`, even common formats like PNG will be **excluded** from detection.
    ///
    /// # Example
    ///
    /// ```rust
    /// use magical_rs::magical::no_std::nostd_magic::NoStdFileKind;
    ///
    /// let png_bytes = [
    ///     0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
    ///     0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    ///     // ... đủ 2048 byte hoặc dùng `Vec::resize` trong test
    /// ];
    ///
    /// // Must be >= DEFAULT_MAX_BYTES_READ (2048)
    /// let kind = NoStdFileKind::no_std_match_with_max_read_rule(&png_bytes, 2048);
    /// assert_eq!(kind, NoStdFileKind::Png);
    ///
    /// // Will return `Unknown` because 100 < 2048 → PNG rule filtered out
    /// let kind = NoStdFileKind::no_std_match_with_max_read_rule(&png_bytes, 100);
    /// assert_eq!(kind, NoStdFileKind::Unknown);
    /// ```
    ///
    /// # `no_std` Compatibility
    ///
    /// - Zero allocation
    /// - No dependency on `std`
    /// - `#[must_use]` and `#[inline]` for performance
    /// - Safe to use in embedded, kernel, or WASM environments
    ///
    /// The `no_std_` prefix signals that this is part of the safe, controlled API surface
    /// for constrained environments.
    #[must_use]
    #[inline]
    // #[cfg(feature = "no_std")]
    pub fn no_std_match_with_max_read_rule(bytes: &[u8], allowed_max_read: usize) -> Self {
        SIGNATURE_KIND
            .iter()
            .filter(|magic| magic.max_bytes_read <= allowed_max_read)
            .find(|magic| magic.no_std_matches(bytes))
            .map_or(Self::Unknown, |magic| magic.kind)
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
    /// - `Self::Unknown`: If the input is too short, or no signature matches.
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
    /// use magical_rs::magical::no_std::nostd_magic::NoStdFileKind;
    ///
    /// let png_bytes = [
    ///     0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
    ///     0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    ///     // ... (ensure total length >= allowed_max_read)
    /// ];
    ///
    /// // Will attempt to match PNG, even if other rules require more bytes
    /// let kind = NoStdFileKind::no_std_match_with_custom_max_read(&png_bytes, 16);
    /// assert_eq!(kind, NoStdFileKind::Png);
    ///
    /// // Returns `Unknown` if input is too short
    /// let tiny = [0x89, 0x50];
    /// let kind = NoStdFileKind::no_std_match_with_custom_max_read(&tiny, 16);
    /// assert_eq!(kind, NoStdFileKind::Unknown);
    /// ```
    ///
    /// # Note
    /// - This function only requires that `bytes.len() >= allowed_max_read`
    ///   to proceed with matching — it does **not** enforce `magic.max_bytes_read`.
    /// - All signature checks (including high-offset ones like ISO, TAR) will be attempted
    ///   as long as the input buffer is large enough.
    ///
    /// # `no_std` Compatibility
    /// - Zero allocation
    /// - No dependency on `std`
    /// - Safe to use in embedded, kernel, or WASM environments
    ///
    /// The `no_std_` prefix indicates this function is part of the `no_std`-safe API,
    /// designed for constrained environments where control over I/O is explicit.
    #[must_use]
    #[inline]
    // #[cfg(feature = "no_std")]
    pub fn no_std_match_with_custom_max_read(bytes: &[u8], allowed_max_read: usize) -> Self {
        if bytes.len() < allowed_max_read {
            return Self::Unknown;
        }

        SIGNATURE_KIND
            .iter()
            .find(|magic| magic.no_std_matches(bytes))
            .map_or(Self::Unknown, |magic| magic.kind)
    }
}
