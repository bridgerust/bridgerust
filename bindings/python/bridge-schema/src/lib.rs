use bridge_schema::Validator as RustValidator;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyclass]
struct Validator {
    inner: RustValidator,
}

#[pymethods]
impl Validator {
    #[new]
    fn new(schema: &str) -> PyResult<Self> {
        let inner = RustValidator::new(schema).map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(Self { inner })
    }
}
