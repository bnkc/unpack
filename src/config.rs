use std::path::PathBuf;

use crate::cli::{Env, OutputKind};
use crate::packages::PackageState;

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
