use crate::error::Result;
use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KabodConfig {
    pub provider: String,
    pub url: String,
    pub api_key: Option<String>,
    pub timeout_ms: Option<u64>,

    // Provider specific settings can be opaque map
    #[serde(default)]
    pub options: std::collections::HashMap<String, String>,
}

impl KabodConfig {
    pub fn new() -> Result<Self> {
        let s = Config::builder()
            .add_source(File::with_name("kabod").required(false))
            .add_source(Environment::with_prefix("KABOD"))
            .build()?;

        s.try_deserialize().map_err(|e| e.into())
    }

    pub fn from_env() -> Result<Self> {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_from_env() {
        unsafe {
            env::set_var("KABOD_PROVIDER", "qdrant");
            env::set_var("KABOD_URL", "http://localhost:6333");
        }

        let config = KabodConfig::new().expect("Failed to load config");
        assert_eq!(config.provider, "qdrant");
        assert_eq!(config.url, "http://localhost:6333");
    }
}


pub use config::ConfigError;