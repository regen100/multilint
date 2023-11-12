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
    #[serde(default)]
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
    // traverse from the root to the path and merge all config files
    let config_files = {
        let mut config_files = Vec::new();
        let mut path = path.as_ref();
        loop {
            let config_file = path.join("multilint.toml");
            if config_file.exists() {
                config_files.push(config_file);
            }
            if path.parent().is_none() {
                break;
            }
            path = path.parent().unwrap();
        }
        config_files.reverse();
        config_files
    };

    let mut merged = toml::Value::Table(toml::Table::new());
    for config_file in &config_files {
        let text = read_to_string(config_file)
            .with_context(|| format!("Cannot read config \"{}\"", config_file.to_string_lossy()))?;
        let value: toml::Table = toml::from_str(&text).with_context(|| {
            format!("Cannot parse config \"{}\"", config_file.to_string_lossy())
        })?;
        merge(&mut merged, &toml::Value::Table(value));
    }

    let merged_text = toml::to_string(&merged)?;
    toml::from_str(&merged_text).context("Cannot parse config")
}

fn merge(merged: &mut toml::Value, value: &toml::Value) {
    match value {
        toml::Value::Table(x) => match merged {
            toml::Value::Table(merged) => {
                for (k, v) in x.iter() {
                    match merged.get_mut(k) {
                        Some(x) => merge(x, v),
                        None => {
                            let _ = merged.insert(k.clone(), v.clone());
                        }
                    }
                }
            }
            _ => *merged = value.clone(),
        },
        _ => *merged = value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::from_path;
    use std::{fs::create_dir_all, fs::File, io::Write};
    use tempfile::tempdir;
    use test_log::test;

    #[test]
    fn run() {
        let root = tempdir().unwrap();
        let subdir = root.path().join("subdir");
        create_dir_all(&subdir).unwrap();

        {
            let path = root.path().join("multilint.toml");
            let mut config = File::create(&path).unwrap();
            writeln!(config, "[linter.test]").unwrap();
            writeln!(config, "command = 'true'").unwrap();
            writeln!(config, "includes = ['*']").unwrap();
        }

        {
            let path = subdir.join("multilint.toml");
            let mut config = File::create(&path).unwrap();
            writeln!(config, "[linter.test]").unwrap();
            writeln!(config, "command = 'false'").unwrap();
        }

        let config = from_path(&subdir).unwrap();
        assert_eq!(config.linter["test"].command, "false");
        assert_eq!(config.linter["test"].includes, vec!["*"]);
    }
}
