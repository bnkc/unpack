// another name could be prune-deps
// or prune-udeps
// or prunes-rs

mod exit_codes;

use crate::exit_codes::ExitCode;
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use std::env;

use pip_udeps::get_used_dependencies;
use std::path::PathBuf;

#[derive(Parser, Debug)]
// #[command(version, about, long_about = None)]
#[command(
    name = "pip-udeps",
    version,
    about = "A simple tool to find and prune unused dependencies in a Python project.",
    after_long_help = "Bugs can be reported on GitHub: https://github.com/bnkc/pip-udeps/issues",
    max_term_width = 98
)]
pub struct Opts {
    /// Change the working directory of pip-udeps to a provided path. This
    /// means that pip-udeps will search for unused dependencies with respect to the given base path.
    /// Note that if the base path provided does not contain a poetry.toml, requirements.txt, etc
    /// within the root of the path provided, operation will exit.
    #[arg(
        long,
        short = 'b',
        help = "The path to the directory to search for Python files.",
        default_value = ".",
        long_help
    )]
    #[arg(default_value = ".")]
    pub base_directory: PathBuf,
}

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

    // set_project_dir(&opts)?;

    // let packages = todo!("get packages");
    let val = pip_udeps::get_packages_from_pyproject_toml();
    println!("{:#?}", val);

    let used_dependencies = get_used_dependencies(&opts.base_directory);

    // // this is temporary
    Ok(ExitCode::HasResults(used_dependencies?.is_empty()))
}

fn set_project_dir(opts: &Opts) -> Result<()> {
    if !opts.base_directory.exists() {
        return Err(anyhow!("The provided path does not exist."));
    } else if !opts.base_directory.is_dir() {
        return Err(anyhow!("The provided path is not a directory."));
    } else if !pip_udeps::check_for_dependency_specification_files(&opts.base_directory) {
        return Err(anyhow!(format!(
            "Could not find `Requirements.txt` or `pyproject.toml` in '{}' or any parent directory",
            env::current_dir()?.to_string_lossy()
        )));
    }
    env::set_current_dir(&opts.base_directory).with_context(|| {
        format!(
            "Could not set '{}' as the current working directory. Please check the path provided.",
            // Not a fan of the unwrap here!!!
            env::current_dir().unwrap().to_string_lossy()
        )
    })?;

    Ok(())
}
