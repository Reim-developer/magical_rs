use pyo3::{PyResult, exceptions::PyRuntimeError, pyfunction};
use webbrowser::open;

#[pyfunction]
/// # Errors
/// Will be convert to `Python RuntimeError`
pub fn open_browser(url: &str) -> PyResult<()> {
    open(url).map_err(|error| {
        PyRuntimeError::new_err(format!("Could not open your browser, with error: {error}"))
    })?;

    Ok(())
}
