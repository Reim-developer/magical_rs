use pyo3::pyfunction;
use std::env::var;

#[pyfunction]
#[must_use]
pub fn get_user_home() -> Option<String> {
    #[cfg(target_family = "unix")]
    {
        var("HOME").ok()
    }

    #[cfg(target_family = "windows")]
    {
        var("USERPROFILE").ok()
    }

    #[cfg(not(any(target_family = "unix", target_family = "windows")))]
    {
        None
    }
}
