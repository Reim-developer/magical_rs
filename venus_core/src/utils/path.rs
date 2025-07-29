use pyo3::{PyResult, exceptions::PyIOError, pyfunction};
use std::fs;
use std::{
    env,
    fs::File,
    io::Write,
    path::{self, Path},
};

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

#[pyfunction]
/// # Errors
/// Will be convert to Python Exception
pub fn write_to_temp_file(file_path: &str, content: &str) -> PyResult<()> {
    let file_temp = Path::new(file_path);

    if !file_temp.exists() {
        write_to_file(file_path, content)?;
    }

    if file_temp.exists() {
        fs::remove_file(file_temp).map_err(|error| {
            PyIOError::new_err(format!(
                "Could not remove file {file_path} with error: {error}"
            ))
        })?;

        write_to_file(file_path, content)?;
    }

    Ok(())
}

#[pyfunction]
#[must_use]
pub fn get_abs_path(file_path: &str) -> Option<String> {
    path::absolute(file_path).map_or(None, |file_str| {
        Some(file_str.to_string_lossy().to_string())
    })
}
