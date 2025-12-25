//! Bridge Rust JSON Schema Validator
use bridge_core::Result;
use serde_json::Value;

pub struct Validator {
    schema: Value,
}

impl Validator {
    pub fn new(schema: &str) -> Result<Self> {
        let schema: Value = serde_json::from_str(schema)?;
        Ok(Self { schema })
    }

    pub fn validate(&self, instance: &str) -> Result<()> {
        let _instance: Value = serde_json::from_str(instance)?;
        // TODO: Implement validation logic
        Ok(())
    }

    pub fn is_valid(&self, instance: &str) -> bool {
        self.validate(instance).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let schema = r#"{"type": "string"}"#;
        let result = Validator::new(schema);

        assert!(result.is_ok());
    }

    #[test]
    fn test_is_valid() {
        let schema = r#"{"type": "string"}"#;
        let validator = Validator::new(schema).unwrap();
        assert!(validator.is_valid(r#""hello""#));
    }
}
