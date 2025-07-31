use crate::mime_types::bytes_read::{DEFAULT_MAX_BYTES_READ, read_file_header};
use crate::mime_types::file_mime::{
    BITMAP_MIME_TYPE, BZIP_MIME_TYPE, CLASS_MIME_TYPE, DEB_MIME_TYPE, GZIP_MIME_TYPE,
    ICO_MIME_TYPE, ISO_MIME_TYPE, JPG_MIME_TYPE, MP3_MIME_TYPE, MS_DOS_MIME_TYPE,
    PKG_ZIP_MIME_TYPE, PNG_MIME_TYPE, RMP_MIME_TYPE, SQLITE_MIME_TYPE, TAR_MIME_TYPE,
    UNKNOWN_MIME_TYPE, WASM_MIME_TYPE, XML_MIME_TYPE,
};
use crate::mime_types::signatures::SIGNATURE_KIND;
use pyo3::exceptions::PyIOError;
use pyo3::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[pyclass]
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
    Unknown,
}

fn with_bytes_read() -> usize {
    SIGNATURE_KIND
        .iter()
        .map(|magic| magic.max_bytes_read)
        .max()
        .unwrap_or(DEFAULT_MAX_BYTES_READ)
}

#[pyfunction]
/// Get file magics bytes
///
/// # Errors
/// Will be convert to `Python Exception`
pub fn get_magic_bytes(file_path: &str) -> PyResult<Vec<u8>> {
    let max_bytes = with_bytes_read();

    let bytes = read_file_header(file_path, max_bytes).map_err(|error| {
        PyIOError::new_err(format!("Could not read: {file_path} with error: {error}"))
    })?;

    Ok(bytes)
}

fn with_magic(bytes: &[u8]) -> FileKind {
    SIGNATURE_KIND
        .iter()
        .find(|magic| {
            magic.offsets.iter().any(|&offset| {
                magic.signatures.iter().any(|&signature| {
                    let offset_end = offset + signature.len();

                    bytes.len() >= offset_end && &bytes[offset..offset_end] == signature
                })
            })
        })
        .map_or(FileKind::Unknown, |magic| magic.kind)
}

#[pymethods]
impl FileKind {
    #[staticmethod]
    #[must_use]
    pub fn match_types(bytes: &[u8]) -> Self {
        with_magic(bytes)
    }

    #[must_use]
    pub fn get_mime_type(&self) -> String {
        match self {
            Self::Png => PNG_MIME_TYPE.to_string(),
            Self::Jpg => JPG_MIME_TYPE.to_string(),
            Self::Gzip => GZIP_MIME_TYPE.to_string(),
            Self::Bzip => BZIP_MIME_TYPE.to_string(),
            Self::PkgZip => PKG_ZIP_MIME_TYPE.to_string(),
            Self::Tar => TAR_MIME_TYPE.to_string(),
            Self::MSDOS => MS_DOS_MIME_TYPE.to_string(),
            Self::Bitmap => BITMAP_MIME_TYPE.to_string(),
            Self::Class => CLASS_MIME_TYPE.to_string(),
            Self::MP3 => MP3_MIME_TYPE.to_string(),
            Self::ISO => ISO_MIME_TYPE.to_string(),
            Self::RPM => RMP_MIME_TYPE.to_string(),
            Self::SQLite => SQLITE_MIME_TYPE.to_string(),
            Self::XML => XML_MIME_TYPE.to_string(),
            Self::ICO => ICO_MIME_TYPE.to_string(),
            Self::WASM => WASM_MIME_TYPE.to_string(),
            Self::Deb => DEB_MIME_TYPE.to_string(),
            Self::Unknown => UNKNOWN_MIME_TYPE.to_string(),
        }
    }
}

#[test]
fn test_magic() {
    let bytes = read_file_header("tests/1.png", DEFAULT_MAX_BYTES_READ).unwrap();

    assert_eq!(with_magic(&bytes), FileKind::Png);
}
