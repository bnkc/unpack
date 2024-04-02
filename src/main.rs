mod analysis;
mod cli;
mod config;
mod dependencies;
mod exit_codes;
mod imports;
mod packages;

use std::env;

use anyhow::{bail, Context, Result};
use clap::Parser;

// use crate::analysis::analyze;
// use crate::analysis::analyze;
use crate::cli::Opts;
use crate::config::Config;
use crate::exit_codes::ExitCode;

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

    set_working_dir(&config)?;

    analysis::scan(config)?;

    Ok(ExitCode::Success)

    // Ok(analysis)
}

fn set_working_dir(config: &Config) -> Result<()> {
    if !config.base_directory.exists() {
        bail!("The provided path does not exist.");
    } else if !config.base_directory.is_dir() {
        bail!("The provided path is not a directory.");
    }
    env::set_current_dir(&config.base_directory).with_context(|| {
        format!(
            "Could not set '{}' as the current working directory. Please check the path provided.",
            env::current_dir().unwrap().to_string_lossy()
        )
    })?;

    Ok(())
}
