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
        let mut deps_by_type: HashMap<Option<String>, Vec<&Dependency>> = HashMap::new();
        for dep in &self.unused_deps {
            deps_by_type
                .entry(dep.type_.clone())
                .or_insert_with(Vec::new)
                .push(dep);
        }
        deps_by_type
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

#[derive(Deserialize, Debug, PartialEq, Clone)]

pub struct InstalledPackages {
    mapping: HashMap<String, HashSet<String>>,
}

impl InstalledPackages {
    pub fn new() -> Self {
        InstalledPackages {
            mapping: HashMap::new(),
        }
    }
    pub fn add_pkg(&mut self, pkg_name: String, import_names: HashSet<String>) {
        let pkg_name = pkg_name.replace("_", "-");
        self.mapping.insert(pkg_name, import_names);
    }

    pub fn filter_used_pkgs(&self, used_imports: &HashSet<String>) -> HashSet<String> {
        self.mapping
            .iter()
            .filter(|(_pkg_name, import_names)| !import_names.is_disjoint(used_imports))
            .map(|(pkg_name, _)| pkg_name)
            .cloned()
            .collect()
    }

    // For `testing` purposes ONLY. Not intended to be public facing API.
    #[cfg(test)]
    pub fn _mapping(&self) -> &HashMap<String, HashSet<String>> {
        &self.mapping
    }
}
