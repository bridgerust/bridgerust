//! BridgeRust Core
//!
//! Shared utilities for the BridgeRust engines

pub mod buffer;
pub mod error;
pub mod io;
#[cfg(feature = "simd")]
pub mod simd;

pub use error::{BridgeError, Result};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
