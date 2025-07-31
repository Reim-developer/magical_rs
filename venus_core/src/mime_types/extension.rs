use pyo3::prelude::*;

#[pyclass]
pub enum FileExtKind {
    Png,
    Bitmap,
    Gzip,
    Bzip,
    PkgZip,
    Tar,
    MsDos,
    Unknown,
}

const PNG: &str = "png";
const BITMAP: &str = "bmp";
const GZIP: &str = "gz";
const PKG_ZIP: &str = "zip";
const TAR: &str = "tar";
const MSDOS: &str = "exe";

#[pymethods]
impl FileExtKind {
    #[must_use]
    #[staticmethod]
    pub fn with_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            PNG => Self::Png,
            BITMAP => Self::Bitmap,
            GZIP => Self::Gzip,
            PKG_ZIP => Self::PkgZip,
            TAR => Self::Tar,
            MSDOS => Self::MsDos,
            _ => Self::Unknown,
        }
    }
}
