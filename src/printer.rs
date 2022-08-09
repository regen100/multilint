use anyhow::Result;
use std::{
    io::{stdout, Write},
    process::Output,
};

use colored::*;

use crate::parser::Parser;

pub trait Printer {
    fn start(&self, name: &str);
    fn no_command(&self, name: &str);
    fn no_file(&self, name: &str);
    fn status(&self, name: &str, output: &Output, parser: &Parser) -> Result<()>;
}

#[derive(Default)]
pub struct NullPrinter {}

impl Printer for NullPrinter {
    fn start(&self, _name: &str) {}
    fn no_command(&self, _name: &str) {}
    fn no_file(&self, _name: &str) {}
    fn status(&self, _name: &str, _output: &Output, _parser: &Parser) -> Result<()> {
        Ok(())
    }
}

#[derive(Default)]
pub struct TextPrinter {}

impl Printer for TextPrinter {
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
        if output.status.success() {
            println!("{}", "ok".green());
        } else {
            println!("{}", "failed".red());
        }
        if !output.stdout.is_empty() {
            stdout().write_all(&output.stdout).unwrap();
        }
        if !output.stderr.is_empty() {
            stdout().write_all(&output.stderr).unwrap();
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct JSONLPrinter {}

impl Printer for JSONLPrinter {
    fn start(&self, _name: &str) {}
    fn no_command(&self, _name: &str) {}
    fn no_file(&self, _name: &str) {}

    fn status(&self, name: &str, output: &Output, parser: &Parser) -> Result<()> {
        let msgs = parser.parse(std::str::from_utf8(&output.stdout)?);
        for mut msg in msgs {
            msg.program.get_or_insert_with(|| name.to_string());
            println!("{}", serde_json::to_string(&msg)?);
        }
        Ok(())
    }
}
