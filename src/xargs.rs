use anyhow::Result;
use argmax;
use log::debug;
use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    process,
};

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
#[cfg(windows)]
use std::os::windows::process::ExitStatusExt;

pub struct Xargs {
    program: OsString,
    max_args: Option<usize>,
    common_args: Vec<OsString>,
    args: Vec<OsString>,
    current_dir: Option<PathBuf>,
}

impl Xargs {
    pub fn new(program: impl AsRef<OsStr>, max_args: Option<usize>) -> Self {
        Self {
            program: program.as_ref().to_os_string(),
            max_args,
            common_args: vec![],
            args: vec![],
            current_dir: None,
        }
    }

    pub fn common_arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
        self.common_args(&[arg])
    }

    pub fn common_args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        for a in args {
            self.common_args.push(a.as_ref().to_os_string());
        }
        self
    }

    pub fn arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
        self.args(&[arg])
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        for a in args {
            self.args.push(a.as_ref().to_os_string());
        }
        self
    }

    pub fn current_dir(&mut self, dir: impl AsRef<Path>) -> &mut Self {
        self.current_dir = Some(dir.as_ref().to_owned());
        self
    }

    pub fn output(&self) -> Result<process::Output> {
        let mut ret = process::Output {
            status: process::ExitStatus::from_raw(0),
            stdout: Vec::new(),
            stderr: Vec::new(),
        };

        let mut args: &[OsString] = &self.args;
        loop {
            let mut cmd = argmax::Command::new(&self.program);
            let mut debug_cmd = vec![&self.program];
            if let Some(dir) = &self.current_dir {
                cmd.current_dir(dir);
            }
            cmd.try_args(&self.common_args)?;
            debug_cmd.extend(&self.common_args);
            if !args.is_empty() {
                cmd.try_arg(&args[0])?;
                debug_cmd.push(&args[0]);
                let max_args = std::cmp::min(args.len(), self.max_args.unwrap_or(args.len()));
                let mut i = 1;
                while i < max_args {
                    if cmd.try_arg(&args[i]).is_err() {
                        break;
                    }
                    debug_cmd.push(&args[i]);
                    i += 1;
                }
                args = &args[i..];
            }
            debug!("command: {:?}", debug_cmd);
            let output = cmd.output()?;
            if !output.status.success() {
                // https://man.archlinux.org/man/xargs.1.en#EXIT_STATUS
                ret.status = process::ExitStatus::from_raw(123);
            }
            ret.stdout.extend(output.stdout);
            ret.stderr.extend(output.stderr);

            if args.is_empty() {
                break;
            }
        }
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::Xargs;
    use test_log::test;

    #[test]
    fn no_arg() {
        let output = Xargs::new("echo", Some(2))
            .common_arg("c")
            .output()
            .unwrap();
        assert!(output.status.success());
        let stdout = std::str::from_utf8(&output.stdout).unwrap();
        assert!(stdout.contains("c"));
    }

    #[test]
    fn max_args() {
        let output = Xargs::new("echo", Some(2))
            .common_arg("c")
            .args(["1", "2", "3"])
            .output()
            .unwrap();
        assert!(output.status.success());
        let stdout = std::str::from_utf8(&output.stdout).unwrap();
        assert!(stdout.contains("c 1 2"));
        assert!(stdout.contains("c 3"));
    }
}
