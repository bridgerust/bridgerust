//! Test suite for async function export

#![allow(unexpected_cfgs)]
use bridgerust_macros::export;

#[export]
pub async fn simple_async() -> i32 {
    42
}

#[export]
pub async fn async_add(a: i32, b: i32) -> i32 {
    a + b
}

#[export]
pub async fn async_greet(name: String) -> String {
    format!("Hello, {}!", name)
}

#[export]
pub async fn async_maybe_divide(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 { None } else { Some(a / b) }
}

// Note: This test file verifies that async functions can be exported
// The macro expansion is validated by the compiler
// For runtime testing of async functions, see the e2e test suite
