mod analysis;
mod cli;
mod config;
mod dependencies;
mod exit_codes;
mod imports;
mod packages;

use std::env;
use std::path::PathBuf;

use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;

// use crate::analysis::analyze;
// use crate::analysis::analyze;
use crate::cli::{Env, Opts};
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
    let config = construct_config(opts)?;

    set_working_dir(&config)?;

    analysis::scan(config)?;

    Ok(ExitCode::Success)

    // Ok(analysis)
}

fn construct_config(opts: Opts) -> Result<Config> {
    let base_directory = opts.base_directory;
    let dep_spec_file = get_dependency_specification_file(&base_directory)?;
    let ignore_hidden = opts.ignore_hidden;
    let output = opts.output;
    Ok(Config {
        base_directory,
        dep_spec_file,
        ignore_hidden,
        env: Env::Dev,
        output,
        package_state: opts.dependency_status,
    })
}

const DEP_SPEC_FILES: [&str; 2] = ["requirements.txt", "pyproject.toml"];

pub fn get_dependency_specification_file(base_dir: &PathBuf) -> Result<PathBuf> {
    let file = base_dir.ancestors().find_map(|dir| {
        DEP_SPEC_FILES
            .into_iter()
            .map(|file_name| dir.join(file_name))
            .find(|file_path| file_path.exists())
    });

    file.ok_or_else(|| {
        anyhow!(format!(
            "Could not find `Requirements.txt` or `pyproject.toml` in '{}' or any parent directory",
            env::current_dir().unwrap().to_string_lossy()
        ))
    })
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
