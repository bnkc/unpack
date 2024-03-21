use serde::Deserialize;

use std::collections::{HashMap, HashSet};
use std::io::{self, Write};

#[derive(Deserialize, Debug, PartialEq, Clone, Eq, Hash)]
pub struct Dependency {
    pub name: String,
    pub type_: Option<String>,
    pub version: Option<String>,
}

#[derive(Deserialize, Debug, PartialEq, Clone, Default)]
pub struct Outcome {
    pub success: bool,
    pub unused_deps: HashSet<Dependency>,
    pub note: Option<String>,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
enum OutputKind {
    Human,
    Json,
}

impl Outcome {
    // pub fn print(&self, output_kind: OutputKind, stdout: impl Write) -> io::Result<()> {
    //     match output_kind {
    //         OutputKind::Human => self.print_human(stdout),
    //         OutputKind::Json => self.print_json(stdout),
    //     }
    // }

    fn edge_and_joint(is_last: bool) -> (char, char) {
        if is_last {
            (' ', '└')
        } else {
            ('│', '├')
        }
    }

    pub fn print_human(&self, mut stdout: impl Write) -> io::Result<()> {
        if self.success {
            writeln!(stdout, "All dependencies are used!")?;
        } else {
            writeln!(stdout, "\nUnused dependencies:")?;

            // Group dependencies by type
            let mut deps_by_type: HashMap<Option<String>, Vec<&Dependency>> = HashMap::new();
            for dep in &self.unused_deps {
                deps_by_type
                    .entry(dep.type_.clone())
                    .or_insert_with(Vec::new)
                    .push(dep);
            }

            // Iterate over grouped dependencies
            for (type_, deps) in &deps_by_type {
                let type_label = type_.as_ref().map_or("General", String::as_str);
                writeln!(stdout, "\n[{}]", type_label)?;

                // Sort dependencies by name for consistent output
                let mut sorted_deps = deps.iter().collect::<Vec<_>>();
                sorted_deps.sort_by_key(|dep| &dep.name);

                for (i, dep) in sorted_deps.iter().enumerate() {
                    let is_last = i == sorted_deps.len() - 1;
                    let (_, joint) = Outcome::edge_and_joint(is_last);
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
        Ok(())
    }

    #[allow(dead_code)]
    pub fn print_json(&self, mut stdout: impl Write) -> io::Result<()> {
        stdout.flush()
    }
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct SitePackages {
    pub paths: Vec<String>,
    pub venv: Option<String>,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct InstalledPackages {
    pub mapping: HashMap<String, HashSet<String>>,
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
}
