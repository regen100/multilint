use crate::config::{GlobalConfig, LinterConfig};
use anyhow::{ensure, Result};
use ignore::{overrides::OverrideBuilder, DirEntry, Walk, WalkBuilder};
use log::{debug, warn};
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

#[derive(Debug, Clone)]
pub struct Linter {
    command: String,
    options: Vec<String>,
    includes: Vec<String>,
    excludes: Vec<String>,
    work_dir: PathBuf,
}

impl Linter {
    pub fn from_config(config: LinterConfig, global: &GlobalConfig) -> Self {
        Self {
            command: config.command,
            options: config.options,
            includes: config.includes,
            excludes: [global.excludes.clone(), config.excludes].concat(),
            work_dir: config.work_dir,
        }
    }

    pub fn is_executable(&self) -> bool {
        which::which(&self.command).is_ok()
    }

    pub fn run_files<I, P>(&self, root: impl AsRef<Path>, files: I) -> Result<Output>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let mut cmd = Command::new(&self.command);
        cmd.args(&self.options);
        for f in files {
            cmd.arg(f.as_ref());
        }
        if !self.work_dir.as_os_str().is_empty() {
            let work_dir = root.as_ref().join(&self.work_dir);
            ensure!(
                work_dir.is_dir(),
                "{} is not a directory",
                work_dir.to_string_lossy()
            );
            cmd.current_dir(work_dir);
        }
        debug!(
            "command: {:?}",
            [vec![cmd.get_program()], cmd.get_args().collect()].concat()
        );
        let output = cmd.output()?;
        debug!("output: {:?}", &output);
        Ok(output)
    }

    pub fn run(&self, root: impl AsRef<Path>) -> Result<Option<Output>> {
        let files = self.paths(&root)?;
        if !self.includes.is_empty() && files.is_empty() {
            debug!("no files");
            return Ok(None);
        }

        Ok(Some(self.run_files(root, files)?))
    }

    fn walk(&self, root: impl AsRef<Path>) -> Result<Walk> {
        let overrides = {
            let mut builder = OverrideBuilder::new(&root);
            for pattern in &self.includes {
                builder.add(pattern)?;
            }
            for pattern in &self.excludes {
                builder.add(&format!("!{}", pattern))?;
            }
            builder.build()?
        };
        Ok(WalkBuilder::new(&root).overrides(overrides).build())
    }

    fn paths(&self, root: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
        if self.includes.is_empty() {
            return Ok(vec![]);
        }

        Ok(self
            .walk(&root)?
            .into_iter()
            .filter_map(|entry| -> Option<DirEntry> {
                match entry {
                    Ok(entry) => Some(entry),
                    Err(err) => {
                        warn!("traversal error: {}", err);
                        None
                    }
                }
            })
            .filter_map(|entry| -> Option<PathBuf> {
                if let Some(file_type) = entry.file_type() {
                    if !file_type.is_dir() && !file_type.is_symlink() {
                        return Some(entry.into_path());
                    }
                }
                None
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::Linter;
    use crate::config::LinterConfig;
    use std::{
        default::Default,
        fs::{create_dir, File},
    };
    use tempfile::tempdir;
    use test_log::test;

    #[cfg(windows)]
    const NEWLINE: &'static str = "\r\n";
    #[cfg(not(windows))]
    const NEWLINE: &'static str = "\n";

    #[test]
    fn files_with_excludes() {
        let root = tempdir().unwrap();
        File::create(root.path().join("main.rs")).unwrap();
        File::create(root.path().join("lib.rs")).unwrap();
        let linter = Linter::from_config(
            LinterConfig {
                command: "echo".to_string(),
                options: vec!["option".to_string()],
                includes: vec!["*.rs".to_string()],
                excludes: vec!["lib.rs".to_string()],
                ..Default::default()
            },
            &Default::default(),
        );
        let output = linter.run(&root).unwrap().unwrap();
        assert!(output.status.success());
        assert_eq!(
            std::str::from_utf8(&output.stdout).unwrap(),
            format!(
                "option {}{}",
                root.path().join("main.rs").to_string_lossy(),
                NEWLINE
            )
        );
    }

    #[test]
    fn no_files() {
        let root = tempdir().unwrap();
        let linter = Linter::from_config(
            LinterConfig {
                command: "echo".to_string(),
                includes: vec!["*.rs".to_string()],
                ..Default::default()
            },
            &Default::default(),
        );
        let output = linter.run(&root).unwrap();
        assert_eq!(output, None);
    }

    #[test]
    fn no_includes() {
        let root = tempdir().unwrap();
        File::create(root.path().join("main.rs")).unwrap();
        let linter = Linter::from_config(
            LinterConfig {
                command: "echo".to_string(),
                ..Default::default()
            },
            &Default::default(),
        );
        let output = linter.run(&root).unwrap().unwrap();
        assert!(output.status.success());
        assert_eq!(std::str::from_utf8(&output.stdout).unwrap(), NEWLINE);
    }

    #[test]
    fn subdir() {
        let root = tempdir().unwrap();
        let sub = root.path().join("sub");
        create_dir(&sub).unwrap();
        File::create(sub.join("main.rs")).unwrap();
        let linter = Linter::from_config(
            LinterConfig {
                command: "ls".to_string(),
                work_dir: "sub".into(),
                ..Default::default()
            },
            &Default::default(),
        );
        let output = linter.run(&root).unwrap().unwrap();
        assert!(output.status.success());
        assert!(std::str::from_utf8(&output.stdout)
            .unwrap()
            .contains("main.rs"),);
    }
}
