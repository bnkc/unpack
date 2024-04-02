use std::env;
use std::path::PathBuf;

use anyhow::{anyhow, Result};

use crate::cli::{Env, Opts, OutputKind};
use crate::packages::PackageState;

const DEP_SPEC_FILES: [&str; 2] = ["requirements.txt", "pyproject.toml"];

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
