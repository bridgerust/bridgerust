//! Comprehensive BridgeRust Example
//!
//! This example demonstrates all major features of BridgeRust:
//! - Function bridges with various types
//! - Struct bridges with methods
//! - Error handling
//! - Complex data structures

use bridgerust::{bridge, error};
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

#[bridge]
pub fn greet(name: String) -> String {
    format!("Hello, {}! Welcome to BridgeRust.", name)
}

#[bridge]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[bridge]
pub fn multiply(a: f64, b: f64) -> f64 {
    a * b
}

#[bridge]
pub fn is_even(n: i32) -> bool {
    n % 2 == 0
}

// ============================================================================
// Functions with Option
// ============================================================================

#[bridge]
pub fn safe_divide(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 {
        None
    } else {
        Some(a / b)
    }
}

#[bridge]
pub fn find_first_even(numbers: Vec<i32>) -> Option<i32> {
    numbers.into_iter().find(|&n| n % 2 == 0)
}

// ============================================================================
// Functions with Vec
// ============================================================================

#[bridge]
pub fn sum_numbers(numbers: Vec<i32>) -> i32 {
    numbers.iter().sum()
}

#[bridge]
pub fn filter_positive(numbers: Vec<i32>) -> Vec<i32> {
    numbers.into_iter().filter(|&n| n > 0).collect()
}

#[bridge]
pub fn double_all(numbers: Vec<i32>) -> Vec<i32> {
    numbers.into_iter().map(|n| n * 2).collect()
}

// ============================================================================
// Functions with Result
// ============================================================================

#[bridge]
pub fn safe_sqrt(n: f64) -> Result<f64, MathError> {
    if n < 0.0 {
        Err(MathError::NegativeNumber)
    } else {
        Ok(n.sqrt())
    }
}

#[bridge]
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

#[bridge]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[bridge]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

#[bridge]
pub struct Calculator {
    value: f64,
}

// ============================================================================
// Shared Implementation (Pure Rust)
// ============================================================================

// ============================================================================
// Shared Implementation (Unified via BridgeRust)
// ============================================================================

#[bridge]
impl Point {
    #[constructor]
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn distance_to(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    #[cfg(feature = "python")]
    fn __repr__(&self) -> String {
        format!("Point({}, {})", self.x, self.y)
    }

    #[cfg(feature = "python")]
    fn __add__(&self, other: &Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

#[bridge]
impl Rectangle {
    #[constructor]
    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    pub fn area(&self) -> f64 {
        self.width * self.height
    }

    pub fn perimeter(&self) -> f64 {
        2.0 * (self.width + self.height)
    }

    #[cfg(feature = "python")]
    fn __repr__(&self) -> String {
        format!("Rectangle({}x{})", self.width, self.height)
    }
}

#[bridge]
impl Calculator {
    #[constructor]
    pub fn new(value: f64) -> Self {
        Self { value }
    }

    pub fn add(&mut self, n: f64) -> f64 {
        self.value += n;
        self.value
    }

    pub fn subtract(&mut self, n: f64) -> f64 {
        self.value -= n;
        self.value
    }

    pub fn multiply(&mut self, n: f64) -> f64 {
        self.value *= n;
        self.value
    }

    pub fn divide(&mut self, n: f64) -> Result<f64, MathError> {
        if n == 0.0 {
            Err(MathError::DivisionByZero)
        } else {
            self.value /= n;
            Ok(self.value)
        }
    }

    pub fn get_value(&self) -> f64 {
        self.value
    }

    pub fn reset(&mut self) {
        self.value = 0.0;
    }
}

// ============================================================================
// Python Methods
// ============================================================================

#[cfg(feature = "python")]
use pyo3::prelude::*;

// Point and Rectangle Python extensions have been merged above

// ============================================================================
// Node.js Methods
// ============================================================================

#[cfg(feature = "nodejs")]
#[allow(unused_imports)]
use napi_derive::napi;

// ============================================================================
// Python Module
// ============================================================================
#[cfg(feature = "python")]
#[pymodule]
fn bridgerust_example(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
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

    // Exceptions
    m.add("CustomError", py.get_type::<CustomError>())?;

    Ok(())
}

// Custom exception for the example module
#[bridgerust::exception(module = "bridgerust_example")]
pub struct CustomError;
