#![feature(test)]

extern crate test;

mod defs;

pub mod exit_codes;
use std::io::Write;

use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use defs::{Dependency, InstalledPackages, Outcome, SitePackages};
use dialoguer::Confirm;

use exit_codes::ExitCode;
use glob::glob;

// text_size
use rustpython_ast::Visitor;
use rustpython_parser::ast;
use rustpython_parser::{parse, Mode};
use std::collections::HashSet;
use std::env;
use std::fs;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

use toml::Value;

use walkdir::WalkDir;

const DEFAULT_PKGS: [&str; 5] = ["pip", "setuptools", "wheel", "python", "python_version"];
const DEP_SPEC_FILES: [&str; 2] = ["requirements.txt", "pyproject.toml"];

#[inline]
fn print_error(msg: impl Into<String>) {
    eprintln!("[pip-udeps error]: {}", msg.into());
}

#[inline]
fn stem_import(import: &str) -> String {
    import.split('.').next().unwrap_or_default().into()
}

struct DependencyCollector {
    deps: HashSet<String>,
}

impl Visitor for DependencyCollector {
    fn visit_stmt(&mut self, node: ast::Stmt<ast::text_size::TextRange>) {
        self.generic_visit_stmt(node);
    }

    fn visit_stmt_import(&mut self, node: ast::StmtImport) {
        node.names.iter().for_each(|alias| {
            self.deps.insert(stem_import(&alias.name));
        })
    }

    fn visit_stmt_import_from(&mut self, node: ast::StmtImportFrom) {
        if let Some(module) = &node.module {
            self.deps.insert(stem_import(module));
        }
    }
}

pub fn get_used_imports(dir: &Path) -> Result<HashSet<String>> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| entry.file_name().to_string_lossy().ends_with(".py"))
        .try_fold(HashSet::new(), |mut acc, entry| {
            let file_content = fs::read_to_string(entry.path())?;
            let module = parse(&file_content, Mode::Module, "<embedded>")?;
            let nodes = module.module().unwrap().body; // Maybe should do a match here??
                                                       // let mut collected_deps: HashSet<String> = HashSet::new();

            let mut collector = DependencyCollector {
                deps: HashSet::new(),
            };

            nodes
                .into_iter()
                .for_each(|node| collector.visit_stmt(node));

            acc.extend(collector.deps);

            Ok(acc)
        })
}

fn visit_toml(value: &Value, deps: &mut HashSet<Dependency>, path: &str) {
    if let Value::Table(table) = value {
        table.iter().for_each(|(key, val)| match val {
            Value::Table(dep_table) if key.ends_with("dependencies") => {
                deps.extend(dep_table.iter().filter_map(|(name, val)| match val {
                    Value::String(v) => Some(Dependency {
                        name: name.to_string(),
                        type_: Some(format!("{}.{}", path.trim_start_matches('.'), key)),
                        version: Some(v.to_string()),
                    }),
                    Value::Table(_) => Some(Dependency {
                        name: name.to_string(),
                        type_: Some(format!("{}.{}", path.trim_start_matches('.'), key)),
                        version: None,
                    }),
                    _ => None,
                }));
            }
            _ => visit_toml(val, deps, &format!("{}.{}", path, key)),
        });
    }
}

fn get_dependencies_from_toml(path: &Path) -> Result<HashSet<Dependency>> {
    let toml_str = fs::read_to_string(path)
        .with_context(|| format!("Failed to read TOML file at {:?}", path))?;
    let toml = toml::from_str(&toml_str).with_context(|| "Failed to parse TOML content")?;

    let mut collected_deps: HashSet<Dependency> = HashSet::new();

    visit_toml(&toml, &mut collected_deps, "");

    Ok(collected_deps)
}

pub fn get_dependency_specification_file(base_dir: &Path) -> Result<PathBuf> {
    let file = base_dir.ancestors().find_map(|dir| {
        DEP_SPEC_FILES
            .into_iter()
            .map(|file_name| dir.join(file_name))
            .find(|file_path| file_path.exists())
    });

    file.ok_or_else(|| {
        anyhow!(format!(
            "Could not find `Requirements.txt` or `pyproject.toml` in '{}' or any parent directory",
            env::current_dir().unwrap().to_string_lossy()
        ))
    })
}

pub fn get_site_package_dir() -> Result<SitePackages> {
    let output = match Command::new("python").arg("-m").arg("site").output() {
        Ok(o) => o,
        Err(_) => {
            print_error(format!(
                "Failed to execute `python -m site`. Are you sure Python is installed?"
            ));
            ExitCode::GeneralError.exit();
        }
    };

    let output_str = str::from_utf8(&output.stdout)
        .context("Output was not valid UTF-8.")?
        .trim();

    let pkg_paths: Vec<String> = output_str
        .lines()
        .filter(|line| {
            line.contains("site-packages") && !line.trim_start().starts_with("USER_SITE:")
        })
        .map(|s| s.trim_matches(|c: char| c.is_whitespace() || c == '\'' || c == ','))
        .map(ToString::to_string)
        .collect();

    let venv = env::var("VIRTUAL_ENV")
        .ok()
        .and_then(|path| path.split('/').last().map(String::from));

    // For testing purposes, we don't want to prompt the user.
    if env::var("RUNNING_TESTS").is_ok() {
        return Ok(SitePackages {
            paths: pkg_paths,
            venv,
        });
    }
    let message = match &venv {
        Some(name) => format!("Detected virtual environment: `{}`. Is this correct?", name),
        None => format!(
            "WARNING: No virtual environment detected. Results may be inaccurate. Continue?"
        )
        .red()
        .to_string(),
    };

    let user_input = Confirm::new()
        .with_prompt(message)
        .interact()
        .context("Failed to get user input.")?;

    if !user_input {
        ExitCode::GeneralError.exit();
    }

    Ok(SitePackages {
        paths: pkg_paths,
        venv,
    })
}

pub fn get_installed_packages(site_pkgs: SitePackages) -> Result<InstalledPackages> {
    let mut pkgs = InstalledPackages::new();

    for path in site_pkgs.paths {
        let glob_pattern = format!("{}/{}-info", path, "*");
        for entry in glob(&glob_pattern).context("Failed to read glob pattern")? {
            let info_dir = entry.context("Invalid glob entry")?;
            let pkg_name = info_dir
                .file_stem()
                .and_then(|stem| stem.to_str())
                .and_then(|s| s.split('-').next())
                .ok_or_else(|| anyhow::anyhow!("Invalid package name format"))
                .map(|s| s.to_lowercase())?;

            let top_level_path = info_dir.join("top_level.txt");

            let import_names = if top_level_path.exists() {
                fs::read_to_string(&top_level_path)?
                    .lines()
                    .map(str::trim)
                    .map(ToString::to_string)
                    .collect()
            } else {
                let mut set = HashSet::new();
                set.insert(pkg_name.clone());
                set
            };

            pkgs.add_pkg(pkg_name, import_names);
        }
    }
    Ok(pkgs)
}

pub fn get_unused_dependencies<W: Write>(base_dir: &Path, stdout: W) -> Result<ExitCode> {
    // potential issues hypercorn that's used in a bash/bat script isn't picked up
    // another example would be flower
    let mut outcome = Outcome::default();

    let deps_file = get_dependency_specification_file(&base_dir)?;
    let pyproject_deps = get_dependencies_from_toml(&deps_file);

    let site_pkgs = get_site_package_dir()?;

    let installed_pkgs = get_installed_packages(site_pkgs)?;

    let used_imports = get_used_imports(base_dir)?;

    let used_pkgs: HashSet<_> = installed_pkgs
        .mapping
        .iter()
        .filter(|(_pkg_name, import_names)| !import_names.is_disjoint(&used_imports))
        .map(|(pkg_name, _)| pkg_name)
        .collect();

    outcome.unused_deps = pyproject_deps?
        .into_iter()
        .filter(|dep| !used_pkgs.contains(&dep.name) && !DEFAULT_PKGS.contains(&dep.name.as_str()))
        .collect();

    outcome.success = outcome.unused_deps.is_empty();

    if !outcome.success {
        let mut note = "".to_owned();
        note += "Note: There might be false-positives.\n";
        note += "      For example, `pip-udeps` cannot detect usage of packages that not imported under `[tool.poetry.*]`.\n";
        outcome.note = Some(note);
    }

    outcome.print_human(stdout)?;
    Ok(if outcome.success {
        ExitCode::Success
    } else {
        ExitCode::GeneralError
    })
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::fs::File;
    use std::io::{self};
    use tempfile::TempDir;
    use test::Bencher;

    #[bench]
    fn bench_get_used_dependencies(b: &mut Bencher) {
        let temp_dir = create_working_directory(
            &["dir1", "dir2"],
            Some(&["requirements.txt", "pyproject.toml", "file1.py"]),
        )
        .unwrap();
        let base_directory = temp_dir.path().join("dir1");
        let file_path = base_directory.join("file1.py");
        let mut file = File::create(file_path).unwrap();
        file.write_all(b"import os, sys").unwrap();
        b.iter(|| get_used_imports(&base_directory));
    }

    fn create_working_directory(
        directories: &[&'static str],
        files: Option<&[&'static str]>,
    ) -> Result<TempDir, io::Error> {
        let temp_dir = TempDir::new()?;

        directories.iter().for_each(|directory| {
            let dir_path = temp_dir.path().join(directory);
            fs::create_dir_all(dir_path).unwrap();
        });

        if let Some(files) = files {
            files.iter().for_each(|file| {
                let file_path = temp_dir.path().join(file);
                File::create(file_path).unwrap();
            });
        }

        Ok(temp_dir)
    }

    #[test]
    fn test_extract_first_part_of_import() {
        let import = "os.path";
        let first_part = stem_import(import);
        assert_eq!(first_part.as_str(), "os");

        let import = "os";
        let first_part = stem_import(import);
        assert_eq!(first_part.as_str(), "os");

        let import = "";
        let first_part = stem_import(import);
        assert_eq!(first_part.as_str(), "");
    }
    // #[test]
    // fn parse_ast_working() {
    //     let file_content = "import os";
    //     let ast = parse_ast(file_content);
    //     assert!(ast.is_ok());

    //     let file_content = "import os, sys";
    //     let ast = parse_ast(file_content);
    //     assert!(ast.is_ok());

    //     let file_content = "import os";
    //     let ast = parse_ast(file_content).unwrap();

    //     assert_eq!(ast.clone().module().unwrap().body.len(), 1);

    //     let body = &ast.module().unwrap().body;
    //     let mut temp_deps_set: HashSet<String> = HashSet::new();
    //     collect_imports(body, &mut temp_deps_set);

    //     assert_eq!(temp_deps_set.len(), 1);
    //     assert!(temp_deps_set.contains("os"));
    // }

    // #[test]
    // fn parse_ast_failing() {
    //     let file_content = "import os,";
    //     let ast = parse_ast(file_content);
    //     assert!(ast.is_err());
    // }
    // #[test]
    // fn collect_imports_success() {
    //     let file_content = "import os";
    //     let ast = parse_ast(file_content).unwrap();
    //     let body = &ast.module().unwrap().body;
    //     let mut temp_deps_set: HashSet<String> = HashSet::new();
    //     collect_imports(body, &mut temp_deps_set);
    //     assert_eq!(temp_deps_set.len(), 1);
    //     assert!(temp_deps_set.contains("os"));

    //     let file_content = "import os, sys";
    //     let ast = parse_ast(file_content).unwrap();
    //     let body = &ast.module().unwrap().body;
    //     let mut temp_deps_set: HashSet<String> = HashSet::new();
    //     collect_imports(body, &mut temp_deps_set);
    //     assert_eq!(temp_deps_set.len(), 2);
    //     assert!(temp_deps_set.contains("os"));
    //     assert!(temp_deps_set.contains("sys"));

    //     let file_content = "from os import path";
    //     let ast: ast::Mod = parse_ast(file_content).unwrap();
    //     let body = &ast.module().unwrap().body;
    //     let mut temp_deps_set: HashSet<String> = HashSet::new();
    //     collect_imports(body, &mut temp_deps_set);
    //     assert_eq!(temp_deps_set.len(), 1);
    //     assert!(temp_deps_set.contains("os"));
    // }

    // #[test]
    // fn collect_imports_failure() {
    //     let file_content = "import os,";
    //     let ast = parse_ast(file_content);
    //     assert!(ast.is_err());

    //     let file_content = "from os import path, sys";
    //     let ast = parse_ast(file_content).unwrap();
    //     let body = &ast.module().unwrap().body;
    //     let mut temp_deps_set: HashSet<String> = HashSet::new();
    //     collect_imports(body, &mut temp_deps_set);
    //     assert_eq!(temp_deps_set.len(), 1);
    //     assert!(temp_deps_set.contains("os"));
    // }

    #[test]
    fn get_dependency_specification_file_that_exists() {
        let temp_dir =
            create_working_directory(&["dir1", "dir2"], Some(&["pyproject.toml"])).unwrap();
        let base_directory = temp_dir.path().join("dir1");
        let file = get_dependency_specification_file(&base_directory).unwrap();
        assert_eq!(file.file_name().unwrap(), "pyproject.toml");
    }

    #[test]
    fn get_dependency_specification_file_that_does_not_exist() {
        let temp_dir = create_working_directory(&["dir1", "dir2"], None).unwrap();
        let base_directory = temp_dir.path().join("dir1");
        let file = get_dependency_specification_file(&base_directory);
        assert!(file.is_err());
    }

    #[test]
    fn test_get_used_dependencies() {
        let temp_dir = create_working_directory(
            &["dir1", "dir2"],
            Some(&["requirements.txt", "pyproject.toml"]),
        )
        .unwrap();
        let base_directory = temp_dir.path().join("dir1");
        let used_dependencies = get_used_imports(&base_directory).unwrap();
        assert_eq!(used_dependencies.len(), 0);

        let temp_dir = create_working_directory(
            &["dir1", "dir2"],
            Some(&["requirements.txt", "pyproject.toml", "file1.py"]),
        )
        .unwrap();
        let base_directory = temp_dir.path().join("dir1");
        let used_dependencies = get_used_imports(&base_directory).unwrap();
        assert_eq!(used_dependencies.len(), 0);

        let temp_dir = create_working_directory(
            &["dir1", "dir2"],
            Some(&["requirements.txt", "pyproject.toml", "file1.py"]),
        )
        .unwrap();
        let base_directory = temp_dir.path().join("dir2");
        let used_dependencies = get_used_imports(&base_directory).unwrap();
        assert_eq!(used_dependencies.len(), 0);

        let temp_dir = create_working_directory(
            &["dir1", "dir2"],
            Some(&["requirements.txt", "pyproject.toml", "file1.py"]),
        )
        .unwrap();
        let base_directory = temp_dir.path().join("dir1");
        let file_path = base_directory.join("file1.py");
        let mut file = File::create(file_path).unwrap();
        file.write_all("import os".as_bytes()).unwrap();

        let used_dependencies = get_used_imports(&base_directory).unwrap();
        assert_eq!(used_dependencies.len(), 1);
        assert!(used_dependencies.contains("os"));

        let temp_dir = create_working_directory(
            &["dir1", "dir2"],
            Some(&["requirements.txt", "pyproject.toml", "file1.py"]),
        )
        .unwrap();
        let base_directory = temp_dir.path().join("dir1");
        let file_path = base_directory.join("file1.py");
        let mut file = File::create(file_path).unwrap();
        file.write_all(b"import os, sys").unwrap();
        let used_dependencies = get_used_imports(&base_directory).unwrap();
        assert_eq!(used_dependencies.len(), 2);
        assert!(used_dependencies.contains("os"));
    }

    // Need to write tests for get_packages_from_pyproject_toml here
    // #[test]
    // fn get_deps_from_pyproject_toml_success() {
    //     let temp_dir =
    //         create_working_directory(&["dir1", "dir2"], Some(&["pyproject.toml"])).unwrap();
    //     let base_directory = temp_dir.path().join("dir1");
    //     let file_path = base_directory.join("pyproject.toml");
    //     let mut file = File::create(&file_path).unwrap();
    //     file.write_all(
    //         r#"
    //         [tool.poetry.dependencies]
    //         requests = "2.25.1"
    //         python = "^3.8"
    //         "#
    //         .as_bytes(),
    //     )
    //     .unwrap();

    //     let packages = get_dependencies_from_pyproject_toml(&file_path).unwrap();
    //     assert_eq!(packages.len(), 2);
    //     assert!(packages.contains(&PyProjectDeps {
    //         name: "requests".to_string()
    //     }));
    //     assert!(packages.contains(&PyProjectDeps {
    //         name: "python".to_string()
    //     }));
    // }
    #[test]
    fn get_site_package_dir_success() {
        std::env::set_var("RUNNING_TESTS", "1");

        let site_packages = get_site_package_dir().unwrap();
        assert!(!site_packages.paths[0].is_empty());

        let is_venv = env::var("VIRTUAL_ENV").is_ok();

        if is_venv {
            let venv_name = env::var("VIRTUAL_ENV")
                .ok()
                .and_then(|path| path.split('/').last().map(String::from));
            assert_eq!(site_packages.venv, venv_name);
        }
    }

    #[test]

    fn check_that_get_installed_pkgs_works() {
        // Create a temporary environment resembling site-packages
        let temp_dir = tempfile::TempDir::new().unwrap();
        let site_packages_dir = temp_dir.path().join("site-packages");
        fs::create_dir(&site_packages_dir).unwrap();

        // Simulate a couple of installed packages with top_level.txt files
        let pkg1_dir = site_packages_dir.join("example_pkg1-0.1.0-info");
        fs::create_dir_all(&pkg1_dir).unwrap();
        fs::write(pkg1_dir.join("top_level.txt"), "example_pkg1\n").unwrap();

        let pkg2_dir = site_packages_dir.join("example_pkg2-0.2.0-info");
        fs::create_dir_all(&pkg2_dir).unwrap();
        fs::write(pkg2_dir.join("top_level.txt"), "example_pkg2\n").unwrap();

        // lets do another package like scikit_learn where we know the name will get remapped to sklearn
        let pkg3_dir = site_packages_dir.join("scikit_learn-0.24.1-info");
        fs::create_dir_all(&pkg3_dir).unwrap();
        fs::write(pkg3_dir.join("top_level.txt"), "sklearn\n").unwrap();

        let site_pkgs = SitePackages {
            paths: vec![
                site_packages_dir.to_string_lossy().to_string(),
                "/usr/lib/python3.8/site-packages".to_string(),
            ],
            venv: Some("test-venv".to_string()),
        };

        let installed_pkgs = get_installed_packages(site_pkgs).unwrap();

        assert_eq!(
            installed_pkgs.mapping.len(),
            3,
            "Should have found two installed packages"
        );

        // Assert that the package names and import names are correct
        assert!(
            installed_pkgs.mapping.get("example-pkg1").is_some(),
            "Should contain example_pkg1"
        );

        assert!(
            installed_pkgs
                .mapping
                .get("example-pkg1")
                .unwrap()
                .contains("example_pkg1"),
            "example-pkg1 should contain example_pkg1"
        );
        assert!(
            installed_pkgs.mapping.get("example-pkg2").is_some(),
            "Should contain example_pkg2"
        );

        assert!(
            installed_pkgs
                .mapping
                .get("example-pkg2")
                .unwrap()
                .contains("example_pkg2"),
            "example-pkg2 should contain example_pkg2"
        );

        assert!(
            installed_pkgs.mapping.get("scikit-learn").is_some(),
            "Should contain scikit_learn"
        );

        assert!(
            installed_pkgs
                .mapping
                .get("scikit-learn")
                .unwrap()
                .contains("sklearn"),
            "scikit_learn should contain sklearn"
        );
        // non-existent package
        assert!(
            installed_pkgs.mapping.get("non-existent").is_some(),
            "Should not contain non-existent"
        );
    }
}
