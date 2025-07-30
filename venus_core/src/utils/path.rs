use pyo3::prelude::*;
use pyo3::{PyResult, exceptions::PyIOError, pyclass, pyfunction};
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
    pub fn is_file_exists(&self) -> bool {
        Path::new(&self.file_path).exists()
    }
}

#[test]
fn test_is_file_exists() {
    assert!(FilePath::new_with_path("Cargo.toml").is_file_exists());
}

#[test]
fn test_get_abs_path() {
    let abs_path = FilePath::new_with_path("Cargo.toml").get_abs_path();
    assert!(abs_path.is_some());
}

#[test]
fn test_get_user_home() {
    assert!(get_user_home().is_some());
}

#[test]
fn test_read_file() {
    let file_path = FilePath::new_with_path("Cargo.toml").read_file().unwrap();

    assert!(!read_file_to_string(&file_path).unwrap().is_empty());
    assert!(read_file_to_string(&file_path).is_ok());
}
