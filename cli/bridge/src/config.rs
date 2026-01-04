use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct BridgeRustConfig {
    pub package: PackageConfig,
    pub python: Option<PythonConfig>,
    pub nodejs: Option<NodejsConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageConfig {
    pub name: String,
    pub version: String,
    pub description: String,
    pub authors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PythonConfig {
    pub module_name: Option<String>,
    pub python_versions: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodejsConfig {
    pub package_name: Option<String>,
    pub node_versions: Option<Vec<String>>,
}

impl BridgeRustConfig {
    pub fn load(path: &PathBuf) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: BridgeRustConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, path: &PathBuf) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}
