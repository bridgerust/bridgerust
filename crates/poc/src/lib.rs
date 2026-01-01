use bridgerust::export;

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

// Example with Option
#[export]
pub fn divide(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 {
        None
    } else {
        Some(a / b)
    }
}

// Example with Vec
#[export]
pub fn sum_numbers(numbers: Vec<i32>) -> i32 {
    numbers.iter().sum()
}

// Example struct export
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
