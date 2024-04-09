use std::path::PathBuf;

use crate::cli::{Env, OutputKind};
use crate::project_assets::PackageState;

pub struct Config {
    /// The path to the directory to search for Python files.
    pub base_directory: PathBuf,

    /// The package status to search for.
    /// Ex: `Unused`, `Untracked`, `Used`
    pub package_state: PackageState,

    /// The path to the dependency specification file.
    /// Ex: `requirements.txt` or `pyproject.toml`
    pub dep_spec_file: PathBuf,

    /// Whether to ignore hidden files and directories (or not).
    pub ignore_hidden: bool,

    /// The maximum search depth, or `None` if no maximum search depth should be set.
    ///
    /// A depth of `1` includes all files under the current directory, a depth of `2` also includes
    /// all files under subdirectories of the current directory, etc.
    pub max_depth: Option<usize>,

    /// The environment to run in.
    pub env: Env,

    /// The output format.
    /// Ex: `human` or `json`
    pub output: OutputKind,
}
