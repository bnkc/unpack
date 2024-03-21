mod cli;
mod exit_codes;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use std::env;

use crate::cli::{Config, Opts};
use crate::exit_codes::ExitCode;

use pip_udeps::get_unused_dependencies;

fn main() {
    let result = run();
    match result {
        Ok(exit_code) => {
            exit_code.exit();
        }
        Err(err) => {
            eprintln!("[pip-udeps error]: {:#}", err);
            ExitCode::GeneralError.exit();
        }
    }
}

fn run() -> Result<ExitCode> {
    let opts = Opts::parse();
    let config = Config::build(opts)?;
    set_working_dir(&opts)?;
    get_unused_dependencies(&config, std::io::stdout())?;

    // This is a hack. I need to decide if I want to move everything to the library or not.
    Ok(ExitCode::Success)
}

fn set_working_dir(opts: &Opts) -> Result<()> {
    if !opts.base_directory.exists() {
        return Err(anyhow!("The provided path does not exist."));
    } else if !opts.base_directory.is_dir() {
        return Err(anyhow!("The provided path is not a directory."));
    }
    env::set_current_dir(&opts.base_directory).with_context(|| {
        format!(
            "Could not set '{}' as the current working directory. Please check the path provided.",
            env::current_dir().unwrap().to_string_lossy()
        )
    })?;

    Ok(())
}
