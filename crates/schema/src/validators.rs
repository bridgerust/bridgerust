//! Validation functions for JSON Schema

use bridge_core::error::BridgeError;
use serde_json::Value;

pub fn validate_type(value: &Value, expected_type: &str) -> Result<(), BridgeError> {
    let matches = match expected_type {
        "null" => value.is_null(),
        "boolean" => value.is_boolean(),
        "object" => value.is_object(),
        "array" => value.is_array(),
        "number" => value.is_number(),
        "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
        "string" => value.is_string(),
        _ => false,
    };

    if matches {
        Ok(())
    } else {
        Err(BridgeError::Validation(format!(
            "expected type `{expected_type}`"
        )))
    }
}
