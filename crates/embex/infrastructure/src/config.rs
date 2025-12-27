use bridge_embex_core::error::{EmbexError, Result};
use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbexConfig {
    pub provider: String,
    pub url: String,
    pub api_key: Option<String>,
    pub timeout_ms: Option<u64>,

    /// Maximum number of connections in the pool
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,

    /// Maximum number of idle connections to keep in the pool
    #[serde(default = "default_idle_timeout_secs")]
    pub idle_timeout_secs: u64,

    // Provider specific settings can be opaque map
    #[serde(default)]
    pub options: std::collections::HashMap<String, String>,
}

fn default_pool_size() -> u32 {
    10
}

fn default_idle_timeout_secs() -> u64 {
    90
}

impl Default for EmbexConfig {
    fn default() -> Self {
        Self {
            provider: String::new(),
            url: String::new(),
            api_key: None,
            timeout_ms: None,
            pool_size: default_pool_size(),
            idle_timeout_secs: default_idle_timeout_secs(),
            options: std::collections::HashMap::new(),
        }
    }
}

impl EmbexConfig {
    pub fn new() -> Result<Self> {
        let s = Config::builder()
            .add_source(File::with_name("embex").required(false))
            .add_source(Environment::with_prefix("EMBEX"))
            .build()
            .map_err(|e| EmbexError::Config(format!("Failed to load config: {}", e)))?;

        s.try_deserialize()
            .map_err(|e| EmbexError::Config(format!("Failed to deserialize config: {}", e)))
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
            env::set_var("EMBEX_PROVIDER", "qdrant");
            env::set_var("EMBEX_URL", "http://localhost:6333");
        }

        let config = EmbexConfig::new().expect("Failed to load config");
        assert_eq!(config.provider, "qdrant");
        assert_eq!(config.url, "http://localhost:6333");
    }
}

/// Configuration error type
#[derive(Debug, Clone)]
pub enum ConfigError {
    Message(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Message(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

