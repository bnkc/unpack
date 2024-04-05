extern crate bytesize;
extern crate fs_extra;

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

    pub fn version(&self) -> Option<&String> {
        self.version.as_ref()
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::tempdir;

    /// Helper function to create a temporary pyproject.toml file.
    fn create_pyproject_toml_file(dir: &tempfile::TempDir, content: &str) -> PathBuf {
        let file_path = dir.path().join("pyproject.toml");
        let mut file = File::create(&file_path).expect("Failed to create file.");
        writeln!(file, "{}", content).expect("Failed to write to file.");
        file_path
    }

    /// Tests the basic functionality of DependencyBuilder and Dependency structs.
    #[test]
    fn dependency_builder_creates_dependency() {
        let dep = DependencyBuilder::new("my_dep".to_string())
            .version("1.0.0".to_string())
            .category("dev".to_string())
            .build();

        assert_eq!(dep.id, "my_dep");
        assert_eq!(dep.version, Some("1.0.0".to_string()));
        assert_eq!(dep.category, Some("dev".to_string()));
    }

    /// Tests the parsing of simple dependencies from a pyproject.toml file.
    #[test]
    fn parse_simple_dependencies() {
        let temp_dir = tempdir().unwrap();
        let toml_path = create_pyproject_toml_file(
            &temp_dir,
            "
            [tool.poetry.dependencies]
            python = \"^3.8\"
            package_a = \"^1.0\"
            package_b = \"^2.0\"
                        ",
        );

        let dependencies =
            get_dependencies(toml_path.as_path()).expect("Failed to get dependencies");

        assert!(dependencies.contains(&Dependency {
            id: "package_a".to_string(),
            version: Some("^1.0".to_string()),
            category: Some("tool.poetry.dependencies".to_string()),
        }));
        assert!(dependencies.contains(&Dependency {
            id: "package_b".to_string(),
            version: Some("^2.0".to_string()),
            category: Some("tool.poetry.dependencies".to_string()),
        }));
        // Including the Python version as a dependency for completeness.
        assert!(dependencies.contains(&Dependency {
            id: "python".to_string(),
            version: Some("^3.8".to_string()),
            category: Some("tool.poetry.dependencies".to_string()),
        }));
        assert_eq!(dependencies.len(), 3);

        // Test different categories such as dev-dependencies, build-dependencies, etc.
        let toml_path = create_pyproject_toml_file(
            &temp_dir,
            "
            [tool.poetry.dev-dependencies]
            package_c = \"^3.0\"
            package_d = \"^4.0\"
                        ",
        );

        let dependencies =
            get_dependencies(toml_path.as_path()).expect("Failed to get dependencies");

        assert!(dependencies.contains(&Dependency {
            id: "package_c".to_string(),
            version: Some("^3.0".to_string()),
            category: Some("tool.poetry.dev-dependencies".to_string()),
        }));
        assert!(dependencies.contains(&Dependency {
            id: "package_d".to_string(),
            version: Some("^4.0".to_string()),
            category: Some("tool.poetry.dev-dependencies".to_string()),
        }));
    }

    /// Tests invalid TOML content.
    #[test]
    fn test_invalid_toml() {
        let temp_dir = tempdir().unwrap();
        let toml_path = create_pyproject_toml_file(&temp_dir, "invalid toml content");

        let result = get_dependencies(toml_path.as_path());
        assert!(result.is_err());
    }
}
