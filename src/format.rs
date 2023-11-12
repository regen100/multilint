use anyhow::Result;
use std::io::{stdout, Write};

use colored::*;

use crate::linter::Output;

pub trait OutputFormat {
    fn start(&self, name: &str);
    fn no_command(&self, name: &str);
    fn no_file(&self, name: &str);
    fn status(&self, name: &str, output: &Output) -> Result<()>;
}

#[derive(Default)]
pub struct NullFormat {}

impl OutputFormat for NullFormat {
    fn start(&self, _name: &str) {}
    fn no_command(&self, _name: &str) {}
    fn no_file(&self, _name: &str) {}
    fn status(&self, _name: &str, _output: &Output) -> Result<()> {
        Ok(())
    }
}

#[derive(Default)]
pub struct TextFormat {}

impl OutputFormat for TextFormat {
    fn start(&self, name: &str) {
        print!("{} {} ... ", "Running".bold().green(), &name);
    }

    fn no_command(&self, _name: &str) {
        println!("{}", "no command".yellow());
    }

    fn no_file(&self, _name: &str) {
        println!("{}", "skipped".yellow());
    }

    fn status(&self, _name: &str, output: &Output) -> Result<()> {
        if output.success() {
            println!("{}", "ok".green());
        } else {
            println!("{}", "failed".red());
        }
        stdout().write_all(output.stdout())?;
        stdout().write_all(output.stderr())?;
        for f in output.modified() {
            println!("{}: modified", f.display());
        }
        Ok(())
    }
}
