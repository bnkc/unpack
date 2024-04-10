mod analyze;
mod cli;
mod config;
mod exit_codes;
mod output;
mod project_assets;

use std::env;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;

use crate::cli::{DepType, Env, Opts};
use crate::config::Config;
use crate::exit_codes::ExitCode;

const DEP_SPEC_FILES: [&str; 2] = ["requirements.txt", "pyproject.toml"];

fn main() {
    let result = run();
    match result {
        Ok(exit_code) => {
            exit_code.exit();
        }
        Err(err) => {
            eprintln!("[unpack error]: {:#}", err);
            ExitCode::GeneralError.exit();
        }
    }
}

fn run() -> Result<ExitCode> {
    let opts = Opts::parse();

    let config = construct_config(opts)?;

    set_working_dir(&config)?;

    analyze::scan(config)
}

fn construct_config(opts: Opts) -> Result<Config> {
    let base_directory = &opts.base_directory;
    let dep_type = opts.dep_type;
    let dep_files = get_dependency_spec_files(base_directory)?;
    let dep_spec_file = match opts.dep_type {
        DepType::Pip => dep_files
            .iter()
            .find(|file| file.ends_with("requirements.txt"))
            .ok_or_else(|| anyhow!("Could not find `requirements.txt` in the provided directory."))?
            .to_owned(),
        DepType::Poetry => dep_files
            .iter()
            .find(|file| file.ends_with("pyproject.toml"))
            .ok_or_else(|| anyhow!("Could not find `pyproject.toml` in the provided directory."))?
            .to_owned(),
    };

    let ignore_hidden = opts.ignore_hidden;
    let output = opts.output;
    let max_depth = opts.max_depth();
    Ok(Config {
        base_directory: base_directory.to_owned(),
        dep_spec_file,
        dep_type,
        ignore_hidden,
        max_depth,
        env: Env::Dev,
        output,
        package_state: opts.package_status,
    })
}

pub fn get_dependency_spec_files(base_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for dir in base_dir.ancestors() {
        for file_name in DEP_SPEC_FILES.iter() {
            let file_path = dir.join(file_name);
            if file_path.exists() {
                files.push(file_path);
            }
        }
    }

    if files.is_empty() {
        Err(anyhow!(format!(
            "Could not find `Requirements.txt` or `pyproject.toml` in '{}' or any parent directory",
            env::current_dir().unwrap().to_string_lossy()
        )))
    } else {
        Ok(files)
    }
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
