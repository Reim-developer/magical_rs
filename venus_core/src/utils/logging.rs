use std::io::{Write, stdout};

use pyo3::{
    Bound, PyErr, PyResult,
    exceptions::PyValueError,
    pyfunction,
    types::{PyAnyMethods, PyStringMethods, PyTuple, PyTupleMethods},
};

/// Format string.
///
/// # Result:
/// Ok if not have any errors.
///
/// # Errors
/// The errors will be convert to `PythonException`.
///
fn fmt_string(fmt_str: &str, args: &mut Vec<String>) -> Result<String, PyErr> {
    let mut output = fmt_str.to_string();

    for arg_str in args.drain(..) {
        if let Some(position) = output.find("{}") {
            output.replace_range(position..position + 2, &arg_str);
        } else {
            return Err(PyValueError::new_err("Too many arguments for placeholder"));
        }

        if output.contains("{}") {
            return Err(PyValueError::new_err(
                "Not enough arguments for placeholders.",
            ));
        }
    }
    Ok(output)
}

fn stdout_str(text: &str) -> PyResult<()> {
    writeln!(stdout().lock(), "{text}")?;
    stdout().lock().flush()?;

    Ok(())
}

#[pyfunction]
#[pyo3[signature = (fmt_str, *args)]]
/// Show debug message to your stdout.
///
/// # Result:
/// Ok if not have any errors.
///
/// # Errors
/// The errors will be convert to `PythonException`.
///
pub fn debug(fmt_str: &str, args: &Bound<PyTuple>) -> Result<(), PyErr> {
    let mut args_string = Vec::new();

    for arg in args.iter() {
        let Ok(py_str) = arg.str() else {
            return Err(PyValueError::new_err(
                "Could not convert 'str' to 'PyString'",
            ));
        };

        args_string.push(py_str.to_string_lossy().to_string());
    }

    let Ok(fmt_text) = fmt_string(fmt_str, &mut args_string) else {
        return Err(PyValueError::new_err("Could not format string."));
    };

    if let Err(error) = stdout_str(&fmt_text) {
        return Err(PyValueError::new_err(format!(
            "Could not stdout to terminal with error: {error}."
        )));
    }

    Ok(())
}
