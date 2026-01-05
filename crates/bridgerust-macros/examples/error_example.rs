//! Example showing how to use the #[bridgerust::error] macro

#![allow(unexpected_cfgs)]
use bridgerust_macros::{error, export};
use std::fmt::{Display, Formatter};

/// Example error enum with Display implementation
#[error]
#[derive(Debug, Clone)]
pub enum MyError {
    Config(String),
    Database(String),
    Validation(String),
}

impl Display for MyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MyError::Config(msg) => write!(f, "Config error: {}", msg),
            MyError::Database(msg) => write!(f, "Database error: {}", msg),
            MyError::Validation(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for MyError {}

// Example function that returns Result
#[export]
pub fn might_fail(value: i32) -> Result<i32, MyError> {
    if value < 0 {
        Err(MyError::Validation("Value must be positive".to_string()))
    } else {
        Ok(value * 2)
    }
}

// In your binding code, you would use the generated conversion functions:
//
// #[cfg(feature = "python")]
// #[pyfunction]
// fn might_fail_py(value: i32) -> PyResult<i32> {
//     might_fail(value).map_err(to_py_err)
// }
//
// #[cfg(feature = "nodejs")]
// #[napi]
// fn might_fail_node(value: i32) -> napi::Result<i32> {
//     might_fail(value).map_err(to_napi_err)
// }

fn main() {
    println!("Error macro example");

    let err = MyError::Config("test".to_string());
    println!("Error: {}", err);
}
