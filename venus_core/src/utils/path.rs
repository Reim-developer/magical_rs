use pyo3::prelude::*;
use pyo3::{PyResult, exceptions::PyIOError, pyclass, pyfunction};
use std::os::unix::fs::MetadataExt;
use std::{
    env,
    fs::File,
    io::Write,
    path::{self, Path},
};
use std::{fs, io};

#[pyfunction]
#[must_use]
pub fn get_user_home() -> Option<String> {
    env::home_dir().map(|user_home| user_home.to_string_lossy().to_string())
}

fn write_to_file(file_path: &str, content: &str) -> PyResult<()> {
    let file_temp = Path::new(file_path);

    let mut file = File::create_new(file_temp).map_err(|error| {
        PyIOError::new_err(format!(
            "Cannot create file: {file_path} with error: {error}"
        ))
    })?;

    file.write_all(content.as_bytes()).map_err(|error| {
        PyIOError::new_err(format!(
            "Cannot write file: {file_path} with error: {error}"
        ))
    })?;

    Ok(())
}

fn read_file_to_string(file_path: &str) -> Result<String, io::Error> {
    let content = fs::read_to_string(file_path)?;

    Ok(content)
}

fn get_file_size(file_path: &str) -> Result<u64, io::Error> {
    let metadata = fs::metadata(file_path)?;

    Ok(metadata.size())
}

#[pyclass]
pub struct FilePath {
    pub file_path: String,
}

#[pymethods]
impl FilePath {
    #[staticmethod]
    #[must_use]
    pub fn new_with_path(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }

    /// # Errors
    /// Will be convert to Python Exception
    pub fn write_to_temp_file(&self, content: &str) -> PyResult<()> {
        let file_temp = Path::new(&self.file_path);

        if !file_temp.exists() {
            write_to_file(&self.file_path, content)?;
        }

        if file_temp.exists() {
            fs::remove_file(file_temp).map_err(|error| {
                PyIOError::new_err(format!(
                    "Could not remove file {} with error: {error}",
                    &self.file_path
                ))
            })?;

            write_to_file(&self.file_path, content)?;
        }

        Ok(())
    }

    #[must_use]
    pub fn get_abs_path(&self) -> Option<String> {
        path::absolute(&self.file_path).map_or(None, |file_str| {
            Some(file_str.to_string_lossy().to_string())
        })
    }

    /// Read the file, then return `String`
    ///
    /// # Errors
    /// Will be convert to Python Exception.
    pub fn read_file(&self) -> PyResult<String> {
        let content_str = read_file_to_string(&self.file_path).map_err(|error| {
            PyIOError::new_err(format!(
                "Could not read the file: {}, with error: {error}",
                &self.file_path
            ))
        })?;

        Ok(content_str)
    }

    #[must_use]
    pub fn get_file_name(&self) -> Option<String> {
        Path::new(&self.file_path)
            .file_stem()
            .map(|file_name| file_name.to_string_lossy().to_string())
    }

    /// Get file size
    ///
    /// # Errors
    /// Will be convert to `Python` exception.
    pub fn get_file_size(&self) -> PyResult<u64> {
        let size = get_file_size(&self.file_path).map_err(|error| {
            PyIOError::new_err(format!(
                "Could not get size of {} with error: {error}",
                &self.file_path
            ))
        })?;

        Ok(size)
    }

    #[must_use]
    pub fn is_file_exists(&self) -> bool {
        Path::new(&self.file_path).exists()
    }
}

#[test]
fn test_read_file() {
    let file = "Cargo.toml";

    assert!(!read_file_to_string(file).unwrap().is_empty());
    assert!(read_file_to_string(file).is_ok());
}
