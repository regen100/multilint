use crate::{config, format::OutputFormat, linter::Linter, parser::Parser};
use anyhow::Result;
use std::path::Path;

pub fn run_linters(config_path: impl AsRef<Path>, format: &dyn OutputFormat) -> Result<bool> {
    let config = config::from_path(&config_path)?;
    let mut ok = true;
    for (name, linter_config) in &config.linter {
        format.start(name);
        let linter = Linter::from_config(linter_config.clone(), &config.global);
        if !linter.is_executable() {
            format.no_command(name);
            continue;
        }
        let parser = Parser::new(&linter_config.formats)?;
        match linter.run(".")? {
            None => format.no_file(name),
            Some(output) => {
                format.status(name, &output, &parser)?;
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
        let config = root.path().join("config.toml");
        let format = TextFormat::default();

        {
            let mut config = File::create(&config).unwrap();
            writeln!(config, "[linter.test]").unwrap();
            writeln!(config, "command = 'true'").unwrap();
            writeln!(config, "includes = ['*']").unwrap();
        }
        assert!(run_linters(&config, &format).unwrap());

        {
            let mut config = File::create(&config).unwrap();
            writeln!(config, "[linter.test]").unwrap();
            writeln!(config, "command = 'false'").unwrap();
            writeln!(config, "includes = ['*']").unwrap();
        }
        assert!(!run_linters(&config, &format).unwrap());
    }
}
