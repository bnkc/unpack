use crate::exit_codes::ExitCode;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Eq, Hash)]

pub struct Dependency {
    pub name: String,
    pub type_: Option<String>,
    pub version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]

pub struct Outcome {
    pub success: bool,
    pub unused_deps: HashSet<Dependency>,
    pub note: Option<String>,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum OutputKind {
    Human,
    Json,
}

impl Outcome {
    pub fn print_result(&self, output_kind: OutputKind, stdout: impl Write) -> Result<ExitCode> {
        match output_kind {
            OutputKind::Human => self.print_human(stdout),
            OutputKind::Json => self.print_json(stdout),
        }
    }

    fn group_unused_deps(&self) -> HashMap<Option<String>, Vec<&Dependency>> {
        let mut res: HashMap<Option<String>, Vec<&Dependency>> = HashMap::new();
        for dep in &self.unused_deps {
            res.entry(dep.type_.clone())
                .or_insert_with(Vec::new)
                .push(dep);
        }
        res
    }

    pub fn print_human(&self, mut stdout: impl Write) -> Result<ExitCode> {
        if self.success {
            writeln!(stdout, "All dependencies are used!")?;
        } else {
            writeln!(stdout, "\nUnused dependencies:")?;

            let grouped_deps = Outcome::group_unused_deps(&self);
            for (type_, deps) in &grouped_deps {
                let type_label = type_.as_ref().map_or("General", String::as_str);
                writeln!(stdout, "\n[{}]", type_label)?;

                let mut sorted_deps = deps.iter().collect::<Vec<_>>();
                sorted_deps.sort_by_key(|dep| &dep.name);

                for (i, dep) in sorted_deps.iter().enumerate() {
                    let is_last = i == sorted_deps.len() - 1;
                    let joint = if is_last { '└' } else { '├' };
                    if let Some(version) = &dep.version {
                        writeln!(stdout, "{}─── {} = \"{}\"", joint, dep.name, version)?;
                    } else {
                        writeln!(stdout, "{}─── {}", joint, dep.name)?;
                    }
                }
            }

            if let Some(note) = &self.note {
                writeln!(stdout, "\n{}", note)?;
            }
        }
        stdout.flush()?;
        Ok(ExitCode::Success)
    }

    fn print_json(&self, mut stdout: impl Write) -> Result<ExitCode> {
        let json = serde_json::to_string(self).expect("Failed to serialize to JSON.");
        writeln!(stdout, "{}", json)?;
        stdout.flush()?;
        Ok(ExitCode::Success)
    }
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct SitePackages {
    pub paths: Vec<PathBuf>,
    pub venv: Option<String>,
}

#[derive(clap::ValueEnum, Debug, PartialEq, Clone)]
pub enum PackageState {
    /// The dependency is installed, actively used in the project, and correctly listed in pyproject.toml.
    /// This state indicates a fully integrated and properly managed dependency.
    Verified,
    /// The dependency is installed and listed in pyproject.toml but is not actively used in the project.
    /// Ideal for identifying and possibly removing unnecessary dependencies to clean up the project.
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

#[derive(Debug, PartialEq, Clone)]
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
        state: PackageState,
    ) -> Vec<PackageInfo> {
        match state {
            PackageState::Verified => self.find_verified(pyproject_deps, imports),
            PackageState::Unused => self.find_unused(pyproject_deps, imports),
            PackageState::Untracked => self.find_untracked(pyproject_deps, imports),
            PackageState::Uninstalled => self.find_uninstalled(pyproject_deps, imports),
        }
    }

    pub fn find_all_packages(
        &self,
        pyproject_deps: &HashSet<Dependency>,
        imports: &HashSet<String>,
    ) -> Vec<PackageInfo> {
        let mut all_packages = Vec::new();
        all_packages.extend(self.find_verified(pyproject_deps, imports));
        all_packages.extend(self.find_unused(pyproject_deps, imports));
        all_packages.extend(self.find_untracked(pyproject_deps, imports));
        all_packages.extend(self.find_uninstalled(pyproject_deps, imports));
        all_packages
    }

    fn find_verified(
        &self,
        pyproject_deps: &HashSet<Dependency>,
        imports: &HashSet<String>,
    ) -> Vec<PackageInfo> {
        let mut verified_packages = Vec::new();
        for dep in pyproject_deps {
            if let Some(import_names) = self.manifest.get(&dep.name) {
                if !import_names.is_disjoint(imports) {
                    verified_packages.push(PackageInfo {
                        name: dep.name.clone(),
                        state: PackageState::Verified,
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
    ) -> Vec<PackageInfo> {
        let mut unused_packages = Vec::new();
        for dep in pyproject_deps {
            if let Some(import_names) = self.manifest.get(&dep.name) {
                if import_names.is_disjoint(imports) {
                    unused_packages.push(PackageInfo {
                        name: dep.name.clone(),
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
    ) -> Vec<PackageInfo> {
        let deps_names: HashSet<String> =
            pyproject_deps.iter().map(|dep| dep.name.clone()).collect();
        let mut untracked_packages = Vec::new();

        for (pkg_name, import_names) in &self.manifest {
            if !import_names.is_disjoint(imports) && !deps_names.contains(pkg_name) {
                untracked_packages.push(PackageInfo {
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
    ) -> Vec<PackageInfo> {
        let mut uninstalled_packages = Vec::new();
        for dep in pyproject_deps {
            if !self.manifest.contains_key(&dep.name)
                && imports.contains(&dep.name.replace("-", "_"))
            {
                uninstalled_packages.push(PackageInfo {
                    name: dep.name.clone(),
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
