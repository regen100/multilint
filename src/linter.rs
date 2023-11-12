use crate::{
    config::{GlobalConfig, LinterConfig},
    xargs::Xargs,
};
use anyhow::{ensure, Result};
use digest;
use ignore::{overrides::OverrideBuilder, DirEntry, Match, WalkBuilder};
use log::{debug, warn};
use sha2::{Digest, Sha256};
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
    path: PathBuf,
    modified: SystemTime,
    hash: Option<digest::Output<Sha256>>,
}

impl Entry {
    fn new(path: impl AsRef<Path>, use_hash: bool) -> Result<Entry> {
        let metadata = fs::metadata(&path)?;
        let hash = if use_hash {
            Some(Sha256::digest(fs::read(&path)?))
        } else {
            None
        };
        Ok(Entry {
            path: path.as_ref().to_owned(),
            modified: metadata.modified()?,
            hash,
        })
    }

    fn is_same(&self) -> Result<bool> {
        if let Some(hash) = &self.hash {
            let new_hash = &Sha256::digest(fs::read(&self.path)?);
            return Ok(hash == new_hash);
        }

        let metadata = fs::metadata(&self.path)?;
        let modified = metadata.modified()?;
        Ok(self.modified == modified)
    }
}

#[derive(Debug, Clone)]
pub struct Linter {
    command: String,
    options: Vec<String>,
    includes: Vec<String>,
    excludes: Vec<String>,
    work_dir: PathBuf,
    exclude_submodules: bool,
    single_file: bool,
    check_hash: bool,
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
            single_file: config.single_file,
            check_hash: config.check_hash,
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
        let work_dir = if self.work_dir.as_os_str().is_empty() {
            None
        } else {
            Some(&self.work_dir)
        };

        let mut entries = Vec::new();
        for f in files {
            entries.push(Entry::new(f, self.check_hash)?);
        }

        let mut cmd = Xargs::new(&self.command, if self.single_file { Some(1) } else { None });
        cmd.common_args(&self.options);
        for e in &entries {
            let path = if work_dir.is_some() {
                fs::canonicalize(root.as_ref().join(&e.path))?.to_path_buf()
            } else {
                e.path.to_path_buf()
            };
            cmd.arg(path);
        }
        if let Some(work_dir) = &work_dir {
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
            if !e.is_same()? {
                modified.push(e.path.to_owned())
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
    fn work_dir() {
        let root = tempdir().unwrap();
        let subdir = root.path().join("sub");
        create_dir(&subdir).unwrap();
        File::create(subdir.join("main.rs")).unwrap();
        let linter = Linter::from_config(
            LinterConfig {
                command: "ls".to_string(),
                work_dir: subdir.clone(),
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

    #[cfg(unix)]
    #[test]
    fn hash() {
        let root = tempdir().unwrap();
        let main = root.path().join("main.rs");
        File::create(&main).unwrap();
        let linter = Linter::from_config(
            LinterConfig {
                command: "bash".to_string(),
                options: vec![
                    "-c".to_string(),
                    "sleep 0.1; touch $@".to_string(),
                    "--".to_string(),
                ],
                includes: vec!["*.rs".to_string()],
                check_hash: true,
                ..Default::default()
            },
            &Default::default(),
        );
        let output = linter.run(&root).unwrap().unwrap();
        assert!(output.success());
    }

    #[test]
    fn submodule() {
        let root = tempdir().unwrap();
        let git = |args: &[&str]| {
            let cmd = process::Command::new("git")
                .current_dir(&root)
                .args(["-c", "protocol.file.allow=always"])
                .args(args)
                .output()
                .unwrap();
            assert!(cmd.status.success(), "{:?}", cmd);
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
