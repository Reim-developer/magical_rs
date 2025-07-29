#![deny(clippy::pedantic, clippy::all, clippy::nursery, clippy::perf)]

use pyo3::{
    Bound, PyResult, pymodule,
    types::{PyModule, PyModuleMethods},
    wrap_pyfunction,
};
use utils::logging::debug;
use utils::path::get_user_home;

pub mod utils {
    pub mod logging;
    pub mod path;
}

#[pymodule]
/// # Errors
/// Add function failed.
pub fn venus_core(module: &Bound<PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(debug, module)?)?;
    module.add_function(wrap_pyfunction!(get_user_home, module)?)?;
    Ok(())
}
