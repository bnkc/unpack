extern crate bytesize;

use crate::exit_codes::ExitCode;
use crate::Config;
use anyhow::{anyhow, bail, Result};
use bytesize::ByteSize;

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::collections::{HashMap, HashSet};

use std::io::Write;
use std::path::PathBuf;
use std::{default, vec};
use tabled::settings::Panel;
use tabled::{settings::Style, Table, Tabled}; // Add missing imports

#[derive(Deserialize, Debug, PartialEq, Clone)]
// pub struct SitePackages(HashSet<PathBuf>);
pub struct SitePackage {
    paths: HashSet<PathBuf>,
}

impl SitePackage {
    pub fn new(paths: HashSet<PathBuf>) -> Result<Self> {
        let validated_paths: HashSet<PathBuf> =
            paths.into_iter().filter(|path| path.exists()).collect();

        if validated_paths.is_empty() {
            bail!("No site-packages found. Are you sure you are in a virtual environment?");
        }

        Ok(SitePackage {
            paths: validated_paths,
        })
    }

    pub fn paths(&self) -> &HashSet<PathBuf> {
        &self.paths
    }
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
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Eq, Hash)]
pub struct Dependency {
    pub id: String,
    pub version: Option<String>,
    pub category: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Eq, Hash)]
pub struct DependencyBuilder {
    pub id: String,
    pub version: Option<String>,
    pub category: Option<String>,
}
impl DependencyBuilder {
    pub fn new(id: String) -> Self {
        Self {
            id,
            version: None,
            category: None,
        }
    }

    pub fn version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

    pub fn category(mut self, category: String) -> Self {
        self.category = Some(category);
        self
    }

    pub fn build(self) -> Dependency {
        Dependency {
            id: self.id,
            version: self.version,
            category: self.category,
        }
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

impl Packages {
    pub fn add_package(&mut self, package: Package) {
        self.manifest.push(package);
    }

    pub fn get_packages(
        &self,
        config: Config,
        deps: &HashSet<Dependency>,
        imports: &HashSet<String>,
    ) -> Vec<Package> {
        match config.package_state {
            PackageState::Used => self.get_used(deps, imports),
            PackageState::Unused => self.get_unused(deps, imports),
            PackageState::Untracked => self.get_untracked(deps, imports),
        }
    }

    fn get_used(&self, deps: &HashSet<Dependency>, imports: &HashSet<String>) -> Vec<Package> {
        deps.iter()
            .filter_map(|dep| {
                self.manifest
                    .iter()
                    .find(|pkg| pkg.id == dep.id && !pkg.aliases.is_disjoint(imports))
                    .map(|pkg| {
                        let mut pkg_clone = pkg.clone();
                        pkg_clone.dependency = Some(dep.clone());
                        pkg_clone
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
                        let mut pkg_clone = pkg.clone();
                        pkg_clone.dependency = Some(dep.clone());
                        pkg_clone
                    })
            })
            .collect()
    }

    fn get_untracked(&self, deps: &HashSet<Dependency>, imports: &HashSet<String>) -> Vec<Package> {
        let dep_ids: HashSet<String> = deps.iter().map(|dep| dep.id.clone()).collect();

        self.manifest
            .iter()
            .filter(|pkg| !pkg.aliases.is_disjoint(imports) && !dep_ids.contains(&pkg.id))
            .map(|pkg| {
                // THIS IS WRONG

                let pkg_clone = pkg.clone();
                // pkg_clone.state = PackageState::Untracked;
                pkg_clone
            })
            .collect()
    }

    // For `testing` purposes ONLY. Not intended to be public facing API.
    #[cfg(test)]
    pub fn _mapping(&self) -> &Vec<Package> {
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
pub struct Analysis {
    pub success: bool,
    pub packages: Vec<Package>,
    pub note: Option<String>,
}

#[derive(Tabled)]
struct Record {
    name: String,
    version: String,
    size: String,
}

impl Analysis {
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
                name: pkg_info.package.id.clone(),
                version: String::from("N/A"),
                size: ByteSize::b(pkg_info.package.size).to_string_as(true),
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
                        size: ByteSize::b(pkg_info.package.size).to_string_as(true),
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
