use crate::exit_codes::ExitCode;
use crate::Config;
use anyhow::Result;

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::PathBuf;
use tabled::{settings::Style, Table, Tabled};

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct SitePackages {
    pub paths: Vec<PathBuf>,
    pub venv: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Eq, Hash)]
pub struct Dependency {
    pub id: String,
    pub version: Option<String>,
    pub category: Option<String>,
}

#[derive(Serialize, Deserialize, clap::ValueEnum, Debug, PartialEq, Eq, Clone, Hash)]
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
    /// The dependency is used in the project and listed in pyproject.toml but is not installed in the
    /// local environment. Useful for identifying missing installations that are expected by the project,
    /// potentially leading to runtime errors.
    Uninstalled,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Hash)]
pub struct PackageInfo {
    name: String,
    state: PackageState,
    dependency: Option<Dependency>,
}

/// Represents the set of packages installed and the `potential` imports or modules they provide.
#[derive(Deserialize, Debug, PartialEq, Clone, Default)]
pub struct Packages {
    manifest: HashMap<String, HashSet<String>>,
}

impl Packages {
    pub fn add_pkg(&mut self, pkg_name: String, import_names: HashSet<String>) {
        let pkg_name = pkg_name.replace("_", "-");
        self.manifest.insert(pkg_name, import_names);
    }

    pub fn find_packages_by_state(
        &self,
        pyproject_deps: &HashSet<Dependency>,
        imports: &HashSet<String>,
        state: &PackageState,
    ) -> HashSet<PackageInfo> {
        match state {
            PackageState::Used => self.find_used(pyproject_deps, imports),
            PackageState::Unused => self.find_unused(pyproject_deps, imports),
            PackageState::Untracked => self.find_untracked(pyproject_deps, imports),
            PackageState::Uninstalled => self.find_uninstalled(pyproject_deps, imports),
        }
    }

    // Come back to this
    fn find_all_packages(
        &self,
        pyproject_deps: &HashSet<Dependency>,
        imports: &HashSet<String>,
    ) -> HashSet<PackageInfo> {
        let mut all_packages = HashSet::new();
        all_packages.extend(self.find_used(pyproject_deps, imports));
        all_packages.extend(self.find_unused(pyproject_deps, imports));
        all_packages.extend(self.find_untracked(pyproject_deps, imports));
        all_packages.extend(self.find_uninstalled(pyproject_deps, imports));
        all_packages
    }

    fn find_used(
        &self,
        pyproject_deps: &HashSet<Dependency>,
        imports: &HashSet<String>,
    ) -> HashSet<PackageInfo> {
        let mut verified_packages = HashSet::new();
        for dep in pyproject_deps {
            // Check if a dependency is existent in the manifest (local packages)
            if let Some(import_names) = self.manifest.get(&dep.id) {
                // Check if the dependency is used in the project
                if !import_names.is_disjoint(imports) {
                    verified_packages.insert(PackageInfo {
                        name: dep.id.clone(),
                        state: PackageState::Used,
                        dependency: Some(dep.clone()),
                    });
                }
            }
        }
        verified_packages
    }

    fn find_unused(
        &self,
        pyproject_deps: &HashSet<Dependency>,
        imports: &HashSet<String>,
    ) -> HashSet<PackageInfo> {
        let mut unused_packages = HashSet::new();
        for dep in pyproject_deps {
            // Check if a dependency is existent in the manifest (local packages)
            if let Some(import_names) = self.manifest.get(&dep.id) {
                // Check if the dependency is not used in the project
                if import_names.is_disjoint(imports) {
                    unused_packages.insert(PackageInfo {
                        name: dep.id.clone(),
                        state: PackageState::Unused,
                        dependency: Some(dep.clone()),
                    });
                }
            }
        }
        unused_packages
    }

    fn find_untracked(
        &self,
        pyproject_deps: &HashSet<Dependency>,
        imports: &HashSet<String>,
    ) -> HashSet<PackageInfo> {
        let deps_names: HashSet<String> = pyproject_deps.iter().map(|dep| dep.id.clone()).collect();

        let mut untracked_packages = HashSet::new();
        // Check if any of the imports are not in the manifest (local packages)
        for (pkg_name, import_names) in &self.manifest {
            //  Check if the package is not listed in the pyproject.toml
            if !import_names.is_disjoint(imports) && !deps_names.contains(pkg_name) {
                untracked_packages.insert(PackageInfo {
                    name: pkg_name.clone(),
                    state: PackageState::Untracked,
                    dependency: None,
                });
            }
        }
        untracked_packages
    }

    fn find_uninstalled(
        &self,
        pyproject_deps: &HashSet<Dependency>,
        imports: &HashSet<String>,
    ) -> HashSet<PackageInfo> {
        let mut uninstalled_packages = HashSet::new();
        for dep in pyproject_deps {
            // Check if a dependency is existent in the manifest (local packages) and is not installed
            if !self.manifest.contains_key(&dep.id) && imports.contains(&dep.id.replace("-", "_")) {
                uninstalled_packages.insert(PackageInfo {
                    name: dep.id.clone(),
                    state: PackageState::Uninstalled,
                    dependency: Some(dep.clone()),
                });
            }
        }
        uninstalled_packages
    }

    // For `testing` purposes ONLY. Not intended to be public facing API.
    #[cfg(test)]
    pub fn _mapping(&self) -> &HashMap<String, HashSet<String>> {
        &self.manifest
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum OutputKind {
    /// Human-readable output format.
    Human,
    /// JSON output format.
    Json,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct Outcome {
    pub success: bool,
    pub packages: HashSet<PackageInfo>,
    pub note: Option<String>,
}

// impl Outcome {
//     pub fn print_report(&self, config: &Config, stdout: impl Write) -> Result<ExitCode> {
//         match config.output {
//             OutputKind::Human => self.pretty_print(stdout, config),
//             OutputKind::Json => self.json_print(stdout),
//         }
//     }

//     fn json_print(&self, mut stdout: impl Write) -> Result<ExitCode> {
//         let json = serde_json::to_string(self).expect("Failed to serialize to JSON.");
//         writeln!(stdout, "{}", json)?;
//         stdout.flush()?;
//         Ok(ExitCode::Success)
//     }

//     // Handles the pretty print for Human-readable output
//     fn pretty_print(&self, mut stdout: impl Write, config: &Config) -> Result<ExitCode> {
//         if self.success {
//             writeln!(stdout, "All dependencies are correctly managed!")?;
//         } else {
//             writeln!(stdout, "\n{:?} Dependencies", config.package_state)?;

//             match config.package_state {
//                 PackageState::Untracked => self.print_untracked(&mut stdout)?,
//                 _ => self.print_other(&mut stdout)?,
//             }

//             if let Some(note) = &self.note {
//                 writeln!(stdout, "\nNote: {}", note)?;
//             }
//         }

//         stdout.flush()?;
//         Ok(ExitCode::Success)
//     }

//     // Specific printing logic for Untracked dependencies
//     fn print_untracked(&self, stdout: &mut impl Write) -> Result<()> {
//         for (i, dep) in self.packages.iter().enumerate() {
//             let is_last = i == self.packages.len() - 1;
//             let joint = if is_last { '└' } else { '├' };
//             writeln!(stdout, "{}─── {}", joint, dep.name)?;
//         }
//         Ok(())
//     }

//     // General printing logic for other dependencies (Used, Unused, Uninstalled)
//     fn print_other(&self, stdout: &mut impl Write) -> Result<()> {
//         let grouped_deps = self.group_by_category();

//         for (type_, deps) in grouped_deps {
//             let type_label = type_.as_ref().map_or("General", String::as_str);
//             writeln!(stdout, "\n[{}]", type_label)?;

//             for (i, dep) in deps.iter().enumerate() {
//                 let is_last = i == deps.len() - 1;
//                 let joint = if is_last { '└' } else { '├' };
//                 match dep.version {
//                     Some(ref version) => {
//                         writeln!(stdout, "{}─── {} = \"{}\"", joint, dep.id, version)?
//                     }
//                     None => writeln!(stdout, "{}─── {}", joint, dep.id)?,
//                 }
//             }
//         }

//         Ok(())
//     }

//     fn group_by_category(&self) -> HashMap<Option<String>, Vec<&Dependency>> {
//         let mut res: HashMap<Option<String>, Vec<&Dependency>> = HashMap::new();
//         for p in &self.packages {
//             let category = p.dependency.as_ref().and_then(|dep| dep.category.clone());
//             res.entry(category)
//                 .or_insert_with(Vec::new)
//                 .push(p.dependency.as_ref().unwrap());
//         }
//         res
//     }
// }

#[derive(Tabled)]
struct DependencyRecord {
    name: String,
    version: String,
    category: String,
}

impl Outcome {
    // Simplified entry point for printing the report
    pub fn print_report(&self, config: &Config, stdout: impl Write) -> Result<ExitCode> {
        match config.output {
            OutputKind::Human => self.pretty_print(stdout, config),
            OutputKind::Json => self.json_print(stdout),
        }
    }

    fn pretty_print(&self, mut stdout: impl Write, config: &Config) -> Result<ExitCode> {
        if self.success {
            writeln!(stdout, "All dependencies are correctly managed!")?;
        } else {
            writeln!(stdout, "{:?} Dependencies", config.package_state)?;

            match config.package_state {
                PackageState::Untracked => self.print_untracked(&mut stdout)?,
                _ => self.print_tracked(&mut stdout)?,
            }

            if let Some(note) = &self.note {
                writeln!(stdout, "\nNote: {}", note)?;
            }
        }

        stdout.flush()?;
        Ok(ExitCode::Success)
    }

    fn print_untracked(&self, stdout: &mut impl Write) -> Result<()> {
        let records: Vec<DependencyRecord> = self
            .packages
            .iter()
            .map(|pkg_info| DependencyRecord {
                name: pkg_info.name.clone(),
                version: String::from("N/A"),
                category: String::from("N/A"),
            })
            .collect();

        let table = Table::new(records).to_string();
        write!(stdout, "{}", table)?;
        Ok(())
    }

    fn print_tracked(&self, stdout: &mut impl Write) -> Result<(), std::io::Error> {
        // Group dependencies by category
        let mut category_groups: HashMap<String, Vec<DependencyRecord>> = HashMap::new();
        for pkg_info in &self.packages {
            if let Some(ref dep) = pkg_info.dependency {
                category_groups
                    .entry(dep.category.clone().unwrap_or_else(|| "N/A".to_string()))
                    .or_default()
                    .push(DependencyRecord {
                        name: dep.id.clone(),
                        version: dep.version.clone().unwrap_or_else(|| "N/A".to_string()),
                        category: dep.category.clone().unwrap_or_else(|| "N/A".to_string()),
                    });
            }
        }

        // Print tables for each category
        for (category, records) in category_groups {
            writeln!(stdout, "\n[{}]\n", category)?;
            let table = Table::new(&records)
                .with(Style::ascii_rounded())
                .to_string();
            writeln!(stdout, "{}", table)?;
        }

        Ok(())
    }

    // JSON printing remains unchanged
    fn json_print(&self, mut stdout: impl Write) -> Result<ExitCode> {
        let json = serde_json::to_string(self).expect("Failed to serialize to JSON.");
        writeln!(stdout, "{}", json)?;
        stdout.flush()?;
        Ok(ExitCode::Success)
    }
}
