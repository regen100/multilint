use anyhow::Result;
use colored::Colorize;
use log::debug;
use multilint::{driver, format};
use std::{env, path::PathBuf, process::exit};
use structopt::clap::arg_enum;
use structopt::{clap, StructOpt};

arg_enum! {
    #[derive(Debug)]
    enum Format {
        Null,
        Raw,
        Text,
    }
}

#[derive(Debug, StructOpt)]
#[structopt(about, global_setting = clap::AppSettings::ColoredHelp)]
struct Opt {
    /// Changes the working directory before running
    #[structopt(short = "C")]
    pub work_dir: Option<PathBuf>,

    /// Output format
    #[structopt(short, long, possible_values = &Format::variants(), case_insensitive = true, default_value="text")]
    format: Format,

    /// Linters to run
    #[structopt(short, long = "linter")]
    linters: Option<Vec<String>>,
}

fn run() -> Result<()> {
    let opt = Opt::from_args();
    if let Some(work_dir) = &opt.work_dir {
        debug!("change CWD: {}", work_dir.display());
        env::set_current_dir(work_dir)?;
    }
    let format: Box<dyn format::OutputFormat> = match opt.format {
        Format::Null => Box::<format::NullFormat>::default(),
        Format::Raw => Box::<format::RawFormat>::default(),
        Format::Text => Box::<format::TextFormat>::default(),
    };
    if !driver::run_linters(env::current_dir()?, &*format, opt.linters.as_deref())? {
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
