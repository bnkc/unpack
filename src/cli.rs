use clap::Parser;

use anyhow::{anyhow, Result};
use std::path::Path;

use std::env;
use std::path::PathBuf;

const DEP_SPEC_FILES: [&str; 2] = ["requirements.txt", "pyproject.toml"];

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

    /// Ignore hidden files and directories.
    /// This is useful when you want to ignore files like `.git` or `.venv`.
    /// By default, hidden files and directories are not ignored.
    #[arg(long, short = 'i', help = "Ignore hidden files and directories.")]
    pub ignore_hidden: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Env {
    Test,
    Dev,
    Prod,
}

pub struct Config {
    /// The path to the directory to search for Python files.
    pub base_directory: PathBuf,

    /// The path to the dependency specification file.
    /// Ex: `requirements.txt` or `pyproject.toml`
    pub dep_spec_file: PathBuf,

    /// Whether to ignore hidden files and directories (or not).
    pub ignore_hidden: bool,

    /// The environment to run in.
    pub env: Env,
}

impl Config {
    pub fn build(opts: Opts) -> Result<Config> {
        let base_directory = opts.base_directory;
        let dep_spec_file = get_dependency_specification_file(&base_directory)?;
        let ignore_hidden = opts.ignore_hidden;
        Ok(Config {
            base_directory,
            dep_spec_file,
            ignore_hidden,
            env: Env::Dev,
        })
    }
}

/// Get the dependency specification file from the base directory.
pub fn get_dependency_specification_file(base_dir: &Path) -> Result<PathBuf> {
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
