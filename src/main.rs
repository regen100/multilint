use anyhow::Result;
use colored::Colorize;
use multilint::printer::{NullPrinter, TextPrinter};
use multilint::{driver, printer};
use std::{env, path::PathBuf, process::exit};
use structopt::clap;
use structopt::clap::arg_enum;
use structopt::StructOpt;

arg_enum! {
    #[derive(Debug)]
    enum Printer {
        Null,
        Text,
    }
}

#[derive(Debug, StructOpt)]
#[structopt(about, global_setting = clap::AppSettings::ColoredHelp)]
struct Opt {
    /// Run linters at the directory
    #[structopt(short = "C", default_value = ".")]
    pub work_dir: PathBuf,

    /// Config file
    #[structopt(short, long, parse(from_os_str), default_value = "multilint.toml")]
    config: PathBuf,

    /// Message format
    #[structopt(short, long, possible_values = &Printer::variants(), case_insensitive = true, default_value="text")]
    printer: Printer,
}

fn run() -> Result<()> {
    let opt = Opt::from_args();
    env::set_current_dir(&opt.work_dir)?;
    let config_path = opt.work_dir.join(&opt.config);
    let printer: Box<dyn printer::Printer> = match opt.printer {
        Printer::Null => Box::new(NullPrinter::default()),
        Printer::Text => Box::new(TextPrinter::default()),
    };
    if !driver::run_linters(config_path, &*printer)? {
        exit(1);
    }
    Ok(())
}

fn main() {
    env_logger::init();
    if let Err(e) = run() {
        eprintln!("{} {:#}", "error:".red().bold(), e);
        exit(2);
    }
}
