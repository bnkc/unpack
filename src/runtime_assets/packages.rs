extern crate bytesize;
extern crate fs_extra;
// extern crate test;

use std::collections::HashSet;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::str;

use anyhow::{anyhow, bail};
use anyhow::{Context, Result};
use fs_extra::dir::get_size;
use glob::glob;
use serde::{Deserialize, Serialize};

// use crate::dependencies::Dependency;

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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Package {
    id: String,
    size: u64,
    aliases: HashSet<String>,
}

impl Hash for Package {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        // Optionally hash other fields that implement Hash and contribute to uniqueness
        // Do NOT hash the `aliases` field as HashSet<String> does not implement Hash
    }
}

impl Package {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn aliases(&self) -> &HashSet<String> {
        &self.aliases
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct PackageBuilder {
    id: String,
    size: u64,
    aliases: HashSet<String>,
}

impl PackageBuilder {
    pub fn new(id: String, aliases: HashSet<String>, size: u64) -> Self {
        Self { id, size, aliases }
    }

    pub fn build(mut self) -> Package {
        self.id = self.id.replace("_", "-");
        Package {
            id: self.id,
            size: self.size,
            aliases: self.aliases,
        }
    }
}

/// This method executes the command `python -m site` to get the site package directory
pub fn get_site_packages() -> Result<HashSet<PathBuf>> {
    let output = Command::new("python")
        .arg("-m")
        .arg("site")
        .output()
        .context("Failed to execute `python -m site`. Are you sure Python is installed?")?;

    let output_str = str::from_utf8(&output.stdout)
        .context("Output was not valid UTF-8.")?
        .trim();

    // Extract the site package paths from the output
    let pkg_paths: HashSet<PathBuf> = output_str
        .lines()
        .filter(|line| line.contains("site-packages"))
        .map(|s| s.trim_matches(|c: char| c.is_whitespace() || c == '\'' || c == ','))
        .map(PathBuf::from)
        .collect();

    if pkg_paths.is_empty() {
        bail!("No site-packages found. Are you sure you are in a virtual environment?");
    }

    Ok(pkg_paths)
}

/// This function loads the packages from the specified site packages directory.
/// It takes a `SitePackages` object as input and returns a `Result` indicating success or failure.
pub fn get_packages(site_packages: HashSet<PathBuf>) -> Result<HashSet<Package>> {
    let mut packages = HashSet::new();

    for path in site_packages {
        // There is also a `*.egg-info` directory that we will ignore for now
        let glob_pattern = format!("{}/{}dist-info", path.display(), "*");

        // Iterate over each entry that matches the glob pattern.
        for entry in glob(&glob_pattern)?.filter_map(Result::ok) {
            // Read the metadata file for the package.
            let metadata_path = entry.join("METADATA");
            let metadata_content = fs::read_to_string(&metadata_path)
                .with_context(|| format!("Failed to read METADATA in {:?}", entry))?;

            // Extract the package `id` from the metadata.
            let pkg_id = metadata_content
                .lines()
                .find_map(|line| line.strip_prefix("Name: "))
                .ok_or_else(|| anyhow!("Package name not found in METADATA"))?
                .to_lowercase();

            // Read the record file for the package.
            let record_path = entry.join("RECORD");
            let record_content = fs::read_to_string(&record_path)
                .with_context(|| format!("Failed to read RECORD in {:?}", entry))?;

            // Collect the aliases (root directory names) for the package.
            let aliases: HashSet<String> = record_content
                .lines()
                .filter_map(|line| {
                    let alias_path_str = line.split(',').next()?;
                    let alias_path = Path::new(alias_path_str);

                    // Check if the file extension is not .py
                    if alias_path.extension().unwrap_or_default() != "py" {
                        return None;
                    }

                    // Ensure there is at least one directory level in the path.
                    // This is to avoid adding packages at top-level directories.
                    // Ex: `site-packages/foo.py` is not a valid package.
                    if alias_path.components().count() <= 1 {
                        return None;
                    }

                    // Extract the root directory name.
                    alias_path.components().next().and_then(|comp| {
                        if let Component::Normal(root_dir) = comp {
                            root_dir.to_str().map(ToString::to_string)
                        } else {
                            None
                        }
                    })
                })
                .collect();

            // If there are no aliases, skip to the next entry.
            if aliases.is_empty() {
                continue;
            }

            // Calculate the size of the package by summing the sizes of all aliases.
            // This is not the most accurate way to calculate the size, but it's a good approximation.
            let size = aliases
                .iter()
                .map(|alias| path.join(alias))
                .map(|potential_path| get_size(&potential_path).unwrap_or(0))
                .sum();

            // Create a new package using the extracted information and add it to the manifest.
            let package = PackageBuilder::new(pkg_id, aliases, size).build();

            packages.insert(package);
        }
    }
    Ok(packages)
}
