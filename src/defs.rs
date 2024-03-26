extern crate bytesize;

use crate::exit_codes::ExitCode;
use crate::Config;
use anyhow::Result;
use bytesize::ByteSize;

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::collections::{HashMap, HashSet};

use std::io::Write;
use std::path::PathBuf;
use std::vec;
use tabled::settings::Panel;
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
pub struct PackageMetadata {
    pub id: String,
    pub size: u64,
    pub aliases: BTreeSet<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Hash)]
pub struct PackageBuilder {
    dependency: Option<Dependency>,
    metadata: PackageMetadata,
    state: PackageState,
}

#[derive(Deserialize, Debug, PartialEq, Clone, Default)]
pub struct Packages {
    manifest: HashSet<PackageMetadata>,
}

impl Packages {
    pub fn add_pkg(&mut self, metadata: PackageMetadata) {
        let name = metadata.id.replace("_", "-");
        self.manifest.insert(PackageMetadata {
            id: name,
            size: metadata.size,
            aliases: metadata.aliases,
        });
    }

    pub fn get_packages_by_state(
        &self,
        pyproject_deps: &HashSet<Dependency>,
        imports: &BTreeSet<String>,
        state: &PackageState,
    ) -> HashSet<PackageBuilder> {
        match state {
            PackageState::Used => self.get_used(pyproject_deps, imports),
            PackageState::Unused => self.get_unused(pyproject_deps, imports),
            PackageState::Untracked => self.get_untracked(pyproject_deps, imports),
            PackageState::Uninstalled => self.get_uninstalled(pyproject_deps, imports),
        }
    }

    /// Returns all packages in the manifest that are in the given state.
    pub fn get_all_packages(
        &self,
        pyproject_deps: &HashSet<Dependency>,
        imports: &BTreeSet<String>,
    ) -> HashSet<PackageBuilder> {
        let mut all_packages = HashSet::new();
        all_packages.extend(self.get_used(pyproject_deps, imports));
        all_packages.extend(self.get_unused(pyproject_deps, imports));
        all_packages.extend(self.get_untracked(pyproject_deps, imports));
        all_packages.extend(self.get_uninstalled(pyproject_deps, imports));
        all_packages
    }

    /// Retrieves a set of `PackageBuilder` instances representing the "used" dependencies.
    ///
    /// A "used" dependency is defined as a dependency listed in `pyproject_deps`
    /// that has at least one alias present in the `imports` set, indicating it is actively
    /// used in the project.
    ///
    /// # Arguments
    ///
    /// * `pyproject_deps` - A set of dependencies as defined in the project's pyproject.toml.
    /// * `imports` - A set of import statements or module names actually used in the project.
    ///
    /// # Returns
    ///
    /// A HashSet of `PackageBuilder` instances, each representing a used dependency.
    fn get_used(
        &self,
        pyproject_deps: &HashSet<Dependency>,
        imports: &BTreeSet<String>,
    ) -> HashSet<PackageBuilder> {
        pyproject_deps
            .iter()
            .filter_map(|dep| {
                // Attempt to find a matching package in the manifest by `ID` that is also used in imports.
                self.manifest
                    .iter()
                    .find(|pkg| pkg.id == dep.id && !pkg.aliases.is_disjoint(imports))
                    // If a match is found, construct a `PackageBuilder` indicating its usage.
                    .map(|pkg| PackageBuilder {
                        metadata: pkg.clone(),
                        state: PackageState::Used,
                        dependency: Some(dep.clone()),
                    })
            })
            .collect()
    }

    /// Retrieves a set of `PackageBuilder` instances representing the "unused" dependencies.
    ///
    /// An "unused" dependency is defined as a dependency listed in `pyproject_deps` that has no
    /// aliases present in the `imports` set, indicating it is not actively used in the project.
    ///
    /// # Arguments
    ///
    /// * `pyproject_deps` - A set of dependencies as defined in the project's pyproject.toml.
    /// * `imports` - A set of import statements or module names actually used in the project.
    ///
    /// # Returns
    ///
    /// A HashSet of `PackageBuilder` instances, each representing an unused dependency.
    fn get_unused(
        &self,
        pyproject_deps: &HashSet<Dependency>,
        imports: &BTreeSet<String>,
    ) -> HashSet<PackageBuilder> {
        pyproject_deps
            .iter()
            .filter_map(|dep| {
                // Attempt to find a matching package in the manifest by `ID` that is not used in imports.
                self.manifest
                    .iter()
                    .find(|pkg| pkg.id == dep.id && pkg.aliases.is_disjoint(imports))
                    // If a match is found, construct a `PackageBuilder` indicating its unused status.
                    .map(|pkg| PackageBuilder {
                        metadata: pkg.clone(),
                        state: PackageState::Unused,
                        dependency: Some(dep.clone()),
                    })
            })
            .collect()
    }

    /// Retrieves a set of `PackageBuilder` instances representing the "untracked" dependencies.
    ///
    /// An "untracked" dependency is defined as a dependency that is used in the project but is not listed
    /// in the `pyproject_deps` set, indicating it is not formally declared in the project's pyproject.toml.
    ///
    /// # Arguments
    ///
    /// * `pyproject_deps` - A set of dependencies as defined in the project's pyproject.toml.
    /// * `imports` - A set of import statements or module names actually used in the project.
    ///
    /// # Returns
    ///
    /// A HashSet of `PackageBuilder` instances, each representing an untracked dependency.
    fn get_untracked(
        &self,
        pyproject_deps: &HashSet<Dependency>,
        imports: &BTreeSet<String>,
    ) -> HashSet<PackageBuilder> {
        let dep_ids: HashSet<String> = pyproject_deps.iter().map(|dep| dep.id.clone()).collect();

        imports
            .iter()
            .filter_map(|import| {
                // Attempt to find a matching package in the manifest by `ID` that is not listed in `pyproject_deps`.
                self.manifest
                    .iter()
                    .find(|pkg| pkg.aliases.contains(import) && !dep_ids.contains(&pkg.id))
                    // If a match is found, construct a `PackageBuilder` indicating its untracked status.
                    .map(|pkg| PackageBuilder {
                        metadata: pkg.clone(),
                        state: PackageState::Untracked,
                        dependency: None,
                    })
            })
            .collect()
    }

    /// Retrieves a set of `PackageBuilder` instances representing the "uninstalled" dependencies.
    ///
    /// An "uninstalled" dependency is defined as a dependency listed in `pyproject_deps` that is not
    /// present in the manifest and is used in the project, indicating it is not installed in the local
    /// environment.
    ///
    /// # Arguments
    ///
    /// * `pyproject_deps` - A set of dependencies as defined in the project's pyproject.toml.
    /// * `imports` - A set of import statements or module names actually used in the project.
    ///
    /// # Returns
    ///
    /// A HashSet of `PackageBuilder` instances, each representing an uninstalled dependency.
    fn get_uninstalled(
        &self,
        pyproject_deps: &HashSet<Dependency>,
        imports: &BTreeSet<String>,
    ) -> HashSet<PackageBuilder> {
        pyproject_deps
            .iter()
            .filter_map(|dep| {
                // Attempt to find a matching package in the manifest by `ID` that is not installed.
                self.manifest
                    .iter()
                    .find(|pkg| pkg.id == dep.id && !imports.contains(&pkg.id.replace("-", "_")))
                    // If a match is found, construct a `PackageBuilder` indicating its uninstalled status.
                    .map(|pkg| PackageBuilder {
                        metadata: pkg.clone(),
                        state: PackageState::Uninstalled,
                        dependency: Some(dep.clone()),
                    })
            })
            .collect()
    }

    // For `testing` purposes ONLY. Not intended to be public facing API.
    #[cfg(test)]
    pub fn _mapping(&self) -> &HashSet<PackageMetadata> {
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
    pub packages: HashSet<PackageBuilder>,
    pub note: Option<String>,
}

#[derive(Tabled)]
struct Record {
    name: String,
    version: String,
    size: String,
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
            writeln!(stdout, "\n{:?} Dependencies", config.package_state)?;

            match config.package_state {
                PackageState::Untracked => self.print_untracked(&mut stdout)?,
                _ => self.print(&mut stdout)?,
            }

            if let Some(note) = &self.note {
                writeln!(stdout, "\nNote: {}", note)?;
            }
        }

        stdout.flush()?;
        Ok(ExitCode::Success)
    }

    fn print_untracked(&self, stdout: &mut impl Write) -> Result<()> {
        let records: Vec<Record> = self
            .packages
            .iter()
            .map(|pkg_info| Record {
                name: pkg_info.metadata.id.clone(),
                version: String::from("N/A"),
                size: ByteSize::b(pkg_info.metadata.size).to_string_as(true),
            })
            .collect();

        let table = Table::new(records).to_string();
        write!(stdout, "{}", table)?;
        Ok(())
    }

    fn print(&self, stdout: &mut impl Write) -> Result<(), std::io::Error> {
        let mut category_groups: HashMap<String, Vec<Record>> = HashMap::new();
        for pkg_info in &self.packages {
            if let Some(ref dep) = pkg_info.dependency {
                category_groups
                    .entry(dep.category.clone().unwrap_or_else(|| "N/A".to_string()))
                    .or_default()
                    .push(Record {
                        name: dep.id.clone(),
                        version: dep.version.clone().unwrap_or_else(|| "N/A".to_string()),
                        size: ByteSize::b(pkg_info.metadata.size).to_string_as(true),
                    });
            }
        }

        for (category, records) in category_groups {
            let mut table = Table::new(&records);

            table
                .with(Panel::header(category))
                .with(Style::ascii())
                .with(Panel::footer("End of table"));

            writeln!(stdout, "\n{}", table)?;
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
