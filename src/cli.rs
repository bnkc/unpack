use clap::Parser;

use anyhow::{anyhow, Result};
use std::path::Path;

use std::env;
use std::path::PathBuf;

const DEP_SPEC_FILES: [&str; 2] = ["requirements.txt", "pyproject.toml"];

#[derive(Parser)]
#[command(
    name = "pip-udeps",
    version,
    about = "A program to find unused dependencies in Python projects.",
    after_long_help = "Bugs can be reported on GitHub: https://github.com/bnkc/pip-udeps/issues",
    max_term_width = 98
)]
pub struct Opts {
    /// Change the working directory of pip-udeps to a provided path.
    /// This means that pip-udeps will search for unused dependencies with
    /// respect to the given `base` path.
    /// Note: If the base path provided does not contain a `poetry.toml`, or
    /// `requirements.txt` within the root of the path provided, operation will exit.
    #[arg(
        long,
        short = 'b',
        help = "The path to the directory to search for Python files.",
        default_value = ".",
        long_help
    )]
    #[arg(default_value = ".")]
    pub base_directory: PathBuf,

    /// Select the dependency status to search for.
    #[arg(
        long,
        short = 'd',
        value_name("STATUS"),
        default_value("unused"),
        value_enum
    )]
    pub dependency_status: PackageState,

    /// Include hidden directories and files in the search results (default:
    /// hidden files and directories are skipped). Files and directories are
    /// considered to be hidden if their name starts with a `.` sign (dot).
    #[arg(
        long,
        short = 'i',
        help = "Ignore hidden files and directories.",
        default_value = "true",
        long_help
    )]
    pub ignore_hidden: bool,

    /// The output format to use allows for the selection of the output format
    /// for the results of the unused dependencies search. The default output
    /// format is `human`. The `json` format is also available.
    #[arg(
        long,
        short = 'o',
        value_name("OUTPUT"),
        default_value("human"),
        value_enum,
        long_help
    )]
    pub output: OutputKind,
}

#[derive(clap::ValueEnum, Debug, PartialEq, Eq, Clone, Hash)]
pub enum PackageState {
    /// The dependency is installed, actively used in the project, and correctly listed in pyproject.toml.
    /// This state indicates a fully integrated and properly managed dependency.
    Used,
    /// The dependency is installed and listed in pyproject.toml but is not actively used in the project.
    /// Ideal for identifying and possibly removing unnecessary dependencies to clean up the project. (default)
    Unused,
    /// The dependency is installed and actively used in the project but is missing from pyproject.toml.
    /// Highlights dependencies that are implicitly used but not formally declared, which may lead to
    /// inconsistencies or issues in dependency management and deployment.
    Untracked,
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

    /// The dependency status to search for.
    /// Ex: `Unused`, `Untracked`, `Used`
    pub package_state: PackageState,

    /// The path to the dependency specification file.
    /// Ex: `requirements.txt` or `pyproject.toml`
    pub dep_spec_file: PathBuf,

    /// Whether to ignore hidden files and directories (or not).
    pub ignore_hidden: bool,

    /// The environment to run in.
    pub env: Env,

    /// The output format.
    /// Ex: `human` or `json`
    pub output: OutputKind,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum OutputKind {
    /// Human-readable output format.
    Human,
    /// JSON output format.
    Json,
}

impl Config {
    pub fn build(opts: Opts) -> Result<Config> {
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
