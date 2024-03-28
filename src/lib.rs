#![feature(test)]
extern crate test;

pub mod cli;
pub mod defs;
pub mod exit_codes;
pub mod output;

extern crate fs_extra;

use fs_extra::dir::get_size;

use std::collections::HashSet;
use std::env;
use std::fs;
use std::hash::Hash;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::str;

use crate::cli::*;
// use crate::defs::{Dependency, Package, PackageBuilder, Packages, SitePackage};
use crate::exit_codes::*;

extern crate bytesize;

use crate::Config;
use anyhow::{anyhow, bail};

use exit_codes::ExitCode;

use serde::{Deserialize, Serialize};

use std::path::Component;

use anyhow::{Context, Result};
use glob::glob;
use rustpython_ast::Visitor;
use rustpython_parser::{ast, parse, Mode};
use toml::Value;
use walkdir::WalkDir;

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct SitePackages {
    paths: HashSet<PathBuf>,
}

impl SitePackages {
    pub fn new(paths: HashSet<PathBuf>) -> Result<Self> {
        let validated_paths: HashSet<PathBuf> =
            paths.into_iter().filter(|path| path.exists()).collect();

        if validated_paths.is_empty() {
            bail!("No site-packages found. Are you sure you are in a virtual environment?");
        }

        Ok(SitePackages {
            paths: validated_paths,
        })
    }
    // Function to get the site package directory
    /// This function executes the command `python -m site` to get the site package directory
    /// It returns a Result containing a `SitePackage` struct or an error
    pub fn get_site_packages() -> Result<Self> {
        let output = Command::new("python")
            .arg("-m")
            .arg("site")
            .output()
            .context("Failed to execute `python -m site`. Are you sure Python is installed?")?;

        let output_str = str::from_utf8(&output.stdout)
            .context("Output was not valid UTF-8.")?
            .trim();

        // Extract the site package paths from the output
        let pkg_paths = output_str
            .lines()
            .filter(|line| line.contains("site-packages"))
            .map(|s| s.trim_matches(|c: char| c.is_whitespace() || c == '\'' || c == ','))
            .map(PathBuf::from)
            .collect();

        // Create a new SitePackage struct with the extracted paths
        SitePackages::new(pkg_paths)
    }

    pub fn paths(&self) -> &HashSet<PathBuf> {
        &self.paths
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Package {
    id: String,
    size: u64,
    aliases: HashSet<String>,
    dependency: Option<Dependency>, // Optionally linked Dependency
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct PackageBuilder {
    id: String,
    size: u64,
    aliases: HashSet<String>,
    dependency: Option<Dependency>, // Optionally linked Dependency
}

impl PackageBuilder {
    pub fn new(id: String, aliases: HashSet<String>, size: u64) -> Self {
        Self {
            id,
            size,
            aliases,
            dependency: None,
        }
    }

    pub fn size(mut self, size: u64) -> Self {
        self.size = size;
        self
    }

    pub fn aliases(mut self, aliases: HashSet<String>) -> Self {
        self.aliases = aliases;
        self
    }
    pub fn dependency(mut self, dependency: Dependency) -> Self {
        self.dependency = Some(dependency);
        self
    }

    pub fn build(mut self) -> Package {
        self.id = self.id.replace("_", "-");
        Package {
            id: self.id,
            size: self.size,
            aliases: self.aliases,
            dependency: self.dependency,
        }
    }
}
#[derive(Default)]

pub struct Packages {
    manifest: Vec<Package>,
}

// This is a perfect example of a strategy pattern
impl Packages {
    pub fn add_package(&mut self, package: Package) {
        self.manifest.push(package);
    }

    fn get_used(&self, deps: &HashSet<Dependency>, imports: &HashSet<String>) -> Vec<Package> {
        deps.iter()
            .filter_map(|dep| {
                self.manifest
                    .iter()
                    .find(|pkg| pkg.id == dep.id && !pkg.aliases.is_disjoint(imports))
                    .map(|pkg| {
                        PackageBuilder::new(pkg.id.clone(), pkg.aliases.clone(), pkg.size)
                            .dependency(dep.clone())
                            .build()
                    })
            })
            .collect()
    }

    fn get_unused(&self, deps: &HashSet<Dependency>, imports: &HashSet<String>) -> Vec<Package> {
        deps.iter()
            .filter_map(|dep| {
                self.manifest
                    .iter()
                    .find(|pkg| pkg.id == dep.id && pkg.aliases.is_disjoint(imports))
                    .map(|pkg| {
                        PackageBuilder::new(pkg.id.clone(), pkg.aliases.clone(), pkg.size)
                            .dependency(dep.clone())
                            .build()
                    })
            })
            .collect()
    }

    fn get_untracked(&self, deps: &HashSet<Dependency>, imports: &HashSet<String>) -> Vec<Package> {
        let dep_ids: HashSet<String> = deps.iter().map(|dep| dep.id.clone()).collect();

        self.manifest
            .iter()
            .filter(|pkg| !pkg.aliases.is_disjoint(imports) && !dep_ids.contains(&pkg.id))
            .cloned()
            .collect()
    }

    pub fn scan(
        &self,
        config: Config,
        deps: &HashSet<Dependency>,
        imports: &HashSet<String>,
    ) -> Vec<Package> {
        match config.package_state {
            PackageState::Untracked => self.get_untracked(deps, imports),
            PackageState::Used => self.get_used(deps, imports),
            PackageState::Unused => self.get_unused(deps, imports),
        }
    }

    /// This function loads the packages from the specified site packages directory.
    /// It takes a `SitePackages` object as input and returns a `Result` indicating success or failure.
    pub fn load(&mut self, site_package: SitePackages) -> Result<()> {
        // Iterate over each path in the site packages directory.
        for path in site_package.paths() {
            let glob_pattern = format!("{}/{}-info", path.display(), "*");

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
                self.add_package(package);
            }
        }
        Ok(())
    }

    // For `testing` purposes ONLY. Not intended to be public facing API.
    #[cfg(test)]
    pub fn _mapping(&self) -> &Vec<Package> {
        &self.manifest
    }
}

/// Extract the first part of an import statement
///  e.g. `os.path` -> `os`
#[inline]
fn stem_import(import: &str) -> String {
    import.split('.').next().unwrap_or_default().into()
}
/// Collects all the dependencies from the AST
struct Imports {
    import: HashSet<String>,
}

/// This is a visitor pattern that implements the Visitor trait
impl Visitor for Imports {
    /// This is a generic visit method that will be called for all nodes
    fn visit_stmt(&mut self, node: ast::Stmt<ast::text_size::TextRange>) {
        self.generic_visit_stmt(node);
    }
    /// This method is `overridden` to collect the dependencies into `self.deps`
    fn visit_stmt_import(&mut self, node: ast::StmtImport) {
        node.names.iter().for_each(|alias| {
            self.import.insert(stem_import(&alias.name));
        })
    }

    /// This method is `overridden` to collect the dependencies into `self.deps`
    fn visit_stmt_import_from(&mut self, node: ast::StmtImportFrom) {
        if let Some(module) = &node.module {
            self.import.insert(stem_import(module));
        }
    }
}

pub fn get_imports(config: &Config) -> Result<HashSet<String>> {
    WalkDir::new(&config.base_directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| {
            let file_name = entry.file_name().to_string_lossy();

            // Ignore hidden files and directories if `ignore_hidden` is set to true
            file_name.ends_with(".py") && !(config.ignore_hidden && file_name.starts_with("."))
        })
        .try_fold(HashSet::new(), |mut acc, entry| {
            let file_content = fs::read_to_string(entry.path())?;
            let module = parse(&file_content, Mode::Module, "<embedded>")?;

            let mut collector = Imports {
                import: HashSet::new(),
            };

            module
                .module()
                .unwrap() //Probably should change this from unwrap to something else
                .body
                .into_iter()
                .for_each(|node| collector.visit_stmt(node));

            acc.extend(collector.import);

            Ok(acc)
        })
}
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Eq, Hash)]
pub struct Dependency {
    pub id: String,
    pub version: Option<String>,
    pub category: Option<String>, // Renamed from type_ for clarity
}

/// Parses the `pyproject.toml` to collect all dependencies specified under `[tool.poetry.*]`.
///
/// # Arguments
///
/// * `path` - The path to the `pyproject.toml` file.
///
/// # Returns
///
/// Returns a `Result` containing a `HashSet` of `Dependency` if successful, or an error.
pub fn collect_dependencies_from_toml(path: &Path) -> Result<HashSet<Dependency>> {
    let toml_content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read TOML file at {:?}", path))?;
    let parsed_toml =
        toml::from_str::<Value>(&toml_content).with_context(|| "Failed to parse TOML content")?;

    let mut dependencies = HashSet::new();
    collect_dependencies(&parsed_toml, &mut dependencies, "");

    Ok(dependencies)
}

/// Recursively visits TOML values to collect dependencies.
fn collect_dependencies(value: &Value, dependencies: &mut HashSet<Dependency>, path_prefix: &str) {
    if let Value::Table(table) = value {
        for (key, value) in table {
            let current_path = if path_prefix.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", path_prefix, key)
            };

            match value {
                Value::Table(dep_table) if key.ends_with("dependencies") => {
                    let category = current_path;
                    dep_table
                        .iter()
                        .filter_map(|(name, value)| match value {
                            Value::String(version) => Some(Dependency {
                                id: name.to_string(),
                                version: Some(version.to_string()),
                                category: Some(category.clone()),
                            }),
                            Value::Table(_) => Some(Dependency {
                                id: name.to_string(),
                                version: None,
                                category: Some(category.clone()),
                            }),
                            _ => None,
                        })
                        .for_each(|dep| {
                            dependencies.insert(dep);
                        });
                }
                _ => collect_dependencies(value, dependencies, &current_path),
            }
        }
    }
}
// // Can't read bash or bat scripts. WIll need to return to this issue
pub fn analyze(config: Config) -> Result<ExitCode> {
    let pyproject_deps = collect_dependencies_from_toml(&config.dep_spec_file)?;

    // THIS IS SUPER SLOW, LET'S USE THE WALKBUILDER TO GET THE PKGS FROM FD
    let project_imports = get_imports(&config)?;
    // let site_pkgs = get_site_package_dir(&config)?;
    let site_pkgs = SitePackages::get_site_packages()?;

    let mut packages = Packages::default();

    // Loads the packages from the local site packages
    packages.load(site_pkgs)?;

    let test = packages.scan(config, &pyproject_deps, &project_imports);
    println!("{:#?}", test);

    Ok(ExitCode::Success)

    // let pkgs = get_installed_packages(site_pkgs)?;
    // println!("here is what is in the site packages {:#?}", pkgs);

    // println!("here is what is in the pyproject.toml {:?}", pyproject_deps);

    // outcome.packages = pkgs.scan(config, &pyproject_deps, &project_imports);

    // println!("here is what is relevant {:#?}", relevant_pkgs);

    // let used_pkgs = installed_pkgs.filter_used_pkgs(&imports);

    // THIS IS WRONG. IF THE DEP IS NOT INSTALLED IT'S NOT A "UNUSED DEP"
    // outcome.unused_deps = pyproject_deps
    //     .into_iter()
    //     .filter(|dep| !used_pkgs.contains(&dep.id) && !DEFAULT_PKGS.contains(&dep.id.as_str()))
    //     .collect();

    // outcome.packages = relevant_pkgs;

    // println!("here is what is unused {:?}", outcome.unused_deps);

    // outcome.success = outcome.unused_deps.is_empty();

    // if !outcome.success {
    //     let mut note = "".to_owned();
    //     note += "Note: There might be false-positives.\n";
    //     note += "      For example, `pip-udeps` cannot detect usage of packages that are not imported under `[tool.poetry.*]`.\n";
    //     outcome.note = Some(note);
    // }

    // Ok(outcome)

    // print_report(&config, std::io::stdout())
}

// #[cfg(test)]
// mod tests {

//     use super::*;

//     use defs::{OutputKind, PackageState};
//     use std::fs::File;
//     use std::io::Write;
//     use std::io::{self};
//     use tempfile::TempDir;
//     use test::Bencher;

//     // Used to create a temporary directory with the given directories and files
//     fn create_working_directory(
//         dirs: &[&'static str],
//         files: Option<&[&'static str]>,
//     ) -> Result<TempDir, io::Error> {
//         let temp_dir = TempDir::new()?;

//         dirs.iter().for_each(|directory| {
//             let dir_path = temp_dir.path().join(directory);
//             fs::create_dir_all(dir_path).unwrap();
//         });

//         if let Some(files) = files {
//             files.iter().for_each(|file| {
//                 let file_path = temp_dir.path().join(file);
//                 File::create(file_path).unwrap();
//             });
//         }

//         Ok(temp_dir)
//     }

//     struct TestEnv {
//         /// Temporary project directory
//         _temp_dir: TempDir,

//         /// Test Configuration struct
//         config: Config,
//     }

//     impl TestEnv {
//         fn new(dirs: &[&'static str], files: Option<&[&'static str]>) -> Self {
//             let temp_dir = create_working_directory(dirs, files).unwrap();
//             let base_directory = temp_dir.path().join(dirs[0]);
//             let pyproject_path: PathBuf = base_directory.join("pyproject.toml");
//             let mut file = File::create(&pyproject_path).unwrap();

//             file.write_all(
//                 r#"
//                             [tool.poetry.dependencies]
//                             requests = "2.25.1"
//                             python = "^3.8"
//                             pandas = "^1.2.0"
//                             "#
//                 .as_bytes(),
//             )
//             .unwrap();

//             let config = Config {
//                 base_directory,
//                 dep_spec_file: pyproject_path,
//                 ignore_hidden: false,
//                 env: Env::Test,
//                 output: OutputKind::Human,
//                 package_state: PackageState::Unused,
//             };

//             Self {
//                 _temp_dir: temp_dir,
//                 config,
//             }
//         }
//     }

//     #[bench]
//     fn bench_get_used_imports(b: &mut Bencher) {
//         let te = TestEnv::new(&["dir1", "dir2"], Some(&["file1.py"]));
//         b.iter(|| get_imports(&te.config));
//     }

//     #[bench]
//     fn bench_get_dependencies_from_toml(b: &mut Bencher) {
//         let te = TestEnv::new(&["dir1", "dir2"], Some(&["pyproject.toml"]));
//         b.iter(|| get_dependencies_from_toml(&te.config.dep_spec_file));
//     }

//     // #[test]
//     // fn basic_usage() {
//     //     let te = TestEnv::new(
//     //         &["dir1", "dir2"],
//     //         Some(&["requirements.txt", "pyproject.toml", "file1.py"]),
//     //     );

//     //     let unused_deps = get_unused_dependencies(&te.config);
//     //     assert!(unused_deps.is_ok());

//     //     let outcome = unused_deps.unwrap();
//     //     assert_eq!(outcome.success, false); // There should be unused dependencies

//     //     // This is because we use python by default
//     //     assert_eq!(
//     //         outcome.unused_deps.len(),
//     //         2,
//     //         "There should be 2 unused dependencies"
//     //     );
//     //     // assert_eq!(outcome.unused_deps.iter().next().unwrap().name, "pandas");
//     //     assert!(outcome
//     //         .unused_deps
//     //         .iter()
//     //         .any(|dep| dep.id == "pandas" || dep.id == "requests"));

//     //     // Now let's import requests in file1.py
//     //     let file_path = te.config.base_directory.join("file1.py");
//     //     let mut file = File::create(file_path).unwrap();
//     //     file.write_all("import requests".as_bytes()).unwrap();

//     //     let unused_deps = get_unused_dependencies(&te.config);
//     //     assert!(unused_deps.is_ok());

//     //     // check that there are no unused dependencies
//     //     let outcome = unused_deps.unwrap();
//     //     assert_eq!(outcome.success, false);
//     //     assert_eq!(outcome.unused_deps.len(), 1);

//     //     // Now let's import requests in file1.py
//     //     let file_path = te.config.base_directory.join("file1.py");
//     //     let mut file = File::create(file_path).unwrap();
//     //     file.write_all("import requests\nimport pandas as pd".as_bytes())
//     //         .unwrap();

//     //     let unused_deps = get_unused_dependencies(&te.config);
//     //     assert!(unused_deps.is_ok());
//     //     assert_eq!(
//     //         unused_deps.unwrap().unused_deps.len(),
//     //         0,
//     //         "There should be no unused dependency"
//     //     );
//     // }

//     #[test]
//     fn stem_import_correctly_stems() {
//         let first_part = stem_import("os.path");
//         assert_eq!(first_part.as_str(), "os");

//         let first_part = stem_import("os");
//         assert_eq!(first_part.as_str(), "os");

//         let first_part = stem_import("");
//         assert_eq!(first_part.as_str(), "");
//     }

//     #[test]
//     fn get_used_imports_correctly_collects() {
//         let te = TestEnv::new(
//             &["dir1", "dir2"],
//             Some(&["requirements.txt", "pyproject.toml", "file1.py"]),
//         );

//         let used_imports = get_imports(&te.config);
//         assert!(used_imports.is_ok());

//         let used_imports = used_imports.unwrap();
//         assert_eq!(used_imports.len(), 0);

//         let file_path = te.config.base_directory.join("file1.py");
//         let mut file = File::create(file_path).unwrap();
//         file.write_all(r#"import pandas as pd"#.as_bytes()).unwrap();

//         let used_imports = get_imports(&te.config);
//         assert!(used_imports.is_ok());

//         let used_imports = used_imports.unwrap();
//         assert_eq!(used_imports.len(), 1);
//         assert!(used_imports.contains("pandas"));
//         assert!(!used_imports.contains("sklearn"));
//     }

//     // #[test]
//     // fn correct_promt_from_get_prompt() {
//     //     let venv = Some("test-venv".to_string());
//     //     let prompt = get_prompt(&venv);
//     //     assert_eq!(
//     //         prompt,
//     //         "Detected virtual environment: `test-venv`. Is this correct?"
//     //     );

//     //     let venv = None;
//     //     let prompt = get_prompt(&venv);
//     //     assert_eq!(
//     //         prompt,
//     //         "WARNING: No virtual environment detected. Results may be inaccurate. Continue?"
//     //             .red()
//     //             .to_string()
//     //     );
//     // }

//     // #[test]
//     // fn get_site_package_dir_success() {
//     //     let te = TestEnv::new(&["dir1", "dir2"], Some(&["pyproject.toml"]));

//     //     let site_pkgs = get_site_package_dir(&te.config).unwrap();

//     //     assert!(!site_pkgs.paths.is_empty());

//     //     let venv_name = env::var("VIRTUAL_ENV")
//     //         .ok()
//     //         .and_then(|path| path.split('/').last().map(String::from));
//     //     assert_eq!(site_pkgs.venv, venv_name);
//     // }

//     // #[test]

//     // fn get_installed_packages_correctly_maps() {
//     //     // Create a temporary environment resembling site-packages
//     //     let temp_dir = tempfile::TempDir::new().unwrap();
//     //     let site_packages_dir = temp_dir.path().join("site-packages");
//     //     fs::create_dir(&site_packages_dir).unwrap();

//     //     // Simulate a couple of installed packages with top_level.txt files
//     //     let pkg1_dir = site_packages_dir.join("example_pkg1-0.1.0-info");
//     //     fs::create_dir_all(&pkg1_dir).unwrap();
//     //     fs::write(pkg1_dir.join("top_level.txt"), "example_pkg1\n").unwrap();

//     //     let pkg2_dir = site_packages_dir.join("example_pkg2-0.2.0-info");
//     //     fs::create_dir_all(&pkg2_dir).unwrap();
//     //     fs::write(pkg2_dir.join("top_level.txt"), "example_pkg2\n").unwrap();

//     //     // lets do another package like scikit_learn where we know the name will get remapped to sklearn
//     //     let pkg3_dir = site_packages_dir.join("scikit_learn-0.24.1-info");
//     //     fs::create_dir_all(&pkg3_dir).unwrap();
//     //     fs::write(pkg3_dir.join("top_level.txt"), "sklearn\n").unwrap();

//     //     let site_pkgs = SitePackages {
//     //         paths: vec![site_packages_dir],
//     //         venv: Some("test-venv".to_string()),
//     //     };

//     //     let installed_pkgs = get_installed_packages(site_pkgs).unwrap();

//     //     assert_eq!(
//     //         installed_pkgs._mapping().len(),
//     //         3,
//     //         "Should have found two installed packages"
//     //     );

//     //     // Assert that the package names and import names are correct
//     //     assert!(
//     //         installed_pkgs._mapping().get("example-pkg1").is_some(),
//     //         "Should contain example_pkg1"
//     //     );

//     //     assert!(
//     //         installed_pkgs
//     //             ._mapping()
//     //             .get("example-pkg1")
//     //             .unwrap()
//     //             .contains("example_pkg1"),
//     //         "example-pkg1 should contain example_pkg1"
//     //     );
//     //     assert!(
//     //         installed_pkgs._mapping().get("example-pkg2").is_some(),
//     //         "Should contain example_pkg2"
//     //     );

//     //     assert!(
//     //         installed_pkgs
//     //             ._mapping()
//     //             .get("example-pkg2")
//     //             .unwrap()
//     //             .contains("example_pkg2"),
//     //         "example-pkg2 should contain example_pkg2"
//     //     );

//     //     assert!(
//     //         installed_pkgs._mapping().get("scikit-learn").is_some(),
//     //         "Should contain scikit_learn"
//     //     );

//     //     assert!(
//     //         installed_pkgs
//     //             ._mapping()
//     //             .get("scikit-learn")
//     //             .unwrap()
//     //             .contains("sklearn"),
//     //         "scikit_learn should contain sklearn"
//     //     );
//     //     // non-existent package
//     //     assert!(
//     //         !installed_pkgs._mapping().get("non-existent").is_some(),
//     //         "Should not contain non-existent"
//     //     );
//     // }

//     // #[test]
//     // fn get_deps_from_pyproject_toml_success() {
//     //     let temp_dir =
//     //         create_working_directory(&["dir1", "dir2"], Some(&["pyproject.toml"])).unwrap();
//     //     let base_directory = temp_dir.path().join("dir1");
//     //     let file_path = base_directory.join("pyproject.toml");
//     //     let mut file = File::create(&file_path).unwrap();
//     //     file.write_all(
//     //         r#"
//     //         [tool.poetry.dependencies]
//     //         requests = "2.25.1"
//     //         python = "^3.8"
//     //         "#
//     //         .as_bytes(),
//     //     )
//     //     .unwrap();

//     //     let packages = get_dependencies_from_pyproject_toml(&file_path).unwrap();
//     //     assert_eq!(packages.len(), 2);
//     //     assert!(packages.contains(&PyProjectDeps {
//     //         name: "requests".to_string()
//     //     }));
//     //     assert!(packages.contains(&PyProjectDeps {
//     //         name: "python".to_string()
//     //     }));
//     // }
// }
