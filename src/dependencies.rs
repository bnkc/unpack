extern crate bytesize;
extern crate fs_extra;
// extern crate test;

use std::collections::HashSet;
use std::fs;
use std::hash::Hash;
use std::path::Path;
use std::str;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Eq, Hash)]
pub struct Dependency {
    id: String,
    version: Option<String>,
    category: Option<String>,
}

impl Dependency {
    pub fn id(&self) -> &str {
        &self.id
    }
}

pub struct DependencyBuilder {
    id: String,
    version: Option<String>,
    category: Option<String>,
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

struct Dependencies {
    manifest: HashSet<Dependency>,
    current_path: Vec<String>,
}

impl Dependencies {
    fn new() -> Self {
        Dependencies {
            manifest: HashSet::new(),
            current_path: Vec::new(),
        }
    }

    fn visit_table(&mut self, key: &str, table: &toml::value::Table) {
        self.current_path.push(key.to_string());

        // Check if we're inside a dependencies table
        let current_path_str = self.current_path.join(".");
        if current_path_str.ends_with("dependencies") {
            for (dep_name, dep_value) in table {
                self.visit_value(dep_name, dep_value);
            }
        } else {
            for (k, v) in table {
                match v {
                    toml::Value::Table(t) => self.visit_table(k, t),
                    _ => self.visit_value(k, v),
                }
            }
        }

        // Backtrack on the path
        self.current_path.pop();
    }

    fn visit_value(&mut self, key: &str, value: &toml::Value) {
        if let toml::Value::String(version) = value {
            let category = self.current_path.join(".");
            let category = category.strip_prefix(".").unwrap_or(&category).to_string();

            self.manifest.insert(
                DependencyBuilder::new(key.to_string())
                    .version(version.clone())
                    .category(category)
                    .build(),
            );
        }
    }
}

// This function reads a TOML file at the specified path and returns a HashSet of Dependency structs.
// It uses the toml crate to parse the TOML content.
pub fn get_dependencies(path: &Path) -> Result<HashSet<Dependency>> {
    let toml_str = fs::read_to_string(path)
        .with_context(|| format!("Failed to read TOML file at {:?}", path))?;

    let toml_value: toml::Value =
        toml::from_str(&toml_str).with_context(|| "Failed to parse TOML content")?;

    let mut deps = Dependencies::new();

    if let toml::Value::Table(table) = toml_value {
        deps.visit_table("", &table);
    }

    Ok(deps.manifest)
}
