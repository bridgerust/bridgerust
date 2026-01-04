//! End-to-end test library for BridgeRust
//!
//! This library exports functions and structs that are tested from both Python and Node.js

use bridgerust::export;

// Simple function exports
#[export]
pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

#[export]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[export]
pub fn multiply(a: f64, b: f64) -> f64 {
    a * b
}

#[export]
pub fn is_even(n: i32) -> bool {
    n % 2 == 0
}

// Function with Option
#[export]
pub fn divide(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 {
        None
    } else {
        Some(a / b)
    }
}

// Function with Vec
#[export]
pub fn sum_numbers(numbers: Vec<i32>) -> i32 {
    numbers.iter().sum()
}

// Local Error Type to satisfy Orphan Rules
pub struct TestError(#[allow(dead_code)] String);

impl From<String> for TestError {
    fn from(s: String) -> Self {
        TestError(s)
    }
}

#[cfg(feature = "python")]
impl From<TestError> for bridgerust::pyo3::PyErr {
    fn from(err: TestError) -> Self {
        bridgerust::pyo3::exceptions::PyRuntimeError::new_err(err.0)
    }
}

#[cfg(feature = "nodejs")]
impl From<TestError> for bridgerust::napi::Error {
    fn from(err: TestError) -> Self {
        bridgerust::napi::Error::from_reason(err.0)
    }
}

#[cfg(feature = "nodejs")]
impl From<TestError> for bridgerust::napi::bindgen_prelude::JsError {
    fn from(err: TestError) -> Self {
        bridgerust::napi::bindgen_prelude::JsError::from(bridgerust::napi::Error::from(err))
    }
}

// Function with Result
#[export]
pub fn might_fail(value: i32) -> Result<i32, TestError> {
    if value < 0 {
        Err(TestError("Value must be positive".to_string()))
    } else {
        Ok(value * 2)
    }
}

// Struct exports
#[export]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[export]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

// Shared Implementation (Pure Rust)
impl Point {
    pub fn new_impl(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance_impl(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

impl Rectangle {
    pub fn new_impl(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    pub fn area_impl(&self) -> f64 {
        self.width * self.height
    }
}

// Python-specific methods
#[cfg(feature = "python")]
use bridgerust::pyo3::prelude::*;

#[cfg(feature = "python")]
#[pymethods]
impl Point {
    #[new]
    fn new_py(x: f64, y: f64) -> Self {
        Self::new_impl(x, y)
    }

    #[pyo3(name = "distance")]
    fn distance_py(&self) -> f64 {
        self.distance_impl()
    }

    fn __repr__(&self) -> String {
        format!("Point({}, {})", self.x, self.y)
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl Rectangle {
    #[new]
    fn new_py(width: f64, height: f64) -> Self {
        Self::new_impl(width, height)
    }

    #[pyo3(name = "area")]
    fn area_py(&self) -> f64 {
        self.area_impl()
    }

    fn __repr__(&self) -> String {
        format!("Rectangle({}x{})", self.width, self.height)
    }
}

// Node.js-specific methods
#[cfg(feature = "nodejs")]
use bridgerust::napi_derive::napi;

#[cfg(feature = "nodejs")]
#[napi]
impl Point {
    #[napi(constructor)]
    pub fn new_js(x: f64, y: f64) -> Self {
        Self::new_impl(x, y)
    }

    #[napi(js_name = "distance")]
    pub fn distance_js(&self) -> f64 {
        self.distance_impl()
    }
}

#[cfg(feature = "nodejs")]
#[napi]
impl Rectangle {
    #[napi(constructor)]
    pub fn new_js(width: f64, height: f64) -> Self {
        Self::new_impl(width, height)
    }

    #[napi(js_name = "area")]
    pub fn area_js(&self) -> f64 {
        self.area_impl()
    }
}

// Python module definition
#[cfg(feature = "python")]
#[pymodule]
fn bridgerust_e2e_test(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(greet, m)?)?;
    m.add_function(wrap_pyfunction!(add, m)?)?;
    m.add_function(wrap_pyfunction!(multiply, m)?)?;
    m.add_function(wrap_pyfunction!(is_even, m)?)?;
    m.add_function(wrap_pyfunction!(divide, m)?)?;
    m.add_function(wrap_pyfunction!(sum_numbers, m)?)?;
    m.add_function(wrap_pyfunction!(might_fail, m)?)?;
    m.add_class::<Point>()?;
    m.add_class::<Rectangle>()?;
    Ok(())
}
