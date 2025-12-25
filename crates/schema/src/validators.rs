//! Validation functions for JSON Schema

use brige_core::BridgeError;
use serde_json::Value;

pub fn validate_type(value: &Value, expected_type: &str) -> Result<(), BridgeError> {
    Ok(())
}
