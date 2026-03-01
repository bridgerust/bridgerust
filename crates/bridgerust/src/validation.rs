use regex::Regex;

use crate::BridgeError;
use crate::Result;

pub trait RuntimeValidate {
    fn runtime_validate(&self) -> Result<()>;
}

pub fn required_value(path: &str, present: bool) -> Result<()> {
    if present {
        Ok(())
    } else {
        Err(BridgeError(format!(
            "Validation failed for `{path}`: value is required"
        )))
    }
}

pub fn min_value(path: &str, actual: f64, min: f64) -> Result<()> {
    if actual >= min {
        Ok(())
    } else {
        Err(BridgeError(format!(
            "Validation failed for `{path}`: expected >= {min}, got {actual}"
        )))
    }
}

pub fn max_value(path: &str, actual: f64, max: f64) -> Result<()> {
    if actual <= max {
        Ok(())
    } else {
        Err(BridgeError(format!(
            "Validation failed for `{path}`: expected <= {max}, got {actual}"
        )))
    }
}

pub fn exact_len(path: &str, actual: usize, expected: usize) -> Result<()> {
    if actual == expected {
        Ok(())
    } else {
        Err(BridgeError(format!(
            "Validation failed for `{path}`: expected length {expected}, got {actual}"
        )))
    }
}

pub fn is_valid_email(value: &str) -> bool {
    let trimmed = value.trim();
    let Some((local, domain)) = trimmed.split_once('@') else {
        return false;
    };
    !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
}

pub fn is_valid_url(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.starts_with("http://") || trimmed.starts_with("https://")
}

pub fn matches_pattern(value: &str, pattern: &str) -> bool {
    Regex::new(pattern)
        .map(|regex| regex.is_match(value))
        .unwrap_or(false)
}
