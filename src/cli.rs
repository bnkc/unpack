use clap::Parser;

use std::path::PathBuf;

use crate::packages::PackageState;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Env {
    Test,
    Dev,
    Prod,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum OutputKind {
    /// Human-readable output format.
    Human,
    /// JSON output format.
    Json,
}
