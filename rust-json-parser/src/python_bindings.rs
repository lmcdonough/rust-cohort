// src/python_bindings.rs

// pyo3::prelude::* gives us Python, PyResult, IntoPyObject, etc.
use crate::parse_json as rust_parse_json;
use pyo3::prelude::*;

// PyDict = Python's dict type. PyList = Python's list type
use pyo3::types::{PyDict, PyList};

// Pull in your existing types from lib.rs
use crate::{JsonError, JsonValue};

// Tell PyO3 how to convert any JsonValue into a native Python object
impl<'py> IntoPyObject<'py> for JsonValue {
    // What Python type are we creating? - PyAny means any Python object
    type Target = PyAny;

    // How is the Python object returned? - Bound <'py> means it's tied to the GIL lifetime
    type Output = Bound<'py, Self::Target>;

    // What can go wrong? - PyErr is a Python exception represented in Rust
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            // Null -> Python's None singleton
            JsonValue::Null => Ok(py.None().into_bound(py)),

            // Boolean -> Python bool; .to_owned() converts Borrowed -> Bound, .into_any() upcasts to PyAny
            JsonValue::Boolean(b) => Ok(b.into_pyobject(py)?.to_owned().into_any()),

            // Number -> Python float (42 becomes 42.0); same Borrowed->Bound->PyAny pattern
            JsonValue::Number(n) => Ok(n.into_pyobject(py)?.to_owned().into_any()),

            // String -> Python str; no .to_owned() needed, String returns an owned Bound directly
            JsonValue::String(s) => Ok(s.into_pyobject(py)?.into_any()),

            // Array -> Python list; build it by recursively converting each element
            JsonValue::Array(arr) => {
                // Start with an empty Python []
                let py_list = PyList::empty(py);

                for item in arr {
                    // Each item calls int_pyobject() too - nested arrays/objects recurse automatically
                    py_list.append(item.into_pyobject(py)?)?;
                }

                // Upcast Bound<PyList> to Bound<PyAny> to match our return type
                Ok(py_list.into_any())
            }

            // Object -> Python dict; build it recursively converting each value
            JsonValue::Object(obj) => {
                // Start with an empty Python {}
                let py_dict = PyDict::new(py);

                for (key, value) in obj {
                    // key (String) auto-converts to Python str; value recurses into_pyobject()
                    py_dict.set_item(key, value.into_pyobject(py)?)?;
                }

                // Upcast Bound<PyDict> to Bound<PyAny> to match our return type
                Ok(py_dict.into_any())
            }
        }
    }
}

// Import Python's ValueError exception type
use pyo3::exceptions::PyValueError;

// Teach the ? operator how to turn any JsonError into a Python exception
impl From<JsonError> for PyErr {
    fn from(err: JsonError) -> PyErr {
        match err {
            // Parsing errors -> Python ValueError (same as Python's json.loads())
            JsonError::UnexpectedToken {
                expected,
                found,
                position,
            } => PyValueError::new_err(format!(
                "JSON parse error at position {}: expected {}, found '{}'",
                position, expected, found
            )),

            // Premature end of input -> ValueError with position info
            JsonError::UnexpectedEndOfInput { expected, position } => {
                PyValueError::new_err(format!(
                    "Unexpected end of input at position {}: expected {}",
                    position, expected
                ))
            }

            // Invalid number literal -> ValueError
            JsonError::InvalidNumber { value, position } => PyValueError::new_err(format!(
                "Invalid number '{}' at position {}",
                value, position
            )),

            // Invalid escape sequence -> ValueError
            JsonError::InvalidEscape { char, position } => PyValueError::new_err(format!(
                "Invalid escape '\\{}' at position {}",
                char, position
            )),

            // Invalid unicode escape -> ValueError
            JsonError::InvalidUnicode { sequence, position } => PyValueError::new_err(format!(
                "Invalid unicode escape '{}' at position {}",
                sequence, position
            )),
        }
    }
}

// Mark this function as callable from Python
#[pyfunction]
// py = GIL token (PyO3 injects this automatically), input = the JSON string
fn parse_json<'py>(py: Python<'py>, input: &str) -> PyResult<Bound<'py, PyAny>> {
    // Call the aliased Rust parser function
    let result = rust_parse_json(input)?;

    // Convert JsonValue -> Python Object using IntoPyObject impl
    result.into_pyobject(py)
}

// Register the module so Python can import it: `import rust_json_parser`
#[pymodule]
fn rust_json_parser(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Expose parse_json() to Python
    m.add_function(wrap_pyfunction!(parse_json, m)?)?;
    Ok(())
}
