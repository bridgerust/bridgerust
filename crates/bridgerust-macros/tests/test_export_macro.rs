//! Test suite for the `#[export]` macro
//!
//! These tests verify that the macro correctly expands to PyO3 and napi-rs attributes

#![allow(unexpected_cfgs)]
use bridgerust_macros::export;

#[export]
pub fn simple_function() -> i32 {
    42
}

#[export]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[export]
pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

#[export]
pub fn maybe_divide(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 { None } else { Some(a / b) }
}

#[export]
pub fn sum_numbers(numbers: Vec<i32>) -> i32 {
    numbers.iter().sum()
}

#[export]
pub fn is_even(n: i32) -> bool {
    n % 2 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_function() {
        assert_eq!(simple_function(), 42);
    }

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn test_greet() {
        assert_eq!(greet("World".to_string()), "Hello, World!");
    }

    #[test]
    fn test_maybe_divide() {
        assert_eq!(maybe_divide(10.0, 2.0), Some(5.0));
        assert_eq!(maybe_divide(10.0, 0.0), None);
    }

    #[test]
    fn test_sum_numbers() {
        assert_eq!(sum_numbers(vec![1, 2, 3, 4]), 10);
    }

    #[test]
    fn test_is_even() {
        assert!(is_even(2));
        assert!(!is_even(3));
    }
}
