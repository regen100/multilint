use anyhow::Result;
use colored::Colorize;
use multilint::driver;
use std::{env, path::PathBuf, process::exit};
use structopt::clap;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about, global_setting = clap::AppSettings::ColoredHelp)]
struct Opt {
    /// Run linters at the directory
    #[structopt(short = "C", default_value = ".")]
    pub work_dir: PathBuf,

    /// Config file
    #[structopt(short, long, parse(from_os_str), default_value = "multilint.toml")]
    config: PathBuf,
}

fn run() -> Result<()> {
    let opt = Opt::from_args();
    env::set_current_dir(&opt.work_dir)?;
    let config_path = opt.work_dir.join(&opt.config);
    if !driver::run_linters(config_path)? {
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
