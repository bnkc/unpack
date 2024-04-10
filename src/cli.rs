use clap::Parser;

use std::path::PathBuf;

use crate::project_assets::PackageState;

#[derive(Parser)]
#[command(
    name = "unpack",
    version,
    about = "Unpack is a simple, fast and user-friendly tool to analyze python project packaging.",
    after_long_help = "Bugs can be reported on GitHub: https://github.com/bnkc/unpack/issues",
    max_term_width = 98
)]
pub struct Opts {
    /// Change the working directory of unpack to a provided path.
    /// This means that unpack will search for unused packages with
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

    /// Select the packages status to search for.
    #[arg(
        long,
        short = 's',
        value_name("STATUS"),
        default_value("unused"),
        value_enum
    )]
    pub package_status: PackageState,

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

    /// Limit the directory traversal to a given depth. By default, there is no
    /// limit on the search depth.
    #[arg(
        long,
        short = 'd',
        value_name("DEPTH"),
        alias("maxdepth"),
        help = "Set maximum search depth (default: none)",
        long_help
    )]
    max_depth: Option<usize>,

    /// The output format to use allows for the selection of the output format
    /// for the results of the unused packages search. The default output
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

    /// Select the depencency specification file of choice if more than one exists.
    /// By default, `pyproject.toml` is selected
    #[arg(
        long,
        short = 't',
        value_name("DEP_TYPE"),
        default_value("poetry"),
        value_enum,
        long_help
    )]
    pub dep_type: DepType,
}

impl Opts {
    pub fn max_depth(&self) -> Option<usize> {
        self.max_depth
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Env {
    #[allow(dead_code)]
    Test,
    Dev,
    #[allow(dead_code)]
    Prod,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum OutputKind {
    /// Human-readable output format.
    Human,
    /// JSON output format.
    Json,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum DepType {
    /// requirements.txt
    Pip,
    /// pyproject.toml
    Poetry,
}
