use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[derive(Deserialize, Debug, PartialEq, Clone, Eq, Hash)]
pub struct Dependency {
    pub name: String,
    pub type_: Option<String>,
}

#[derive(Deserialize, Debug, PartialEq, Clone, Default)]
pub struct Outcome {
    pub success: bool,
    pub unused_deps: HashSet<Dependency>,
    pub note: Option<String>,
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
    pub fn get_pkg(&self, pkg_name: &str) -> Option<&HashSet<String>> {
        self.mapping.get(pkg_name)
    }

    pub fn remove_pkg(&mut self, pkg_name: &str) -> Option<HashSet<String>> {
        self.mapping.remove(pkg_name)
    }
}
