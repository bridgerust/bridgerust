//! Comprehensive BridgeRust Example
//!
//! This example demonstrates all major features of BridgeRust:
//! - Function exports with various types
//! - Struct exports with methods
//! - Error handling
//! - Complex data structures

use bridgerust::{error, export};
use std::fmt::{Display, Formatter};

// ============================================================================
// Error Types
// ============================================================================

#[error]
#[derive(Debug, Clone)]
pub enum MathError {
    DivisionByZero,
    NegativeNumber,
    Overflow,
}

impl Display for MathError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MathError::DivisionByZero => write!(f, "Division by zero"),
            MathError::NegativeNumber => write!(f, "Negative number not allowed"),
            MathError::Overflow => write!(f, "Integer overflow"),
        }
    }
}

impl std::error::Error for MathError {}

#[cfg(feature = "nodejs")]
impl From<MathError> for napi::Error {
    fn from(err: MathError) -> Self {
        napi::Error::from_reason(err.to_string())
    }
}

// Added JsError impl
#[cfg(feature = "nodejs")]
impl From<MathError> for napi::bindgen_prelude::JsError {
    fn from(err: MathError) -> Self {
        napi::bindgen_prelude::JsError::from(napi::Error::from(err))
    }
}

#[cfg(feature = "python")]
impl From<MathError> for pyo3::PyErr {
    fn from(err: MathError) -> Self {
        pyo3::exceptions::PyRuntimeError::new_err(err.to_string())
    }
}

// ============================================================================
// Basic Functions
// ============================================================================

#[export]
pub fn greet(name: String) -> String {
    format!("Hello, {}! Welcome to BridgeRust.", name)
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

// ============================================================================
// Functions with Option
// ============================================================================

#[export]
pub fn safe_divide(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 {
        None
    } else {
        Some(a / b)
    }
}

#[export]
pub fn find_first_even(numbers: Vec<i32>) -> Option<i32> {
    numbers.into_iter().find(|&n| n % 2 == 0)
}

// ============================================================================
// Functions with Vec
// ============================================================================

#[export]
pub fn sum_numbers(numbers: Vec<i32>) -> i32 {
    numbers.iter().sum()
}

#[export]
pub fn filter_positive(numbers: Vec<i32>) -> Vec<i32> {
    numbers.into_iter().filter(|&n| n > 0).collect()
}

#[export]
pub fn double_all(numbers: Vec<i32>) -> Vec<i32> {
    numbers.into_iter().map(|n| n * 2).collect()
}

// ============================================================================
// Functions with Result
// ============================================================================

#[export]
pub fn safe_sqrt(n: f64) -> Result<f64, MathError> {
    if n < 0.0 {
        Err(MathError::NegativeNumber)
    } else {
        Ok(n.sqrt())
    }
}

#[export]
pub fn safe_divide_result(a: f64, b: f64) -> Result<f64, MathError> {
    if b == 0.0 {
        Err(MathError::DivisionByZero)
    } else {
        Ok(a / b)
    }
}

// ============================================================================
// Enums
// ============================================================================

#[cfg_attr(feature = "python", pyo3::pyclass(eq))]
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessStatus {
    Success(),
    Error(String),
    Warning { message: String, code: i32 },
}

#[cfg_attr(feature = "python", pyo3::pyclass(eq))]
#[derive(Debug, Clone, PartialEq)]
pub enum RgbColor {
    Red(),
    Green(),
    Blue(),
    Rgb { r: u8, g: u8, b: u8 },
}

// ============================================================================
// Structs
// ============================================================================

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

#[export]
pub struct Calculator {
    value: f64,
}

// ============================================================================
// Shared Implementation (Pure Rust)
// ============================================================================

impl Point {
    pub fn new_impl(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance_impl(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn distance_to_impl(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn repr_impl(&self) -> String {
        format!("Point({}, {})", self.x, self.y)
    }

    pub fn add_impl(&self, other: &Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Rectangle {
    pub fn new_impl(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    pub fn area_impl(&self) -> f64 {
        self.width * self.height
    }

    pub fn perimeter_impl(&self) -> f64 {
        2.0 * (self.width + self.height)
    }

    pub fn repr_impl(&self) -> String {
        format!("Rectangle({}x{})", self.width, self.height)
    }
}

impl Calculator {
    pub fn new_impl(value: f64) -> Self {
        Self { value }
    }

    pub fn add_impl(&mut self, n: f64) -> f64 {
        self.value += n;
        self.value
    }

    pub fn subtract_impl(&mut self, n: f64) -> f64 {
        self.value -= n;
        self.value
    }

    pub fn multiply_impl(&mut self, n: f64) -> f64 {
        self.value *= n;
        self.value
    }

    pub fn divide_impl(&mut self, n: f64) -> Result<f64, MathError> {
        if n == 0.0 {
            Err(MathError::DivisionByZero)
        } else {
            self.value /= n;
            Ok(self.value)
        }
    }

    pub fn get_value_impl(&self) -> f64 {
        self.value
    }

    pub fn reset_impl(&mut self) {
        self.value = 0.0;
    }
}

// ============================================================================
// Python Methods
// ============================================================================

#[cfg(feature = "python")]
use pyo3::prelude::*;

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

    #[pyo3(name = "distance_to")]
    fn distance_to_py(&self, other: &Point) -> f64 {
        self.distance_to_impl(other)
    }

    fn __repr__(&self) -> String {
        self.repr_impl()
    }

    fn __add__(&self, other: &Point) -> Point {
        self.add_impl(other)
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

    #[pyo3(name = "perimeter")]
    fn perimeter_py(&self) -> f64 {
        self.perimeter_impl()
    }

    fn __repr__(&self) -> String {
        self.repr_impl()
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl Calculator {
    #[new]
    fn new_py(value: f64) -> Self {
        Self::new_impl(value)
    }

    #[pyo3(name = "add")]
    fn add_py(&mut self, n: f64) -> f64 {
        self.add_impl(n)
    }

    #[pyo3(name = "subtract")]
    fn subtract_py(&mut self, n: f64) -> f64 {
        self.subtract_impl(n)
    }

    #[pyo3(name = "multiply")]
    fn multiply_py(&mut self, n: f64) -> f64 {
        self.multiply_impl(n)
    }

    #[pyo3(name = "divide")]
    fn divide_py(&mut self, n: f64) -> Result<f64, MathError> {
        self.divide_impl(n)
    }

    #[pyo3(name = "get_value")]
    fn get_value_py(&self) -> f64 {
        self.get_value_impl()
    }

    #[pyo3(name = "reset")]
    fn reset_py(&mut self) {
        self.reset_impl()
    }
}

// ============================================================================
// Node.js Methods
// ============================================================================

#[cfg(feature = "nodejs")]
use napi_derive::napi;

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

    #[napi(js_name = "distanceTo")]
    pub fn distance_to_js(&self, other: &Point) -> f64 {
        self.distance_to_impl(other)
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

    #[napi(js_name = "perimeter")]
    pub fn perimeter_js(&self) -> f64 {
        self.perimeter_impl()
    }
}

#[cfg(feature = "nodejs")]
#[napi]
impl Calculator {
    #[napi(constructor)]
    pub fn new_js(value: f64) -> Self {
        Self::new_impl(value)
    }

    #[napi(js_name = "add")]
    pub fn add_js(&mut self, n: f64) -> f64 {
        self.add_impl(n)
    }

    #[napi(js_name = "subtract")]
    pub fn subtract_js(&mut self, n: f64) -> f64 {
        self.subtract_impl(n)
    }

    #[napi(js_name = "multiply")]
    pub fn multiply_js(&mut self, n: f64) -> f64 {
        self.multiply_impl(n)
    }

    #[napi(js_name = "divide")]
    pub fn divide_js(&mut self, n: f64) -> Result<f64, MathError> {
        self.divide_impl(n)
    }

    #[napi(js_name = "getValue")]
    pub fn get_value_js(&self) -> f64 {
        self.get_value_impl()
    }

    #[napi(js_name = "reset")]
    pub fn reset_js(&mut self) {
        self.reset_impl()
    }
}

// ============================================================================
// Python Module
// ============================================================================

#[cfg(feature = "python")]
#[pymodule]
fn bridgerust_example(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Functions
    m.add_function(wrap_pyfunction!(greet, m)?)?;
    m.add_function(wrap_pyfunction!(add, m)?)?;
    m.add_function(wrap_pyfunction!(multiply, m)?)?;
    m.add_function(wrap_pyfunction!(is_even, m)?)?;
    m.add_function(wrap_pyfunction!(safe_divide, m)?)?;
    m.add_function(wrap_pyfunction!(find_first_even, m)?)?;
    m.add_function(wrap_pyfunction!(sum_numbers, m)?)?;
    m.add_function(wrap_pyfunction!(filter_positive, m)?)?;
    m.add_function(wrap_pyfunction!(double_all, m)?)?;
    m.add_function(wrap_pyfunction!(safe_sqrt, m)?)?;
    m.add_function(wrap_pyfunction!(safe_divide_result, m)?)?;

    // Enums
    m.add_class::<ProcessStatus>()?;
    m.add_class::<RgbColor>()?;

    // Structs
    m.add_class::<Point>()?;
    m.add_class::<Rectangle>()?;
    m.add_class::<Calculator>()?;

    Ok(())
}
