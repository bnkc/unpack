// another name could be prune-deps
// or prune-udeps
// or prunes-rs

mod exit_codes;

use crate::exit_codes::ExitCode;
use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;
use std::env;

use pip_udeps::get_deps;
use std::path::{Path, PathBuf};

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
        short = 'P',
        help = "The path to the directory to search for Python files.",
        default_value = ".",
        long_help
    )]
    #[arg(default_value = ".")]
    pub base_directory: Option<PathBuf>,
}

fn main() {
    let _result = run();
    // match result {
    //     Ok(exit_code) => {
    //         exit_code.exit();
    //     }
    //     Err(err) => {
    //         eprintln!("[fd error]: {:#}", err);
    //         ExitCode::GeneralError.exit();
    //     }
    // }
}

fn run() -> Result<ExitCode> {
    let opts = Opts::parse();

    set_working_dir(&opts)?;

    let deps = get_deps(&opts.base_directory);
    println!("{:?}", deps);

    // this is temporary
    Ok(ExitCode::HasResults(deps?.is_empty()))
}

fn set_working_dir(opts: &Opts) -> Result<()> {
    if let Some(ref base_directory) = opts.base_directory {
        if !base_directory.exists() {
            return Err(anyhow!("The provided path does not exist."));
        }
        if !base_directory.is_dir() {
            return Err(anyhow!("The provided path is not a directory."));
        }
        env::set_current_dir(base_directory).with_context(|| {
            format!(
                "Could not set '{}' as the current working directory",
                base_directory.to_string_lossy()
            )
        })?;
    }
    Ok(())
}

// fn is_existing_dir(path: &Path) -> bool {
//     path.is_dir()
// }
