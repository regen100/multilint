use crate::{
    config::{GlobalConfig, LinterConfig},
    xargs::Xargs,
};
use anyhow::{ensure, Result};
use ignore::{overrides::OverrideBuilder, DirEntry, Match, WalkBuilder};
use log::{debug, warn};
use std::{
    fs,
    path::{Path, PathBuf},
    process,
    time::SystemTime,
};

#[derive(Debug, Clone)]
pub struct Output {
    process: process::Output,
    modified: Vec<PathBuf>,
}

impl Output {
    pub fn success(&self) -> bool {
        self.process.status.success() && self.modified.is_empty()
    }

    pub fn stdout(&self) -> &[u8] {
        &self.process.stdout
    }

    pub fn stderr(&self) -> &[u8] {
        &self.process.stderr
    }

    pub fn modified(&self) -> &[PathBuf] {
        &self.modified
    }
}

struct Entry {
    file: PathBuf,
    modified: SystemTime,
}

#[derive(Debug, Clone)]
pub struct Linter {
    command: String,
    options: Vec<String>,
    includes: Vec<String>,
    excludes: Vec<String>,
    work_dir: PathBuf,
    exclude_submodules: bool,
}

impl Linter {
    pub fn from_config(config: LinterConfig, global: &GlobalConfig) -> Self {
        Self {
            command: config.command,
            options: config.options,
            includes: config.includes,
            excludes: [global.excludes.clone(), config.excludes].concat(),
            work_dir: config.work_dir,
            exclude_submodules: config.exclude_submodules,
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
        let mut entries = Vec::new();
        for f in files {
            let metadata = fs::metadata(f.as_ref())?;
            entries.push(Entry {
                file: f.as_ref().to_owned(),
                modified: metadata.modified()?,
            });
        }

        let mut cmd = Xargs::new(&self.command, None);
        cmd.common_args(&self.options);
        for e in &entries {
            cmd.arg(&e.file);
        }
        if !self.work_dir.as_os_str().is_empty() {
            let work_dir = root.as_ref().join(&self.work_dir);
            ensure!(
                work_dir.is_dir(),
                "{} is not a directory",
                work_dir.display()
            );
            cmd.current_dir(work_dir);
        }
        let output = cmd.output()?;

        let mut modified = Vec::new();
        for e in &entries {
            let metadata = fs::metadata(&e.file)?;
            if e.modified < metadata.modified()? {
                modified.push(e.file.to_owned())
            }
        }
        debug!("modified: {:?}", &modified);

        Ok(Output {
            process: output,
            modified,
        })
    }

    pub fn run(&self, root: impl AsRef<Path>) -> Result<Option<Output>> {
        let files = self.paths(&root)?;
        if !self.includes.is_empty() && files.is_empty() {
            debug!("no files");
            return Ok(None);
        }

        Ok(Some(self.run_files(root, files)?))
    }

    fn paths(&self, root: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
        if self.includes.is_empty() {
            return Ok(vec![]);
        }

        let overrides = {
            let mut builder = OverrideBuilder::new(&root);
            for pattern in &self.includes {
                builder.add(&escape_pattern(pattern))?;
            }
            for pattern in &self.excludes {
                builder.add(&format!("!{}", escape_pattern(pattern)))?;
            }
            builder.build()?
        };

        let mut walk = WalkBuilder::new(&root);
        walk.hidden(false)
            .overrides(OverrideBuilder::new(&root).add("!.git/")?.build()?);
        if self.exclude_submodules {
            walk.filter_entry(|entry| {
                if let Some(file_type) = entry.file_type() {
                    // this method must cover most cases
                    if file_type.is_dir() && entry.path().join(".git").is_file() {
                        return false;
                    }
                }
                true
            });
        }
        Ok(walk
            .build()
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
                        return Some(
                            entry
                                .path()
                                .strip_prefix(".")
                                .unwrap_or_else(|_| entry.path())
                                .to_path_buf(),
                        );
                    }
                }
                None
            })
            .filter(|path| match overrides.matched(path, false) {
                Match::Whitelist(_) => true,
                Match::None => false,
                Match::Ignore(i) => {
                    debug!("ignoring {}: {:?}", path.display(), i);
                    false
                }
            })
            .collect())
    }
}

fn escape_pattern(glob: &str) -> String {
    if glob.starts_with('!') {
        format!("\\{}", glob)
    } else {
        glob.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::Linter;
    use crate::config::LinterConfig;
    use std::{
        default::Default,
        fs::{create_dir, read_to_string, File},
        io::Write,
        process,
    };
    use tempfile::tempdir;
    use test_log::test;

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
        assert!(output.success());
        assert_eq!(
            std::str::from_utf8(output.stdout()).unwrap().trim_end(),
            format!("option {}", root.path().join("main.rs").display())
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
        assert!(output.is_none());
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
        assert!(output.success());
        assert_eq!(std::str::from_utf8(output.stdout()).unwrap().trim_end(), "");
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
        assert!(output.success());
        assert!(std::str::from_utf8(output.stdout())
            .unwrap()
            .contains("main.rs"));
    }

    #[test]
    fn modified() {
        let root = tempdir().unwrap();
        let main = root.path().join("main.rs");
        {
            let mut file = File::create(&main).unwrap();
            write!(&mut file, " use std;").unwrap();
        }
        let linter = Linter::from_config(
            LinterConfig {
                command: "cargo".to_string(),
                options: vec!["fmt".to_string(), "--".to_string()],
                includes: vec!["*.rs".to_string()],
                ..Default::default()
            },
            &Default::default(),
        );
        let output = linter.run(&root).unwrap().unwrap();
        assert!(!output.success());
        assert!(read_to_string(&main).unwrap().starts_with("use std;"));
    }

    #[test]
    fn submodule() {
        let root = tempdir().unwrap();
        let git = |args: &[&str]| {
            assert!(process::Command::new("git")
                .current_dir(&root)
                .args(args)
                .output()
                .unwrap()
                .status
                .success());
        };
        git(&["init"]);
        git(&["config", "user.name", "test"]);
        git(&["config", "user.email", "test"]);
        File::create(root.path().join("main.rs")).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "init"]);
        git(&["submodule", "add", &root.path().display().to_string()]);
        let test = |exclude_submodules: bool, count: usize| {
            let linter = Linter::from_config(
                LinterConfig {
                    command: "ls".to_string(),
                    includes: vec!["*.rs".to_string()],
                    exclude_submodules,
                    ..Default::default()
                },
                &Default::default(),
            );
            let output = linter.run(&root).unwrap().unwrap();
            assert!(output.success());
            assert_eq!(
                std::str::from_utf8(output.stdout())
                    .unwrap()
                    .matches("main.rs")
                    .count(),
                count
            );
        };
        test(false, 2);
        test(true, 1);
    }
}
