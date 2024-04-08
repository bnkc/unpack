extern crate bytesize;
extern crate fs_extra;

use std::collections::HashSet;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::str;

use anyhow::bail;
use anyhow::{Context, Result};
use fs_extra::dir::get_size;
use glob::glob;
use serde::{Deserialize, Serialize};

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
    }
}

impl Package {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn aliases(&self) -> &HashSet<String> {
        &self.aliases
    }

    pub fn size(&self) -> u64 {
        self.size
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
        self.id = self.id.replace('_', "-");
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

/// Process the METADATA and RECORD files in the dist-info directory to extract package information.
fn process_dist_info(entry: &Path) -> Result<Package> {
    let metadata_path = entry.join("METADATA");
    let metadata_content = fs::read_to_string(metadata_path)?;

    let pkg_id = metadata_content
        .lines()
        .find_map(|line| line.strip_prefix("Name: "))
        .map(str::to_lowercase)
        .context("Package name not found in METADATA")?;

    let record_path = entry.join("RECORD");
    let record_content = fs::read_to_string(record_path)?;

    let aliases: HashSet<String> = record_content
        .lines()
        .filter_map(|line| {
            let alias_path_str = line.split(',').next()?;
            let alias_path = Path::new(alias_path_str);
            if alias_path.extension().unwrap_or_default() != "py"
                || alias_path.components().count() <= 1
            {
                return None;
            }
            alias_path.components().next().and_then(|comp| {
                if let Component::Normal(root_dir) = comp {
                    root_dir.to_str().map(ToString::to_string)
                } else {
                    None
                }
            })
        })
        .collect();

    if aliases.is_empty() {
        bail!("No valid aliases found in RECORD");
    }

    // root dir without the "dist-info" suffix
    let site_dir = entry.parent().unwrap();

    let size = aliases
        .iter()
        .map(|alias| site_dir.join(alias))
        .map(|potential_path| get_size(potential_path).unwrap_or(0))
        .sum();

    Ok(PackageBuilder::new(pkg_id, aliases, size).build())
}

/// Process the PKG-INFO and top_level.txt files in the egg-info directory to extract package information.
fn process_egg_info(entry: &Path) -> Result<Package> {
    let metadata_path = entry.join("PKG-INFO");
    let metadata_content = fs::read_to_string(metadata_path)?;

    let pkg_id = metadata_content
        .lines()
        .find_map(|line| line.strip_prefix("Name: "))
        .map(str::to_lowercase)
        .context("Package name not found in PKG-INFO")?;

    let top_level_path = entry.join("top_level.txt");

    let aliases: HashSet<String> = fs::read_to_string(top_level_path)?
        .lines()
        .map(ToString::to_string)
        .collect();

    if aliases.is_empty() {
        bail!("No valid aliases found in top_level.txt");
    }

    // root dir without the "egg-info" suffix
    let site_dir = entry.parent().unwrap();

    let size = aliases
        .iter()
        .map(|alias| site_dir.join(alias))
        .map(|potential_path| get_size(potential_path).unwrap_or(0))
        .sum();

    Ok(PackageBuilder::new(pkg_id, aliases, size).build())
}

/// This function determines the packages installed in the site-packages directory.
pub fn get_packages(site_packages: HashSet<PathBuf>) -> Result<HashSet<Package>> {
    let mut packages = HashSet::new();

    for path in site_packages {
        let dist_info_pattern = format!("{}/{}dist-info", path.display(), "*");
        for entry in glob(&dist_info_pattern)?.filter_map(Result::ok) {
            if let Ok(package) = process_dist_info(entry.as_path()) {
                packages.insert(package);
            }
        }

        let egg_info_pattern = format!("{}/{}egg-info", path.display(), "*");
        for entry in glob(&egg_info_pattern)?.filter_map(Result::ok) {
            if let Ok(package) = process_egg_info(entry.as_path()) {
                packages.insert(package);
            }
        }
    }

    Ok(packages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    /// Helper function to create either dist-info or egg-info directory structure
    /// with optional files and their contents.
    fn create_info_dir(
        temp_dir: &TempDir,
        package_name: &str,
        dir_type: &str,
        files: Vec<(&str, Option<&str>)>,
    ) {
        let info_path = temp_dir
            .path()
            .join(format!("{}-0.1.{}", package_name, dir_type));
        fs::create_dir(&info_path).unwrap();

        for (file_name, content) in files {
            let file_path = info_path.join(file_name);
            let mut file = File::create(&file_path).unwrap();
            if let Some(content) = content {
                writeln!(file, "{}", content).unwrap();
            }
        }
    }

    #[test]
    fn test_process_dist_info_successful() {
        let temp_dir = TempDir::new().unwrap();
        create_info_dir(
            &temp_dir,
            "successful_package",
            "dist-info",
            vec![
                ("METADATA", Some("Name: Successful_Package\nVersion: 1.0.0")),
                (
                    "RECORD",
                    Some("successful_package/__init__.py,,\nsuccessful_package/module.py,,"),
                ),
            ],
        );

        let result = process_dist_info(&temp_dir.path().join("successful_package-0.1.dist-info"));
        assert!(result.is_ok());
        let package = result.unwrap();
        assert_eq!(package.id, "successful-package");
        assert!(package.aliases.contains("successful_package"));
    }

    #[test]
    fn test_process_egg_info_successful() {
        let temp_dir = TempDir::new().unwrap();
        create_info_dir(
            &temp_dir,
            "successful_egg",
            "egg-info",
            vec![
                ("PKG-INFO", Some("Name: Successful_Egg\nVersion: 2.0.0")),
                ("top_level.txt", Some("successful_egg")),
            ],
        );

        let result = process_egg_info(&temp_dir.path().join("successful_egg-0.1.egg-info"));
        assert!(result.is_ok());
        let package = result.unwrap();
        assert_eq!(package.id, "successful-egg");
        assert!(package.aliases.contains("successful_egg"));
    }

    #[test]
    fn test_process_dist_info_no_metadata() {
        let temp_dir = TempDir::new().unwrap();
        create_info_dir(
            &temp_dir,
            "no_metadata_package",
            "dist-info",
            vec![("RECORD", Some("no_metadata_package/__init__.py,,"))],
        );

        let result = process_dist_info(&temp_dir.path().join("no_metadata_package-0.1.dist-info"));
        assert!(result.is_err());
    }

    #[test]
    fn test_process_egg_info_no_pkg_info() {
        let temp_dir = TempDir::new().unwrap();
        create_info_dir(
            &temp_dir,
            "no_pkg_info_egg",
            "egg-info",
            vec![("top_level.txt", Some("no_pkg_info_egg"))],
        );

        let result = process_egg_info(&temp_dir.path().join("no_pkg_info_egg-0.1.egg-info"));
        assert!(result.is_err());
    }

    #[test]
    fn test_process_dist_info_no_record() {
        let temp_dir = TempDir::new().unwrap();
        create_info_dir(
            &temp_dir,
            "no_record_package",
            "dist-info",
            vec![("METADATA", Some("Name: No_Record_Package\nVersion: 1.0.0"))],
        );

        let result = process_dist_info(&temp_dir.path().join("no_record_package-0.1.dist-info"));
        assert!(result.is_err());
    }

    #[test]
    fn test_process_egg_info_no_top_level() {
        let temp_dir = TempDir::new().unwrap();
        create_info_dir(
            &temp_dir,
            "no_top_level_egg",
            "egg-info",
            vec![("PKG-INFO", Some("Name: No_Top_Level_Egg\nVersion: 2.0.0"))],
        );

        let result = process_egg_info(&temp_dir.path().join("no_top_level_egg-0.1.egg-info"));
        assert!(result.is_err());
    }
}
