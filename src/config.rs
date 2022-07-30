use anyhow::{Context, Result};
use serde::Deserialize;
use std::{collections::BTreeMap, fs::read_to_string, path::Path};

#[derive(Debug, Clone, Deserialize)]
pub struct Root {
    pub global: Option<GlobalConfig>,
    pub linter: BTreeMap<String, LinterConfig>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub excludes: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct LinterConfig {
    pub command: String,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub includes: Vec<String>,
    #[serde(default)]
    pub excludes: Vec<String>,
}

pub fn from_path(path: impl AsRef<Path>) -> Result<Root> {
    let config = read_to_string(&path)
        .with_context(|| format!("Cannot read config {}", path.as_ref().to_string_lossy()))?;
    toml::from_str(&config).context("Cannot parse config")
}
