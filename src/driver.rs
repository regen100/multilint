use crate::{config, linter::Linter, parser::Parser, printer::Printer};
use anyhow::Result;
use std::path::Path;

pub fn run_linters(config_path: impl AsRef<Path>, printer: &dyn Printer) -> Result<bool> {
    let config = config::from_path(&config_path)?;
    let global = config.global.unwrap_or_default();
    let mut ok = true;
    for (name, linter_config) in &config.linter {
        printer.start(name);
        let linter = Linter::from_config(linter_config.clone(), &global);
        if !linter.is_executable() {
            printer.no_command(name);
            continue;
        }
        let parser = Parser::new(&linter_config.formats)?;
        match linter.run(".")? {
            None => printer.no_file(name),
            Some(output) => {
                printer.status(name, &output, &parser)?;
                ok &= output.status.success();
            }
        }
    }
    Ok(ok)
}

#[cfg(test)]
mod tests {
    use crate::printer::TextPrinter;

    use super::run_linters;
    use std::{fs::File, io::Write};
    use tempfile::tempdir;
    use test_log::test;

    #[test]
    fn run() {
        let root = tempdir().unwrap();
        let config = root.path().join("config.toml");
        let printer = TextPrinter::default();

        {
            let mut config = File::create(&config).unwrap();
            writeln!(config, "[linter.test]").unwrap();
            writeln!(config, "command = 'true'").unwrap();
            writeln!(config, "includes = ['*']").unwrap();
        }
        assert!(run_linters(&config, &printer).unwrap());

        {
            let mut config = File::create(&config).unwrap();
            writeln!(config, "[linter.test]").unwrap();
            writeln!(config, "command = 'false'").unwrap();
            writeln!(config, "includes = ['*']").unwrap();
        }
        assert!(!run_linters(&config, &printer).unwrap());
    }
}
