//! Test suite for struct export functionality

#![allow(unexpected_cfgs)]
use bridgerust_macros::export;

// Test basic struct export
#[export]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

// Test struct with methods (methods need separate #[pymethods] and #[napi] impl blocks)
#[export]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

// Note: Methods would be in separate impl blocks with #[pymethods] and #[napi] attributes
// This is because PyO3 and napi-rs require separate attribute handling for methods

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_creation() {
        let point = Point { x: 1.0, y: 2.0 };
        assert_eq!(point.x, 1.0);
        assert_eq!(point.y, 2.0);
    }

    #[test]
    fn test_rectangle_creation() {
        let rect = Rectangle {
            width: 10.0,
            height: 20.0,
        };
        assert_eq!(rect.width, 10.0);
        assert_eq!(rect.height, 20.0);
    }
}
