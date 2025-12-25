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

    fn validate(&self, instance: &str) -> PyResult<()> {
        self.inner
            .validate(instance)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    fn is_valid(&self, instance: &str) -> bool {
        self.inner.is_valid(instance)
    }
}

#[pymodule]
fn bridge_schema(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Validator>()?;
    Ok(())
}
