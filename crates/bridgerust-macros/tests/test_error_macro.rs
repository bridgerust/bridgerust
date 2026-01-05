//! Test suite for the `#[bridgerust::error]` macro

#![allow(unexpected_cfgs)]
use bridgerust_macros::error;
use std::fmt::{Display, Formatter};

// Test basic error enum with Display implementation
#[error]
#[derive(Debug, Clone)]
pub enum TestError {
    Config(String),
    Database(String),
}

impl Display for TestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TestError::Config(msg) => write!(f, "Config error: {}", msg),
            TestError::Database(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for TestError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = TestError::Config("test".to_string());
        assert_eq!(err.to_string(), "Config error: test");
    }

    #[test]
    fn test_error_conversion_functions_exist() {
        // Just verify the functions are generated
        let _err = TestError::Config("test".to_string());

        #[cfg(feature = "python")]
        {
            // Function should exist (won't actually work without PyO3 context)
            let _ = to_py_err;
        }

        #[cfg(feature = "nodejs")]
        {
            // Function should exist (won't actually work without napi context)
            let _ = to_napi_err;
        }
    }
}
