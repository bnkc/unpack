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

// We need to come back here and clean this up
impl Outcome {
    pub fn print(&self, mut stdout: impl Write) -> io::Result<()> {
        if self.success {
            writeln!(stdout, "All deps seem to have been used.")?;
        } else {
            writeln!(stdout, "Unused dependencies:")?;

            let mut deps_by_type: HashMap<Option<String>, Vec<&Dependency>> = HashMap::new();
            for dep in &self.unused_deps {
                deps_by_type
                    .entry(dep.type_.clone())
                    .or_insert_with(Vec::new)
                    .push(dep);
            }

            let edge_and_joint = |is_last: bool| {
                if is_last {
                    (' ', '└')
                } else {
                    ('│', '├')
                }
            };

            let package_id = std::env::var("CARGO_PKG_NAME").unwrap();
            let version = std::env::var("CARGO_PKG_VERSION").unwrap();

            writeln!(stdout, "`{}`", format!("{} {}", package_id, version))?;

            for (type_, deps) in deps_by_type.iter() {
                let type_label = type_.as_ref().map_or("General", String::as_str);
                writeln!(stdout, "[{}]", type_label)?;

                // Sort dependencies by name for consistent output
                let mut sorted_deps = deps.iter().collect::<Vec<_>>();
                sorted_deps.sort_by_key(|dep| &dep.name);

                for (i, dep) in sorted_deps.iter().enumerate() {
                    let (_, joint) = edge_and_joint(i == sorted_deps.len() - 1);
                    writeln!(stdout, "{}─── {}", joint, dep.name)?;
                }
            }

            if let Some(note) = &self.note {
                writeln!(stdout, "{}", note)?;
            }
        }
        stdout.flush()
    }
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct SitePackages {
    pub paths: Vec<String>,
    pub venv_name: Option<String>,
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
