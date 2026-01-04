//! Test suite for the `#[bridgerust::export]` macro with enums

#![allow(unexpected_cfgs)]
use bridgerust_macros::export;

// Test simple enum
#[export]
pub enum SimpleEnum {
    Variant1,
    Variant2,
    Variant3,
}

// Test enum with data
#[export]
pub enum Status {
    Success,
    Error(String),
    Warning { message: String, code: i32 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_enum() {
        let variant = SimpleEnum::Variant1;
        // Just verify it compiles
        match variant {
            SimpleEnum::Variant1 => {}
            SimpleEnum::Variant2 => {}
            SimpleEnum::Variant3 => {}
        }
    }

    #[test]
    fn test_status_enum() {
        let success = Status::Success;
        let error = Status::Error("test".to_string());
        let warning = Status::Warning {
            message: "test".to_string(),
            code: 42,
        };

        // Just verify it compiles
        match success {
            Status::Success => {}
            Status::Error(_) => {}
            Status::Warning { .. } => {}
        }

        match error {
            Status::Success => {}
            Status::Error(msg) => assert_eq!(msg, "test"),
            Status::Warning { .. } => {}
        }

        match warning {
            Status::Success => {}
            Status::Error(_) => {}
            Status::Warning { message, code } => {
                assert_eq!(message, "test");
                assert_eq!(code, 42);
            }
        }
    }
}
