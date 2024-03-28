// extern crate bytesize;

// use crate::exit_codes;
// use crate::Config;
// use anyhow::{anyhow, bail, Result};

// use anyhow::Context;

// use exit_codes::ExitCode;
// use glob::glob;

// use serde::{Deserialize, Serialize};
// use std::collections::HashSet;

// use std::fs;
// use std::path::Component;
// use std::path::Path;
// extern crate fs_extra;

// // use std::process::ExitCode;

// use fs_extra::dir::get_size;
// use std::path::PathBuf;

// #[derive(Deserialize, Debug, PartialEq, Clone)]
// pub struct SitePackage {
//     paths: HashSet<PathBuf>,
// }

// impl SitePackage {
//     pub fn new(paths: HashSet<PathBuf>) -> Result<Self> {
//         let validated_paths: HashSet<PathBuf> =
//             paths.into_iter().filter(|path| path.exists()).collect();

//         if validated_paths.is_empty() {
//             bail!("No site-packages found. Are you sure you are in a virtual environment?");
//         }

//         Ok(SitePackage {
//             paths: validated_paths,
//         })
//     }

//     pub fn paths(&self) -> &HashSet<PathBuf> {
//         &self.paths
//     }
// }

// #[derive(Serialize, Deserialize, clap::ValueEnum, Debug, PartialEq, Eq, Clone, Hash)]
// pub enum PackageState {
//     /// The dependency is installed, actively used in the project, and correctly listed in pyproject.toml.
//     /// This state indicates a fully integrated and properly managed dependency.
//     Used,
//     /// The dependency is installed and listed in pyproject.toml but is not actively used in the project.
//     /// Ideal for identifying and possibly removing unnecessary dependencies to clean up the project. (default)
//     Unused,
//     /// The dependency is installed and actively used in the project but is missing from pyproject.toml.
//     /// Highlights dependencies that are implicitly used but not formally declared, which may lead to
//     /// inconsistencies or issues in dependency management and deployment.
//     Untracked,
// }

// #[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Eq, Hash)]
// pub struct Dependency {
//     pub id: String,
//     pub version: Option<String>,
//     pub type_: Option<String>,
// }

// #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
// pub struct Package {
//     id: String,
//     size: u64,
//     aliases: HashSet<String>,
//     dependency: Option<Dependency>, // Optionally linked Dependency
// }

// #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
// pub struct PackageBuilder {
//     id: String,
//     size: u64,
//     aliases: HashSet<String>,
//     dependency: Option<Dependency>, // Optionally linked Dependency
// }

// impl PackageBuilder {
//     pub fn new(id: String, aliases: HashSet<String>, size: u64) -> Self {
//         Self {
//             id,
//             size,
//             aliases,
//             dependency: None,
//         }
//     }

//     pub fn size(mut self, size: u64) -> Self {
//         self.size = size;
//         self
//     }

//     pub fn aliases(mut self, aliases: HashSet<String>) -> Self {
//         self.aliases = aliases;
//         self
//     }
//     pub fn dependency(mut self, dependency: Dependency) -> Self {
//         self.dependency = Some(dependency);
//         self
//     }

//     pub fn build(mut self) -> Package {
//         self.id = self.id.replace("_", "-");
//         Package {
//             id: self.id,
//             size: self.size,
//             aliases: self.aliases,
//             dependency: self.dependency,
//         }
//     }
// }
// #[derive(Default)]

// pub struct Packages {
//     manifest: Vec<Package>,
// }

// impl Packages {
//     pub fn add_package(&mut self, package: Package) {
//         self.manifest.push(package);
//     }

//     fn get_used(&self, deps: &HashSet<Dependency>, imports: &HashSet<String>) -> Vec<Package> {
//         deps.iter()
//             .filter_map(|dep| {
//                 self.manifest
//                     .iter()
//                     .find(|pkg| pkg.id == dep.id && !pkg.aliases.is_disjoint(imports))
//                     .map(|pkg| {
//                         PackageBuilder::new(pkg.id.clone(), pkg.aliases.clone(), pkg.size)
//                             .dependency(dep.clone())
//                             .build()
//                     })
//             })
//             .collect()
//     }

//     fn get_unused(&self, deps: &HashSet<Dependency>, imports: &HashSet<String>) -> Vec<Package> {
//         deps.iter()
//             .filter_map(|dep| {
//                 self.manifest
//                     .iter()
//                     .find(|pkg| pkg.id == dep.id && pkg.aliases.is_disjoint(imports))
//                     .map(|pkg| {
//                         PackageBuilder::new(pkg.id.clone(), pkg.aliases.clone(), pkg.size)
//                             .dependency(dep.clone())
//                             .build()
//                     })
//             })
//             .collect()
//     }

//     fn get_untracked(&self, deps: &HashSet<Dependency>, imports: &HashSet<String>) -> Vec<Package> {
//         let dep_ids: HashSet<String> = deps.iter().map(|dep| dep.id.clone()).collect();

//         self.manifest
//             .iter()
//             .filter(|pkg| !pkg.aliases.is_disjoint(imports) && !dep_ids.contains(&pkg.id))
//             .cloned()
//             .collect()
//     }

//     pub fn scan(
//         &self,
//         config: Config,
//         deps: &HashSet<Dependency>,
//         imports: &HashSet<String>,
//     ) -> Vec<Package> {
//         match config.package_state {
//             PackageState::Used => self.get_used(deps, imports),
//             PackageState::Unused => self.get_unused(deps, imports),
//             PackageState::Untracked => self.get_untracked(deps, imports),
//         }
//     }

//     pub fn load(&mut self, site_package: SitePackage) -> Result<()> {
//         for path in site_package.paths() {
//             let glob_pattern = format!("{}/{}-info", path.display(), "*");
//             for entry in glob(&glob_pattern)?.filter_map(Result::ok) {
//                 let metadata_path = entry.join("METADATA");
//                 let metadata_content = fs::read_to_string(&metadata_path)
//                     .with_context(|| format!("Failed to read METADATA in {:?}", entry))?;

//                 let pkg_id = metadata_content
//                     .lines()
//                     .find_map(|line| line.strip_prefix("Name: "))
//                     .ok_or_else(|| anyhow!("Package name not found in METADATA"))?
//                     .to_lowercase();

//                 let record_path = entry.join("RECORD");
//                 let record_content = fs::read_to_string(&record_path)
//                     .with_context(|| format!("Failed to read RECORD in {:?}", entry))?;

//                 let aliases: HashSet<String> = record_content
//                     .lines()
//                     .filter_map(|line| {
//                         let alias_path_str = line.split(',').next()?;
//                         let alias_path = Path::new(alias_path_str);

//                         // Check if the file extension is not .py
//                         if alias_path.extension().unwrap_or_default() != "py" {
//                             return None;
//                         }

//                         // Ensure there is at least one directory level in the path.
//                         // This is to avoid adding packages are top-level directories.
//                         // Ex: `site-packages/foo.py` is not a valid package.
//                         if alias_path.components().count() <= 1 {
//                             return None;
//                         }

//                         // Extract the root directory name.
//                         alias_path.components().next().and_then(|comp| {
//                             if let Component::Normal(root_dir) = comp {
//                                 root_dir.to_str().map(ToString::to_string)
//                             } else {
//                                 None
//                             }
//                         })
//                     })
//                     .collect();

//                 if aliases.is_empty() {
//                     continue;
//                 }

//                 let size = aliases
//                     .iter()
//                     .map(|alias| path.join(alias))
//                     .map(|potential_path| get_size(&potential_path).unwrap_or(0))
//                     .sum();

//                 let package = PackageBuilder::new(pkg_id, aliases, size).build();

//                 self.add_package(package);
//             }
//         }
//         Ok(())
//     }

//     // For `testing` purposes ONLY. Not intended to be public facing API.
//     #[cfg(test)]
//     pub fn _mapping(&self) -> &Vec<Package> {
//         &self.manifest
//     }
// }
