use std::{
    io::{stdout, Write},
    process::Output,
};

use colored::*;

pub trait Printer {
    fn start(&self, name: &str);
    fn no_command(&self);
    fn no_file(&self);
    fn status(&self, output: &Output);
}

#[derive(Default)]
pub struct NullPrinter {}

impl Printer for NullPrinter {
    fn start(&self, _name: &str) {}
    fn no_command(&self) {}
    fn no_file(&self) {}
    fn status(&self, _output: &Output) {}
}

#[derive(Default)]
pub struct TextPrinter {}

impl Printer for TextPrinter {
    fn start(&self, name: &str) {
        print!("{} {} ... ", "Running".bold().green(), &name);
    }

    fn no_command(&self) {
        println!("{}", "no command".yellow());
    }

    fn no_file(&self) {
        println!("{}", "skipped".yellow());
    }

    fn status(&self, output: &Output) {
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
    }
}
