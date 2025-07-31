use crate::mime_types::file_mime::{
    BITMAP_MIME_TYPE, BZIP_MIME_TYPE, GZIP_MIME_TYPE, MS_DOS_MIME_TYPE, PKG_ZIP_MIME_TYPE,
    PNG_MIME_TYPE, TAR_MIME_TYPE, UNKNOWN_MIME_TYPE,
};
use pyo3::exceptions::PyIOError;
use pyo3::prelude::*;
use std::fs::File;
use std::io::{self, BufReader, Read};

#[derive(Clone, Copy, Debug)]
#[pyclass]
pub enum FileKind {
    Png,
    Bitmap,
    Gzip,
    Bzip,
    PkgZip,
    Tar,
    MsDos,
    Unknown,
}

struct Magic {
    signature: &'static [u8],
    offset: usize,
    kind: FileKind,
}

const MAX_BYTES_READ: usize = 2048;

const PNG_SIGNATURE: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
const GZIP_SIGNATURE: &[u8] = &[0x1F, 0x8B];
const BZIP_SIGNATURE: &[u8] = &[0x42, 0x5A];
const PKG_ZIP_SIGNATURE: &[u8] = &[0x50, 0x4B, 0x03, 0x04];
const BITMAP_SIGNATURE: &[u8] = &[0x42, 0x4D];
const TAR_SIGNATURE: &[u8] = &[0x75, 0x73, 0x74, 0x61, 0x72];
const MS_DOS_SIGNATURE: &[u8] = &[0x4D, 0x5A];

static SIGNATURE_KIND: &[Magic] = &[
    Magic {
        signature: PNG_SIGNATURE,
        offset: 0,
        kind: FileKind::Png,
    },
    Magic {
        signature: GZIP_SIGNATURE,
        offset: 0,
        kind: FileKind::Gzip,
    },
    Magic {
        signature: BZIP_SIGNATURE,
        offset: 0,
        kind: FileKind::Bzip,
    },
    Magic {
        signature: PKG_ZIP_SIGNATURE,
        offset: 0,
        kind: FileKind::PkgZip,
    },
    Magic {
        signature: BITMAP_SIGNATURE,
        offset: 0,
        kind: FileKind::Bitmap,
    },
    Magic {
        signature: MS_DOS_SIGNATURE,
        offset: 0,
        kind: FileKind::MsDos,
    },
    Magic {
        signature: TAR_SIGNATURE,
        offset: 257,
        kind: FileKind::Tar,
    },
];

/// Read the file header with limit caption as `2048 KB`.
///
/// # Errors
/// `File::open` failed.
/// `Buffer Reader` failed.
fn read_file_header(file_path: &str) -> Result<Vec<u8>, io::Error> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let mut buffer = vec![0u8; MAX_BYTES_READ];

    let _ = reader.read(&mut buffer)?;

    Ok(buffer)
}

#[pyfunction]
/// Get file magics bytes
///
/// # Errors
/// Will be convert to `Python Exception`
pub fn get_magic_bytes(file_path: &str) -> PyResult<Vec<u8>> {
    let bytes = read_file_header(file_path).map_err(|error| {
        PyIOError::new_err(format!("Could not read: {file_path} with error: {error}"))
    })?;

    Ok(bytes)
}

#[pymethods]
impl FileKind {
    #[staticmethod]
    #[must_use]
    pub fn match_types(bytes: &[u8]) -> Self {
        SIGNATURE_KIND
            .iter()
            .find(|magic| {
                let offset_start = magic.offset;

                bytes.len() >= offset_start + magic.signature.len()
                    && &bytes[offset_start..offset_start + magic.signature.len()] == magic.signature
            })
            .map_or(Self::Unknown, |magic| magic.kind)
    }

    #[must_use]
    pub fn get_mime_type(&self) -> String {
        match self {
            Self::Png => PNG_MIME_TYPE.to_string(),
            Self::Gzip => GZIP_MIME_TYPE.to_string(),
            Self::Bzip => BZIP_MIME_TYPE.to_string(),
            Self::PkgZip => PKG_ZIP_MIME_TYPE.to_string(),
            Self::Tar => TAR_MIME_TYPE.to_string(),
            Self::MsDos => MS_DOS_MIME_TYPE.to_string(),
            Self::Bitmap => BITMAP_MIME_TYPE.to_string(),
            Self::Unknown => UNKNOWN_MIME_TYPE.to_string(),
        }
    }
}
