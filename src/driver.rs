use crate::{config, format::OutputFormat, linter::Linter};
use anyhow::Result;
use std::path::Path;

pub fn run_linters(
    config_path: impl AsRef<Path>,
    format: &dyn OutputFormat,
    linters: Option<&[String]>,
) -> Result<bool> {
    let config = config::from_path(&config_path)?;
    let mut ok = true;
    for (name, linter_config) in &config.linter {
        if let Some(linters) = linters {
            if !linters.contains(name) {
                continue;
            }
        }
        format.start(name);
        let linter = Linter::from_config(linter_config.clone(), &config.global);
        if !linter.is_executable() {
            format.no_command(name);
            continue;
        }
        match linter.run(".")? {
            None => format.no_file(name),
            Some(output) => {
                format.status(name, &output)?;
                ok &= output.success();
            }
        }
    }
    Ok(ok)
}

#[cfg(test)]
mod tests {
    use crate::format::TextFormat;

    use super::run_linters;
    use std::{fs::File, io::Write};
    use tempfile::tempdir;
    use test_log::test;

    #[test]
    fn run() {
        let root = tempdir().unwrap();
        let config = root.path().join("multilint.toml");
        let format = TextFormat::default();

        {
            let mut config = File::create(&config).unwrap();
            writeln!(config, "[linter.test]").unwrap();
            writeln!(config, "command = 'true'").unwrap();
            writeln!(config, "includes = ['*']").unwrap();
        }
        assert!(run_linters(&root.path(), &format, None).unwrap());

        {
            let mut config = File::create(&config).unwrap();
            writeln!(config, "[linter.test]").unwrap();
            writeln!(config, "command = 'false'").unwrap();
            writeln!(config, "includes = ['*']").unwrap();
        }
        assert!(!run_linters(&root.path(), &format, None).unwrap());
    }

    #[test]
    fn run_selected() {
        let root = tempdir().unwrap();
        let config = root.path().join("multilint.toml");
        let format = TextFormat::default();

        {
            let mut config = File::create(&config).unwrap();
            writeln!(config, "[linter.test]").unwrap();
            writeln!(config, "command = 'false'").unwrap();
            writeln!(config, "includes = ['*']").unwrap();
        }
        assert!(run_linters(&root.path(), &format, Some(&[])).unwrap());
    }
}
