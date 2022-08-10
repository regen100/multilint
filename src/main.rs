use anyhow::Result;
use colored::Colorize;
use log::debug;
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
        JSONL,
        GNU,
    }
}

#[derive(Debug, StructOpt)]
#[structopt(about, global_setting = clap::AppSettings::ColoredHelp)]
struct Opt {
    /// Changes the working directory before running
    #[structopt(short = "C")]
    pub work_dir: Option<PathBuf>,

    /// Config file
    #[structopt(short, long, parse(from_os_str), default_value = "multilint.toml")]
    config: PathBuf,

    /// Message format
    #[structopt(short, long, possible_values = &Printer::variants(), case_insensitive = true, default_value="text")]
    printer: Printer,
}

fn run() -> Result<()> {
    let opt = Opt::from_args();
    if let Some(work_dir) = &opt.work_dir {
        debug!("change CWD: {}", work_dir.display());
        env::set_current_dir(work_dir)?;
    }
    let printer: Box<dyn printer::Printer> = match opt.printer {
        Printer::Null => Box::new(printer::NullPrinter::default()),
        Printer::Text => Box::new(printer::TextPrinter::default()),
        Printer::JSONL => Box::new(printer::JSONLPrinter::default()),
        Printer::GNU => Box::new(printer::GNUPrinter::default()),
    };
    if !driver::run_linters(&opt.config, &*printer)? {
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
