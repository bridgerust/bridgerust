//! Bridge Rust JSON Schema Validator
use bridge_core::Result;
use bridge_core::error::BridgeError;
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
        let instance: Value = serde_json::from_str(instance)?;
        validate_value(&self.schema, &instance, "$")
    }

    pub fn is_valid(&self, instance: &str) -> bool {
        self.validate(instance).is_ok()
    }
}

fn validate_value(schema: &Value, instance: &Value, path: &str) -> Result<()> {
    validate_type(schema, instance, path)?;
    validate_const(schema, instance, path)?;
    validate_enum(schema, instance, path)?;
    validate_string_constraints(schema, instance, path)?;
    validate_string_pattern(schema, instance, path)?;
    validate_number_constraints(schema, instance, path)?;
    validate_number_multiples(schema, instance, path)?;
    validate_array_constraints(schema, instance, path)?;
    validate_array_contains(schema, instance, path)?;
    validate_array_uniqueness(schema, instance, path)?;
    validate_object_constraints(schema, instance, path)?;
    validate_composition(schema, instance, path)?;
    Ok(())
}

fn validate_const(schema: &Value, instance: &Value, path: &str) -> Result<()> {
    let Some(expected) = schema.get("const") else {
        return Ok(());
    };

    if expected == instance {
        Ok(())
    } else {
        Err(BridgeError::Validation(format!(
            "{path}: value does not match const constraint"
        )))
    }
}

fn validate_type(schema: &Value, instance: &Value, path: &str) -> Result<()> {
    let Some(typ) = schema.get("type") else {
        return Ok(());
    };

    let is_match = match typ {
        Value::String(s) => type_matches(s, instance),
        Value::Array(types) => types
            .iter()
            .filter_map(Value::as_str)
            .any(|raw| type_matches(raw, instance)),
        _ => {
            return Err(BridgeError::Validation(format!(
                "{path}: schema field `type` must be a string or array"
            )));
        }
    };

    if is_match {
        Ok(())
    } else {
        Err(BridgeError::Validation(format!(
            "{path}: expected type {}, got {}",
            describe_expected_type(typ),
            value_type_name(instance)
        )))
    }
}

fn validate_enum(schema: &Value, instance: &Value, path: &str) -> Result<()> {
    let Some(candidates) = schema.get("enum") else {
        return Ok(());
    };

    let Value::Array(items) = candidates else {
        return Err(BridgeError::Validation(format!(
            "{path}: schema field `enum` must be an array"
        )));
    };

    if items.iter().any(|candidate| candidate == instance) {
        Ok(())
    } else {
        Err(BridgeError::Validation(format!(
            "{path}: value does not match any enum variant"
        )))
    }
}

fn validate_string_constraints(schema: &Value, instance: &Value, path: &str) -> Result<()> {
    let Value::String(value) = instance else {
        return Ok(());
    };

    if let Some(min) = schema.get("minLength").and_then(Value::as_u64)
        && value.chars().count() < min as usize
    {
        return Err(BridgeError::Validation(format!(
            "{path}: string is shorter than minLength={min}"
        )));
    }

    if let Some(max) = schema.get("maxLength").and_then(Value::as_u64)
        && value.chars().count() > max as usize
    {
        return Err(BridgeError::Validation(format!(
            "{path}: string is longer than maxLength={max}"
        )));
    }

    Ok(())
}

fn validate_string_pattern(schema: &Value, instance: &Value, path: &str) -> Result<()> {
    let Value::String(value) = instance else {
        return Ok(());
    };

    let Some(pattern) = schema.get("pattern") else {
        return Ok(());
    };

    let Some(pattern) = pattern.as_str() else {
        return Err(BridgeError::Validation(format!(
            "{path}: schema field `pattern` must be a string"
        )));
    };

    let regex = regex::Regex::new(pattern).map_err(|err| {
        BridgeError::Validation(format!("{path}: invalid regex in `pattern`: {err}"))
    })?;

    if regex.is_match(value) {
        Ok(())
    } else {
        Err(BridgeError::Validation(format!(
            "{path}: string does not match pattern `{pattern}`"
        )))
    }
}

fn validate_number_constraints(schema: &Value, instance: &Value, path: &str) -> Result<()> {
    let Some(value) = instance.as_f64() else {
        return Ok(());
    };

    if let Some(min) = schema.get("minimum").and_then(Value::as_f64)
        && value < min
    {
        return Err(BridgeError::Validation(format!(
            "{path}: number is lower than minimum={min}"
        )));
    }

    if let Some(max) = schema.get("maximum").and_then(Value::as_f64)
        && value > max
    {
        return Err(BridgeError::Validation(format!(
            "{path}: number is greater than maximum={max}"
        )));
    }

    if let Some(min) = schema.get("exclusiveMinimum").and_then(Value::as_f64)
        && value <= min
    {
        return Err(BridgeError::Validation(format!(
            "{path}: number must be > exclusiveMinimum={min}"
        )));
    }

    if let Some(max) = schema.get("exclusiveMaximum").and_then(Value::as_f64)
        && value >= max
    {
        return Err(BridgeError::Validation(format!(
            "{path}: number must be < exclusiveMaximum={max}"
        )));
    }

    Ok(())
}

fn validate_number_multiples(schema: &Value, instance: &Value, path: &str) -> Result<()> {
    let Some(value) = instance.as_f64() else {
        return Ok(());
    };

    let Some(divisor) = schema.get("multipleOf").and_then(Value::as_f64) else {
        return Ok(());
    };

    if divisor <= 0.0 {
        return Err(BridgeError::Validation(format!(
            "{path}: schema field `multipleOf` must be > 0"
        )));
    }

    let ratio = value / divisor;
    let nearest = ratio.round();
    if (ratio - nearest).abs() <= 1e-9 {
        Ok(())
    } else {
        Err(BridgeError::Validation(format!(
            "{path}: number is not a multiple of {divisor}"
        )))
    }
}

fn validate_array_constraints(schema: &Value, instance: &Value, path: &str) -> Result<()> {
    let Value::Array(items) = instance else {
        return Ok(());
    };

    if let Some(min) = schema.get("minItems").and_then(Value::as_u64)
        && items.len() < min as usize
    {
        return Err(BridgeError::Validation(format!(
            "{path}: array has fewer than minItems={min}"
        )));
    }

    if let Some(max) = schema.get("maxItems").and_then(Value::as_u64)
        && items.len() > max as usize
    {
        return Err(BridgeError::Validation(format!(
            "{path}: array has more than maxItems={max}"
        )));
    }

    if let Some(item_schema) = schema.get("items") {
        for (index, value) in items.iter().enumerate() {
            validate_value(item_schema, value, &format!("{path}[{index}]"))?;
        }
    }

    Ok(())
}

fn validate_array_contains(schema: &Value, instance: &Value, path: &str) -> Result<()> {
    let Value::Array(items) = instance else {
        return Ok(());
    };

    let Some(contains_schema) = schema.get("contains") else {
        return Ok(());
    };

    let mut count = 0usize;
    for (index, value) in items.iter().enumerate() {
        if validate_value(contains_schema, value, &format!("{path}[{index}]")).is_ok() {
            count += 1;
        }
    }

    let min_contains = schema
        .get("minContains")
        .and_then(Value::as_u64)
        .unwrap_or(1) as usize;
    let max_contains = schema
        .get("maxContains")
        .and_then(Value::as_u64)
        .map(|v| v as usize);

    if count < min_contains {
        return Err(BridgeError::Validation(format!(
            "{path}: array matches `contains` {count} times, below minContains={min_contains}"
        )));
    }

    if let Some(max) = max_contains
        && count > max
    {
        return Err(BridgeError::Validation(format!(
            "{path}: array matches `contains` {count} times, above maxContains={max}"
        )));
    }

    Ok(())
}

fn validate_array_uniqueness(schema: &Value, instance: &Value, path: &str) -> Result<()> {
    let Value::Array(items) = instance else {
        return Ok(());
    };

    if !matches!(schema.get("uniqueItems"), Some(Value::Bool(true))) {
        return Ok(());
    }

    for i in 0..items.len() {
        for j in (i + 1)..items.len() {
            if items[i] == items[j] {
                return Err(BridgeError::Validation(format!(
                    "{path}: array violates uniqueItems at indexes {i} and {j}"
                )));
            }
        }
    }

    Ok(())
}

fn validate_object_constraints(schema: &Value, instance: &Value, path: &str) -> Result<()> {
    let Value::Object(obj) = instance else {
        return Ok(());
    };

    if let Some(required) = schema.get("required") {
        let Value::Array(fields) = required else {
            return Err(BridgeError::Validation(format!(
                "{path}: schema field `required` must be an array"
            )));
        };

        for field in fields {
            let Some(field_name) = field.as_str() else {
                return Err(BridgeError::Validation(format!(
                    "{path}: `required` entries must be strings"
                )));
            };

            if !obj.contains_key(field_name) {
                return Err(BridgeError::Validation(format!(
                    "{path}: missing required property `{field_name}`"
                )));
            }
        }
    }

    if let Some(min) = schema.get("minProperties").and_then(Value::as_u64)
        && obj.len() < min as usize
    {
        return Err(BridgeError::Validation(format!(
            "{path}: object has fewer than minProperties={min}"
        )));
    }

    if let Some(max) = schema.get("maxProperties").and_then(Value::as_u64)
        && obj.len() > max as usize
    {
        return Err(BridgeError::Validation(format!(
            "{path}: object has more than maxProperties={max}"
        )));
    }

    let mut declared_keys: std::collections::HashSet<&str> = std::collections::HashSet::new();
    if let Some(properties) = schema.get("properties") {
        let Value::Object(map) = properties else {
            return Err(BridgeError::Validation(format!(
                "{path}: schema field `properties` must be an object"
            )));
        };

        for (key, child_schema) in map {
            declared_keys.insert(key);
            if let Some(child_value) = obj.get(key) {
                validate_value(child_schema, child_value, &format!("{path}.{key}"))?;
            }
        }
    }

    if let Some(additional_props) = schema.get("additionalProperties") {
        match additional_props {
            Value::Bool(false) => {
                for key in obj.keys() {
                    if !declared_keys.contains(key.as_str()) {
                        return Err(BridgeError::Validation(format!(
                            "{path}: additional property `{key}` is not allowed"
                        )));
                    }
                }
            }
            Value::Object(_) => {
                for (key, value) in obj {
                    if !declared_keys.contains(key.as_str()) {
                        validate_value(additional_props, value, &format!("{path}.{key}"))?;
                    }
                }
            }
            Value::Bool(true) => {}
            _ => {
                return Err(BridgeError::Validation(format!(
                    "{path}: schema field `additionalProperties` must be bool or object"
                )));
            }
        }
    }

    Ok(())
}

fn validate_composition(schema: &Value, instance: &Value, path: &str) -> Result<()> {
    if let Some(all_of) = schema.get("allOf") {
        let Value::Array(schemas) = all_of else {
            return Err(BridgeError::Validation(format!(
                "{path}: schema field `allOf` must be an array"
            )));
        };
        for child in schemas {
            validate_value(child, instance, path)?;
        }
    }

    if let Some(any_of) = schema.get("anyOf") {
        let Value::Array(schemas) = any_of else {
            return Err(BridgeError::Validation(format!(
                "{path}: schema field `anyOf` must be an array"
            )));
        };
        let mut ok = false;
        for child in schemas {
            if validate_value(child, instance, path).is_ok() {
                ok = true;
                break;
            }
        }
        if !ok {
            return Err(BridgeError::Validation(format!(
                "{path}: value does not satisfy any `anyOf` schema"
            )));
        }
    }

    if let Some(one_of) = schema.get("oneOf") {
        let Value::Array(schemas) = one_of else {
            return Err(BridgeError::Validation(format!(
                "{path}: schema field `oneOf` must be an array"
            )));
        };
        let mut matches = 0usize;
        for child in schemas {
            if validate_value(child, instance, path).is_ok() {
                matches += 1;
            }
        }
        if matches != 1 {
            return Err(BridgeError::Validation(format!(
                "{path}: value must satisfy exactly one `oneOf` schema (matched {matches})"
            )));
        }
    }

    if let Some(not_schema) = schema.get("not")
        && validate_value(not_schema, instance, path).is_ok()
    {
        return Err(BridgeError::Validation(format!(
            "{path}: value matches forbidden `not` schema"
        )));
    }

    Ok(())
}

fn type_matches(expected: &str, instance: &Value) -> bool {
    match expected {
        "null" => instance.is_null(),
        "boolean" => instance.is_boolean(),
        "object" => instance.is_object(),
        "array" => instance.is_array(),
        "number" => instance.is_number(),
        "integer" => instance.as_i64().is_some() || instance.as_u64().is_some(),
        "string" => instance.is_string(),
        _ => false,
    }
}

fn describe_expected_type(value: &Value) -> String {
    match value {
        Value::String(s) => s.to_string(),
        Value::Array(items) => {
            let raw = items
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(" | ");
            if raw.is_empty() {
                "<invalid-type-array>".to_string()
            } else {
                raw
            }
        }
        _ => "<invalid-type>".to_string(),
    }
}

fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(number) => {
            if number.as_i64().is_some() || number.as_u64().is_some() {
                "integer"
            } else {
                "number"
            }
        }
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
        assert!(!validator.is_valid("42"));
    }

    #[test]
    fn test_object_required_and_properties() {
        let schema = json!({
            "type": "object",
            "required": ["name", "age"],
            "properties": {
                "name": { "type": "string", "minLength": 2 },
                "age": { "type": "integer", "minimum": 0 }
            },
            "additionalProperties": false
        })
        .to_string();

        let validator = Validator::new(&schema).unwrap();
        assert!(validator.is_valid(r#"{"name":"Ada","age":33}"#));
        assert!(!validator.is_valid(r#"{"name":"A","age":33}"#));
        assert!(!validator.is_valid(r#"{"name":"Ada"}"#));
        assert!(!validator.is_valid(r#"{"name":"Ada","age":33,"extra":true}"#));
    }

    #[test]
    fn test_array_and_composition_constraints() {
        let schema = json!({
            "type": "array",
            "minItems": 2,
            "items": {
                "oneOf": [
                    { "type": "integer", "minimum": 0 },
                    { "type": "string", "minLength": 1 }
                ]
            }
        })
        .to_string();

        let validator = Validator::new(&schema).unwrap();
        assert!(validator.is_valid(r#"[1, "two", 3]"#));
        assert!(!validator.is_valid(r#"[1]"#));
        assert!(!validator.is_valid(r#"[1, -2]"#));
    }

    #[test]
    fn test_string_pattern_const_and_multiple_of() {
        let schema = json!({
            "type": "object",
            "properties": {
                "code": { "type": "string", "pattern": "^[A-Z]{2}[0-9]{2}$" },
                "kind": { "const": "invoice" },
                "step": { "type": "number", "multipleOf": 0.5 }
            },
            "required": ["code", "kind", "step"]
        })
        .to_string();

        let validator = Validator::new(&schema).unwrap();
        assert!(validator.is_valid(r#"{"code":"AB12","kind":"invoice","step":1.5}"#));
        assert!(!validator.is_valid(r#"{"code":"ab12","kind":"invoice","step":1.5}"#));
        assert!(!validator.is_valid(r#"{"code":"AB12","kind":"receipt","step":1.5}"#));
        assert!(!validator.is_valid(r#"{"code":"AB12","kind":"invoice","step":1.3}"#));
    }

    #[test]
    fn test_contains_unique_and_additional_properties_schema() {
        let schema = json!({
            "type": "object",
            "properties": {
                "tags": {
                    "type": "array",
                    "uniqueItems": true,
                    "contains": { "type": "string", "pattern": "^prod-" },
                    "minContains": 1
                }
            },
            "additionalProperties": { "type": "integer", "minimum": 0 }
        })
        .to_string();

        let validator = Validator::new(&schema).unwrap();
        assert!(validator.is_valid(r#"{"tags":["prod-us","blue"],"retries":2}"#));
        assert!(!validator.is_valid(r#"{"tags":["prod-us","prod-us"]}"#));
        assert!(!validator.is_valid(r#"{"tags":["blue"],"retries":2}"#));
        assert!(!validator.is_valid(r#"{"tags":["prod-us"],"retries":-1}"#));
    }
}
