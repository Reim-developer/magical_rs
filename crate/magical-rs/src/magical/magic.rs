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
    Unknown,
}

impl FileKind {
    /// Attemps to determine the file type by matching the given bytes slice against known magic
    /// signatures.
    ///
    /// This function compares the input bytes with a list of predefined
    /// file signatures (magic numbers) at specified.
    /// It returns the most appropriate [`FileKind`] if a match is found.
    /// Otherwise, return [`FileKind::Unknown`] if no signature matches.
    ///
    /// # Parameters
    ///
    /// * `bytes` - A slice of bytes, typicaly the header (prefix) of a file, to be analyzed.
    ///
    /// # Returns
    ///
    /// * `Self` (`FileKind`) - The detected file type. This is:
    ///   - The `kind` of the **first** matching signature in `SIGNATURE_KIND`, or
    ///   - [`FileKind::Unknown`] if no match is found.
    ///
    /// # Matching Logic
    ///
    /// For each signature entry in `SIGNATURE_KIND`:
    /// - Checks if the byte sequence (`signature`) appears at any of the specified `offsets`.
    /// - A valid match requires:
    ///   1. The input `bytes` is long enough to cover the range `[offset..offset + signature.len()]`.
    ///   2. The slice `&bytes[offset..offset + signature.len()]` exactly matches `signature`.
    ///
    /// The search is performed in the order of `SIGNATURE_KIND`, and the **first successful match** wins.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use magical_rs::magical::magic::FileKind;
    ///
    /// let png_header = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    /// let kind = FileKind::match_types(png_header);
    /// assert_eq!(kind, FileKind::Png);
    /// ```
    ///
    /// ```rust
    /// use magical_rs::magical::magic::FileKind;
    ///
    /// let unknown_bytes = &[0x00, 0x01, 0x02, 0x03];
    /// let kind = FileKind::match_types(unknown_bytes);
    /// assert_eq!(kind, FileKind::Unknown);
    /// ```
    ///
    /// ```rust
    ///
    /// // With the specify file
    ///
    /// use magical_rs::magical::{magic::FileKind, bytes_read::{read_file_header, DEFAULT_MAX_BYTES_READ}};
    /// let bytes = read_file_header("tests/1.png", DEFAULT_MAX_BYTES_READ).unwrap();
    ///
    /// assert_eq!(FileKind::match_types(&bytes), FileKind::Png);
    ///
    /// ```
    ///
    /// # Notes
    ///
    /// - **Order matters**: Since the function returns on the first match, the order of entries
    ///   in `SIGNATURE_KIND` can affect the result (e.g., in case of ambiguous or overlapping signatures).
    /// - **Performance**: The function short-circuits on the first match. Worst-case time complexity
    ///   is linear in the number of signatures and offsets.
    /// - **Buffer size**: Ensure that `bytes` is at least as large as required by [`with_bytes_read()`],
    ///   otherwise some signatures may not be detectable.
    ///
    /// # Safety
    ///
    /// This function performs safe slice comparisons and bounds checks (`bytes.len() >= offset_end`),
    /// so it will not panic or cause memory access violations even with small or empty input.
    #[must_use]
    pub fn match_types(bytes: &[u8]) -> Self {
        SIGNATURE_KIND
            .iter()
            .find(|magic| magic.matches(bytes))
            .map_or(Self::Unknown, |magic| magic.kind)
    }
}
