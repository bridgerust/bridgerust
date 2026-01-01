//! Example showing how to use #[bridgerust::export] with structs

#![allow(unexpected_cfgs)]
use bridgerust::export;

/// Simple struct that can be exported to both Python and Node.js
#[export]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// Struct with methods
///
/// Note: Methods require separate impl blocks with #[pymethods] and #[napi] attributes
/// because PyO3 and napi-rs handle methods differently.
#[export]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

// For Python methods, you would add:
// #[cfg(feature = "python")]
// use bridgerust::pyo3::prelude::*;
//
// #[cfg(feature = "python")]
// #[pymethods]
// impl Rectangle {
//     #[new]
//     fn new(width: f64, height: f64) -> Self {
//         Self { width, height }
//     }
//
//     fn area(&self) -> f64 {
//         self.width * self.height
//     }
// }

// For Node.js methods, you would add:
// #[cfg(feature = "nodejs")]
// use bridgerust::napi_derive::napi;
//
// #[cfg(feature = "nodejs")]
// #[napi]
// impl Rectangle {
//     #[napi(constructor)]
//     pub fn new(width: f64, height: f64) -> Self {
//         Self { width, height }
//     }
//
//     #[napi]
//     pub fn area(&self) -> f64 {
//         self.width * self.height
//     }
// }

fn main() {
    let point = Point { x: 1.0, y: 2.0 };
    println!("Point: ({}, {})", point.x, point.y);

    let rect = Rectangle {
        width: 10.0,
        height: 20.0,
    };
    println!("Rectangle: {}x{}", rect.width, rect.height);
}
