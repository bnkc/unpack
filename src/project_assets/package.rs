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
    use tempfile::tempdir;

    /// Helper function to create dist-info directory structure with optional METADATA and RECORD files.
    fn create_dist_info_dir(
        temp_dir: &tempfile::TempDir,
        package_name: &str,
        metadata_content: Option<&str>,
        record_content: Option<&str>,
    ) {
        let dist_info_path = temp_dir
            .path()
            .join(format!("{}-0.1.dist-info", package_name));
        fs::create_dir(&dist_info_path).unwrap();

        if let Some(metadata) = metadata_content {
            let metadata_path = dist_info_path.join("METADATA");
            let mut metadata_file = File::create(&metadata_path).unwrap();
            writeln!(metadata_file, "{}", metadata).unwrap();
        }

        if let Some(record) = record_content {
            let record_path = dist_info_path.join("RECORD");
            let mut record_file = File::create(&record_path).unwrap();
            writeln!(record_file, "{}", record).unwrap();
        }
    }

    /// Helper function to create egg-info directory structure with optional PKG-INFO and top_level.txt files.
    fn create_egg_info_dir(
        temp_dir: &tempfile::TempDir,
        package_name: &str,
        metadata_content: Option<&str>,
        top_level_content: Option<&str>,
    ) {
        let egg_info_path = temp_dir
            .path()
            .join(format!("{}-0.1.egg-info", package_name));
        fs::create_dir(&egg_info_path).unwrap();

        if let Some(metadata) = metadata_content {
            let metadata_path = egg_info_path.join("PKG-INFO");
            let mut metadata_file = File::create(&metadata_path).unwrap();
            writeln!(metadata_file, "{}", metadata).unwrap();
        }

        if let Some(top_level) = top_level_content {
            let top_level_path = egg_info_path.join("top_level.txt");
            let mut top_level_file = File::create(&top_level_path).unwrap();
            writeln!(top_level_file, "{}", top_level).unwrap();
        }
    }

    #[test]
    fn test_process_dist_info() {
        let temp_dir = tempdir().unwrap();
        create_dist_info_dir(
            &temp_dir,
            "test_package",
            Some("Name: Test_Package"),
            Some("test_package/__init__.py,,"),
        );

        let package = process_dist_info(&temp_dir.path().join("test_package-0.1.dist-info"))
            .expect("Failed to process dist-info directory.");

        assert_eq!(package.id, "test-package");
        assert_eq!(package.size, 0);
        assert!(package.aliases.contains("test_package"));
    }

    /// test that raises an error when the aliases are not found in the RECORD file
    #[test]
    fn test_process_dist_info_no_aliases() {
        let temp_dir = tempdir().unwrap();
        create_dist_info_dir(&temp_dir, "no_aliases", Some("Name: No_Aliases"), Some(""));

        let result = process_dist_info(&temp_dir.path().join("no_aliases-0.1.dist-info"));
        assert!(
            result.is_err(),
            "Should raise an error when aliases are not found."
        );
    }

    #[test]
    fn test_process_egg_info() {
        let temp_dir = tempdir().unwrap();
        create_egg_info_dir(
            &temp_dir,
            "test_package",
            Some("Name: Test_Package"),
            Some("test_package"),
        );

        let package = process_egg_info(&temp_dir.path().join("test_package-0.1.egg-info"))
            .expect("Failed to process egg-info directory.");

        assert_eq!(package.id, "test-package");
        assert_eq!(package.size, 0);
        assert!(package.aliases.contains("test_package"));
    }

    /// test that raises an error when the aliases are not found in the top_level.txt file
    #[test]
    fn test_process_egg_info_no_aliases() {
        let temp_dir = tempdir().unwrap();
        create_egg_info_dir(&temp_dir, "no_aliases", Some("Name: No_Aliases"), None);

        let result = process_egg_info(&temp_dir.path().join("no_aliases-0.1.egg-info"));
        assert!(
            result.is_err(),
            "Should raise an error when aliases are not found."
        );
    }

    /// Tests that `get_site_packages` successfully retrieves the site-packages directory.
    #[test]
    fn test_get_site_packages() {
        // This test assumes that Python and a virtual environment are correctly set up.
        let site_packages = get_site_packages();
        assert!(
            site_packages.is_ok(),
            "Failed to get site-packages directory. "
        );
    }

    #[test]
    fn test_get_packages() {
        let temp_dir = tempdir().unwrap();
        let site_packages_path = temp_dir.path().join("site-packages");
        fs::create_dir(&site_packages_path).unwrap();

        // Mock a package structure
        let package_name = "test_package";
        let dist_info_path = site_packages_path.join(format!("{}-0.1.dist-info", package_name));
        fs::create_dir(&dist_info_path).unwrap();

        // Create METADATA file
        let metadata_path = dist_info_path.join("METADATA");
        let mut metadata_file = File::create(&metadata_path).unwrap();
        writeln!(metadata_file, "Name: Test_Package").unwrap();

        // Create RECORD file
        let record_path = dist_info_path.join("RECORD");
        let mut record_file = File::create(&record_path).unwrap();
        writeln!(record_file, "test_package/__init__.py,,").unwrap();

        let packages = get_packages(std::iter::once(site_packages_path).collect()).unwrap();

        assert_eq!(packages.len(), 1);
        let package = packages.iter().next().unwrap();
        assert_eq!(package.id, "test-package");
        assert!(package.aliases.contains("test_package"));
    }

    #[test]
    fn test_get_packages_missing_metadata() {
        let temp_dir = tempdir().unwrap();
        create_dist_info_dir(&temp_dir, "missing_metadata", None, Some(""));

        let packages =
            get_packages(std::iter::once(temp_dir.path().to_path_buf()).collect()).unwrap();

        assert!(
            packages.is_empty(),
            "Packages set should be empty when RECORD is missing."
        );
    }

    /// Test case with invalid METADATA file.
    #[test]
    fn test_get_packages_invalid_metadata() {
        let temp_dir = tempdir().unwrap();
        create_dist_info_dir(
            &temp_dir,
            "invalid_metadata",
            Some("Invalid Content"),
            Some(""),
        );

        let packages =
            get_packages(std::iter::once(temp_dir.path().to_path_buf()).collect()).unwrap();
        assert!(
            packages.is_empty(),
            "Packages set should be empty with invalid METADATA content."
        );
    }

    /// Test case with empty RECORD file.
    #[test]
    fn test_get_packages_empty_record() {
        let temp_dir = tempdir().unwrap();
        create_dist_info_dir(
            &temp_dir,
            "empty_record",
            Some("Name: Test_Package"),
            Some(""),
        );

        let packages =
            get_packages(std::iter::once(temp_dir.path().to_path_buf()).collect()).unwrap();
        assert!(
            packages.is_empty(),
            "Packages set should be empty with an empty RECORD file."
        );
    }

    /// Tests `PackageBuilder` functionality.
    #[test]
    fn test_package_builder() {
        let id = "test_package";
        let aliases = HashSet::from(["test_package".to_string()]);
        let size = 1024;

        let package = PackageBuilder::new(id.to_string(), aliases, size).build();

        assert_eq!(package.id, "test-package"); // underscore replaced by hyphen
        assert_eq!(package.size, size);
        assert!(package.aliases.contains("test_package"));
    }
}
