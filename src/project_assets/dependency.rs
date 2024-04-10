extern crate bytesize;
extern crate fs_extra;

use std::collections::HashSet;
use std::fs;
use std::hash::Hash;
use std::path::Path;
use std::str;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::cli::DepType;
use crate::config::Config;

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Eq, Hash)]
pub struct Dependency {
    id: String,
    version: Option<String>,
}

impl Dependency {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn version(&self) -> &str {
        self.version.as_deref().unwrap_or("N/A")
    }
}

pub struct DependencyBuilder {
    id: String,
    version: Option<String>,
}

impl DependencyBuilder {
    pub fn new(id: String) -> Self {
        Self { id, version: None }
    }

    pub fn version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

    pub fn build(self) -> Dependency {
        Dependency {
            id: self.id,
            version: self.version,
        }
    }
}

#[derive(Default)]
struct DependencyCollector {
    dependencies: HashSet<Dependency>,
}

impl DependencyCollector {
    fn visit_table(&mut self, key: &str, table: &toml::value::Table) {
        // If the key contains "dependencies", then we are looking at a dependency table.
        if key.contains("dependencies") {
            for (dep_name, dep_value) in table {
                self.visit_value(dep_name, dep_value);
            }
        } else {
            for (k, v) in table {
                if let toml::Value::Table(t) = v {
                    self.visit_table(k, t);
                }
            }
        }
    }

    fn visit_value(&mut self, key: &str, value: &toml::Value) {
        match value {
            // For simple string values, assume it's the version directly
            toml::Value::String(version) => {
                self.dependencies.insert(
                    DependencyBuilder::new(key.to_string())
                        .version(version.clone())
                        .build(),
                );
            }
            // For complex structures, look for a "version" key
            toml::Value::Table(table) => {
                if let Some(toml::Value::String(version)) = table.get("version") {
                    self.dependencies.insert(
                        DependencyBuilder::new(key.to_string())
                            .version(version.clone())
                            .build(),
                    );
                }
            }
            // Ignore other types for now...
            _ => (),
        }
    }
}

fn get_pip_dependencies(dep_spec_file: &Path) -> Result<HashSet<Dependency>> {
    let file_content = fs::read_to_string(dep_spec_file)
        .with_context(|| format!("Failed to read file at {:?}", dep_spec_file))?;

    let mut dependencies = HashSet::new();
    for line in file_content.lines() {
        let parts: Vec<&str> = line.split("==").collect();
        if parts.len() == 2 {
            dependencies.insert(
                DependencyBuilder::new(parts[0].to_string())
                    .version(parts[1].to_string())
                    .build(),
            );
        }
    }
    println!("Found {:#?} dependencies", dependencies);

    Ok(dependencies)
}

fn get_poetry_dependencies(dep_spec_file: &Path) -> Result<HashSet<Dependency>> {
    let toml_str = fs::read_to_string(dep_spec_file)
        .with_context(|| format!("Failed to read TOML file at {:?}", dep_spec_file))?;

    let toml_value: toml::Value =
        toml::from_str(&toml_str).with_context(|| "Failed to parse TOML content")?;

    let mut collector = DependencyCollector::default();

    if let toml::Value::Table(table) = toml_value {
        collector.visit_table("", &table);
    }

    Ok(collector.dependencies)
}

/// This function reads a TOML file at the specified path and returns a HashSet of Dependency structs.
pub fn get_dependencies(config: &Config) -> Result<HashSet<Dependency>> {
    let dependencies = match config.dep_type {
        DepType::Pip => get_pip_dependencies(&config.dep_spec_file),
        DepType::Poetry => get_poetry_dependencies(&config.dep_spec_file),
    };

    dependencies
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
            .build();

        assert_eq!(dep.id, "my_dep");
        assert_eq!(dep.version, Some("1.0.0".to_string()));
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
            get_poetry_dependencies(toml_path.as_path()).expect("Failed to get dependencies");

        assert!(dependencies.contains(&Dependency {
            id: "package_a".to_string(),
            version: Some("^1.0".to_string()),
        }));
        assert!(dependencies.contains(&Dependency {
            id: "package_b".to_string(),
            version: Some("^2.0".to_string()),
        }));
        // Including the Python version as a dependency for completeness.
        assert!(dependencies.contains(&Dependency {
            id: "python".to_string(),
            version: Some("^3.8".to_string()),
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
            get_poetry_dependencies(toml_path.as_path()).expect("Failed to get dependencies");

        assert!(dependencies.contains(&Dependency {
            id: "package_c".to_string(),
            version: Some("^3.0".to_string()),
        }));
        assert!(dependencies.contains(&Dependency {
            id: "package_d".to_string(),
            version: Some("^4.0".to_string()),
        }));

        // Test categories that are not dependencies.
        let toml_path = create_pyproject_toml_file(
            &temp_dir,
            "
            [tool.poetry]
            name = \"my_project\"
            version = \"0.1.0\"
            description = \"My project\"
                        ",
        );

        let dependencies =
            get_poetry_dependencies(toml_path.as_path()).expect("Failed to get dependencies");

        assert!(dependencies.is_empty());
    }

    #[test]
    fn test_complex_dependencies() {
        let temp_dir = tempdir().unwrap();

        // Test a more complex TOML dependency file.
        // Ex: fastapi = { version = "^0.109.2", optional = true }
        let toml_path = create_pyproject_toml_file(
            &temp_dir,
            "
            [tool.poetry.dependencies]
            fastapi = { version = \"^0.109.2\", optional = true }
            mkdocs-material = {extras = [\"imaging\"], version = \"^9.5.9\"}
            uvicorn = \"^0.13.4\"
                        ",
        );

        let dependencies =
            get_poetry_dependencies(toml_path.as_path()).expect("Failed to get dependencies");

        assert!(dependencies.contains(&Dependency {
            id: "fastapi".to_string(),
            version: Some("^0.109.2".to_string()),
        }));

        assert!(dependencies.contains(&Dependency {
            id: "mkdocs-material".to_string(),
            version: Some("^9.5.9".to_string()),
        }));

        assert!(dependencies.contains(&Dependency {
            id: "uvicorn".to_string(),
            version: Some("^0.13.4".to_string()),
        }));
    }

    /// Tests invalid TOML content.
    #[test]
    fn test_invalid_toml() {
        let temp_dir = tempdir().unwrap();
        let toml_path = create_pyproject_toml_file(&temp_dir, "invalid toml content");

        let result = get_poetry_dependencies(toml_path.as_path());
        assert!(result.is_err());
    }
}
