use anyhow::Result;
use std::io::{stdout, Write};

use colored::*;

use crate::{
    linter::Output,
    parser::{Parsed, Parser},
};

fn parse(parser: &Parser, name: &str, output: &Output) -> Result<Vec<Parsed>> {
    let mut msgs = parser.parse(std::str::from_utf8(output.stdout())?);
    for f in output.modified() {
        msgs.push(Parsed {
            file: Some(f.display().to_string()),
            message: Some("modified".to_string()),
            ..Default::default()
        })
    }
    for msg in &mut msgs {
        msg.program.get_or_insert_with(|| name.to_string());
    }
    Ok(msgs)
}

pub trait OutputFormat {
    fn start(&self, name: &str);
    fn no_command(&self, name: &str);
    fn no_file(&self, name: &str);
    fn status(&self, name: &str, output: &Output, parser: &Parser) -> Result<()>;
}

#[derive(Default)]
pub struct NullFormat {}

impl OutputFormat for NullFormat {
    fn start(&self, _name: &str) {}
    fn no_command(&self, _name: &str) {}
    fn no_file(&self, _name: &str) {}
    fn status(&self, _name: &str, _output: &Output, _parser: &Parser) -> Result<()> {
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

    fn status(&self, _name: &str, output: &Output, _parser: &Parser) -> Result<()> {
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

#[derive(Default)]
pub struct JSONLFormat {}

impl OutputFormat for JSONLFormat {
    fn start(&self, _name: &str) {}
    fn no_command(&self, _name: &str) {}
    fn no_file(&self, _name: &str) {}

    fn status(&self, name: &str, output: &Output, parser: &Parser) -> Result<()> {
        let msgs = parse(parser, name, output)?;
        for msg in msgs {
            println!("{}", serde_json::to_string(&msg)?);
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct GNUFormat {}

impl OutputFormat for GNUFormat {
    fn start(&self, _name: &str) {}
    fn no_command(&self, _name: &str) {}
    fn no_file(&self, _name: &str) {}

    fn status(&self, name: &str, output: &Output, parser: &Parser) -> Result<()> {
        let msgs = parse(parser, name, output)?;
        for msg in msgs {
            if let Some(program) = msg.program {
                print!("{}:", program);
            }
            if let Some(file) = msg.file {
                print!("{}:", file);
            }
            if let Some(line) = msg.line {
                print!("{}:", line);
            }
            if let Some(column) = msg.column {
                print!("{}:", column);
            }
            if let Some(message) = msg.message {
                print!(" {}", message);
            }
            println!();
        }
        Ok(())
    }
}
