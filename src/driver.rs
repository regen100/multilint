use crate::{config, linter::Linter};
use anyhow::Result;
use colored::*;
use std::{
    io::{stdout, Write},
    path::Path,
};

pub fn run_linters(config_path: impl AsRef<Path>) -> Result<bool> {
    let config = config::from_path(&config_path)?;
    let global = config.global.unwrap_or_default();
    let root = config_path.as_ref().parent().unwrap();
    let mut ok = true;
    for (name, linter_config) in &config.linter {
        print!("{} {} ... ", "Running".bold().green(), &name);
        let linter = Linter::from_config(linter_config.clone(), &global);
        if !linter.is_executable() {
            println!("{}", "no command".yellow());
            continue;
        }
        match linter.run(&root)? {
            Some(output) => {
                if output.status.success() {
                    println!("{}", "ok".green());
                } else {
                    println!("{}", "failed".red());
                }
                if !output.stdout.is_empty() {
                    stdout().write_all(&output.stdout)?;
                }
                if !output.stderr.is_empty() {
                    stdout().write_all(&output.stderr)?;
                }
                ok &= output.status.success();
            }
            None => println!("{}", "skipped".yellow()),
        }
    }
    Ok(ok)
}

#[cfg(test)]
mod tests {
    use super::run_linters;
    use std::{fs::File, io::Write};
    use tempfile::tempdir;
    use test_log::test;

    #[test]
    fn run() {
        let root = tempdir().unwrap();
        let config = root.path().join("config.toml");

        {
            let mut config = File::create(&config).unwrap();
            writeln!(config, "[linter.test]").unwrap();
            writeln!(config, "command = 'true'").unwrap();
            writeln!(config, "includes = ['*']").unwrap();
        }
        assert!(run_linters(&config).unwrap());

        {
            let mut config = File::create(&config).unwrap();
            writeln!(config, "[linter.test]").unwrap();
            writeln!(config, "command = 'false'").unwrap();
            writeln!(config, "includes = ['*']").unwrap();
        }
        assert!(!run_linters(&config).unwrap());
    }
}
