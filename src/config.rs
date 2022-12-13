use anyhow::{Context, Result};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    fs::read_to_string,
    path::{Path, PathBuf},
};

fn bool_true() -> bool {
    true
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Root {
    /// Settings applied to all linters
    #[serde(default)]
    pub global: GlobalConfig,

    /// Linter settings
    pub linter: BTreeMap<String, LinterConfig>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GlobalConfig {
    /// Glob patterns to exclude files
    #[serde(default)]
    pub excludes: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct LinterConfig {
    /// Linter command to run
    pub command: String,

    /// Arguments
    #[serde(default)]
    pub options: Vec<String>,

    /// Glob patterns for files to be processed by the linter
    #[serde(default)]
    pub includes: Vec<String>,

    /// Glob patterns to exclude files
    #[serde(default)]
    pub excludes: Vec<String>,

    /// Working directory for the linter
    #[serde(default)]
    pub work_dir: PathBuf,

    /// Error message regex patterns
    ///
    /// You can use short commands (for details: [`to_re()`](crate::parser::to_re))
    /// - `%p`: Program name
    /// - `%f`: File name
    /// - `%l`: Line number
    /// - `%c`: Column number
    /// - `%m`: Message
    /// - `%%`: `%`
    ///
    /// Examples:
    /// - `^%f:%l:%c: %m$`
    #[serde(default)]
    pub formats: Vec<String>,

    /// Exclude git submodules (default value in toml is `true`)
    #[serde(default = "bool_true")]
    pub exclude_submodules: bool,

    /// Force the linter to process one file at a time.
    #[serde(default)]
    pub single_file: bool,

    /// Use hash functions to detect file changes
    #[serde(default)]
    pub check_hash: bool,
}

pub fn from_path(path: impl AsRef<Path>) -> Result<Root> {
    let config = read_to_string(&path)
        .with_context(|| format!("Cannot read config \"{}\"", path.as_ref().to_string_lossy()))?;
    toml::from_str(&config).context("Cannot parse config")
}
