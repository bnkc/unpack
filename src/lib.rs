#![feature(test)]
extern crate test;

pub mod cli;
pub mod defs;
pub mod exit_codes;

extern crate fs_extra;

use fs_extra::dir::get_size;

use std::collections::{BTreeSet, HashSet};
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::str;

use crate::cli::*;
use crate::defs::{Dependency, Outcome, PackageMetadata, Packages, SitePackages};
use crate::exit_codes::*;

use anyhow::{bail, Context, Result};
use colored::Colorize;
use dialoguer::Confirm;
use glob::glob;
use rustpython_ast::Visitor;
use rustpython_parser::{ast, parse, Mode};
use toml::Value;
use walkdir::WalkDir;

/// Print an error message to stderr
#[inline]
fn print_error(msg: impl Into<String>) {
    eprintln!("[pip-udeps error]: {}", msg.into());
}

/// Extract the first part of an import statement
///  e.g. `os.path` -> `os`
#[inline]
fn stem_import(import: &str) -> String {
    import.split('.').next().unwrap_or_default().into()
}

/// Collects all the dependencies from the AST
struct DependencyCollector {
    deps: HashSet<String>,
}

/// This is a visitor pattern that implements the Visitor trait
impl Visitor for DependencyCollector {
    /// This is a generic visit method that will be called for all nodes
    fn visit_stmt(&mut self, node: ast::Stmt<ast::text_size::TextRange>) {
        self.generic_visit_stmt(node);
    }
    /// This method is `overridden` to collect the dependencies into `self.deps`
    fn visit_stmt_import(&mut self, node: ast::StmtImport) {
        node.names.iter().for_each(|alias| {
            self.deps.insert(stem_import(&alias.name));
        })
    }

    /// This method is `overridden` to collect the dependencies into `self.deps`
    fn visit_stmt_import_from(&mut self, node: ast::StmtImportFrom) {
        if let Some(module) = &node.module {
            self.deps.insert(stem_import(module));
        }
    }
}

pub fn get_imports(config: &Config) -> Result<BTreeSet<String>> {
    WalkDir::new(&config.base_directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| {
            let file_name = entry.file_name().to_string_lossy();

            // Ignore hidden files and directories if `ignore_hidden` is set to true
            file_name.ends_with(".py") && !(config.ignore_hidden && file_name.starts_with("."))
        })
        .try_fold(BTreeSet::new(), |mut acc, entry| {
            let file_content = fs::read_to_string(entry.path())?;
            let module = parse(&file_content, Mode::Module, "<embedded>")?;

            let mut collector = DependencyCollector {
                deps: HashSet::new(),
            };

            module
                .module()
                .unwrap() //Probably should change this from unwrap to something else
                .body
                .into_iter()
                .for_each(|node| collector.visit_stmt(node));

            acc.extend(collector.deps);

            Ok(acc)
        })
}

// This will 100% need revisited and extended out to requirements.txt
fn visit_toml(value: &Value, deps: &mut HashSet<Dependency>, path: &str) {
    if let Value::Table(table) = value {
        table.iter().for_each(|(key, val)| match val {
            Value::Table(dep_table) if key.ends_with("dependencies") => {
                deps.extend(dep_table.iter().filter_map(|(name, val)| match val {
                    Value::String(v) => Some(Dependency {
                        id: name.to_string(),
                        category: Some(format!("{}.{}", path.trim_start_matches('.'), key)),
                        version: Some(v.to_string()),
                    }),
                    Value::Table(_) => Some(Dependency {
                        id: name.to_string(),
                        category: Some(format!("{}.{}", path.trim_start_matches('.'), key)),
                        version: None,
                    }),
                    _ => None,
                }));
            }
            _ => visit_toml(val, deps, &format!("{}.{}", path, key)),
        });
    }
}

/// Given a path to a `pyproject.toml` file, we will collect all the dependencies
/// from `[tool.poetry.*]`
// This will 100% need revisited and extended out to requirements.txt
fn get_dependencies_from_toml(path: &Path) -> Result<HashSet<Dependency>> {
    let toml_str = fs::read_to_string(path)
        .with_context(|| format!("Failed to read TOML file at {:?}", path))?;
    let toml = toml::from_str(&toml_str).with_context(|| "Failed to parse TOML content")?;

    let mut deps = HashSet::new();

    visit_toml(&toml, &mut deps, "");

    Ok(deps)
}

/// Given a virtual environment, we will prompt the user to confirm if it is the correct one
fn get_prompt(venv: &Option<String>) -> String {
    match venv {
        Some(name) => format!("Detected virtual environment: `{}`. Is this correct?", name),
        None => format!(
            "WARNING: No virtual environment detected. Results may be inaccurate. Continue?"
        )
        .red()
        .to_string(),
    }
}

/// Collect the site-packages directory path(s) and the virtual environment name
pub fn get_site_package_dir(config: &Config) -> Result<SitePackages> {
    let output = match Command::new("python").arg("-m").arg("site").output() {
        Ok(o) => o,
        Err(_) => {
            print_error("Failed to execute `python -m site`. Are you sure Python is installed?");
            ExitCode::GeneralError.exit();
        }
    };

    let output_str = str::from_utf8(&output.stdout)
        .context("Output was not valid UTF-8.")?
        .trim();

    let pkg_paths: Vec<PathBuf> = output_str
        .lines()
        .filter(|line| {
            line.contains("site-packages") && !line.trim_start().starts_with("USER_SITE:")
        })
        .map(|s| s.trim_matches(|c: char| c.is_whitespace() || c == '\'' || c == ','))
        .map(PathBuf::from)
        .collect();

    if pkg_paths.is_empty() {
        bail!("No site-packages found. Are you sure you are in a virtual environment?");
    }

    let venv = env::var("VIRTUAL_ENV")
        .ok()
        .and_then(|path| path.split('/').last().map(String::from));

    if config.env != Env::Test {
        let prompt = get_prompt(&venv);

        let user_input = Confirm::new()
            .with_prompt(prompt)
            .interact()
            .context("Failed to get user input.")?;

        if !user_input {
            print_error("Exiting. Please activate the correct virtual environment and try again.");
            ExitCode::GeneralError.exit();
        }
    }

    Ok(SitePackages {
        paths: pkg_paths,
        venv,
    })
}

/// Collects all installed packages on the system from given site package directories,
/// ignoring packages without any aliases and aggregates sizes of all aliases.
pub fn get_installed_packages(site_pkgs: SitePackages) -> Result<Packages> {
    let mut packages = Packages::default();

    for site_path in &site_pkgs.paths {
        let glob_pattern = format!("{}/{}-info", site_path.display(), "*");
        let dist_info_dirs = glob(&glob_pattern)
            .with_context(|| format!("Failed to read glob pattern {}", glob_pattern))?;

        for entry in dist_info_dirs {
            let dist_info_path = entry.with_context(|| "Invalid glob entry")?;

            let package_name = dist_info_path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .and_then(|s| s.split('-').next())
                .map(|s| s.to_lowercase())
                .ok_or_else(|| anyhow::anyhow!("Invalid package name format"))?;

            let top_level_path = dist_info_path.join("top_level.txt");
            let mut total_size = 0u64;
            let mut import_names = BTreeSet::new();

            if top_level_path.exists() {
                let lines = fs::read_to_string(&top_level_path)
                    .with_context(|| format!("Failed to read {}", top_level_path.display()))?;
                for line in lines.lines().map(str::trim) {
                    let potential_path = site_path.join(line);
                    if potential_path.exists() {
                        import_names.insert(line.to_string());
                        total_size += get_size(&potential_path).with_context(|| {
                            format!("Failed to get size for {}", potential_path.display())
                        })?;
                    }
                }
            } else {
                let path = site_path.join(&package_name);
                if path.exists() {
                    import_names.insert(package_name.clone());
                    total_size = get_size(&path)
                        .with_context(|| format!("Failed to get size for {}", path.display()))?;
                }
            }

            // If there are no valid aliases, skip this package
            if import_names.is_empty() {
                continue;
            }

            // let human_readable_size = ByteSize::b(total_size).to_string_as(true);

            packages.add_pkg(PackageMetadata {
                id: package_name,
                aliases: import_names,
                size: total_size,
            });
        }
    }

    Ok(packages)
}

// Can't read bash or bat scripts. WIll need to return to this issue
pub fn analyze(config: &Config) -> Result<Outcome> {
    let mut outcome = Outcome::default();

    let site_pkgs = get_site_package_dir(&config)?;
    let pkgs = get_installed_packages(site_pkgs)?;
    // println!("here is what is in the site packages {:#?}", pkgs);

    let pyproject_deps = get_dependencies_from_toml(&config.dep_spec_file)?;
    // println!("here is what is in the pyproject.toml {:?}", pyproject_deps);

    let imports = get_imports(&config)?;

    outcome.packages = pkgs.get_packages_by_state(&pyproject_deps, &imports, &config.package_state);

    // println!("here is what is relevant {:#?}", relevant_pkgs);

    // let used_pkgs = installed_pkgs.filter_used_pkgs(&imports);

    // THIS IS WRONG. IF THE DEP IS NOT INSTALLED IT'S NOT A "UNUSED DEP"
    // outcome.unused_deps = pyproject_deps
    //     .into_iter()
    //     .filter(|dep| !used_pkgs.contains(&dep.id) && !DEFAULT_PKGS.contains(&dep.id.as_str()))
    //     .collect();

    // outcome.packages = relevant_pkgs;

    // println!("here is what is unused {:?}", outcome.unused_deps);

    // outcome.success = outcome.unused_deps.is_empty();

    if !outcome.success {
        let mut note = "".to_owned();
        note += "Note: There might be false-positives.\n";
        note += "      For example, `pip-udeps` cannot detect usage of packages that are not imported under `[tool.poetry.*]`.\n";
        outcome.note = Some(note);
    }

    Ok(outcome)
}

#[cfg(test)]
mod tests {

    use super::*;

    use defs::{OutputKind, PackageState};
    use std::fs::File;
    use std::io::Write;
    use std::io::{self};
    use tempfile::TempDir;
    use test::Bencher;

    // Used to create a temporary directory with the given directories and files
    fn create_working_directory(
        dirs: &[&'static str],
        files: Option<&[&'static str]>,
    ) -> Result<TempDir, io::Error> {
        let temp_dir = TempDir::new()?;

        dirs.iter().for_each(|directory| {
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

    struct TestEnv {
        /// Temporary project directory
        _temp_dir: TempDir,

        /// Test Configuration struct
        config: Config,
    }

    impl TestEnv {
        fn new(dirs: &[&'static str], files: Option<&[&'static str]>) -> Self {
            let temp_dir = create_working_directory(dirs, files).unwrap();
            let base_directory = temp_dir.path().join(dirs[0]);
            let pyproject_path: PathBuf = base_directory.join("pyproject.toml");
            let mut file = File::create(&pyproject_path).unwrap();

            file.write_all(
                r#"
                            [tool.poetry.dependencies]
                            requests = "2.25.1"
                            python = "^3.8"
                            pandas = "^1.2.0"
                            "#
                .as_bytes(),
            )
            .unwrap();

            let config = Config {
                base_directory,
                dep_spec_file: pyproject_path,
                ignore_hidden: false,
                env: Env::Test,
                output: OutputKind::Human,
                package_state: PackageState::Unused,
            };

            Self {
                _temp_dir: temp_dir,
                config,
            }
        }
    }

    #[bench]
    fn bench_get_used_imports(b: &mut Bencher) {
        let te = TestEnv::new(&["dir1", "dir2"], Some(&["file1.py"]));
        b.iter(|| get_imports(&te.config));
    }

    #[bench]
    fn bench_get_dependencies_from_toml(b: &mut Bencher) {
        let te = TestEnv::new(&["dir1", "dir2"], Some(&["pyproject.toml"]));
        b.iter(|| get_dependencies_from_toml(&te.config.dep_spec_file));
    }

    // #[test]
    // fn basic_usage() {
    //     let te = TestEnv::new(
    //         &["dir1", "dir2"],
    //         Some(&["requirements.txt", "pyproject.toml", "file1.py"]),
    //     );

    //     let unused_deps = get_unused_dependencies(&te.config);
    //     assert!(unused_deps.is_ok());

    //     let outcome = unused_deps.unwrap();
    //     assert_eq!(outcome.success, false); // There should be unused dependencies

    //     // This is because we use python by default
    //     assert_eq!(
    //         outcome.unused_deps.len(),
    //         2,
    //         "There should be 2 unused dependencies"
    //     );
    //     // assert_eq!(outcome.unused_deps.iter().next().unwrap().name, "pandas");
    //     assert!(outcome
    //         .unused_deps
    //         .iter()
    //         .any(|dep| dep.id == "pandas" || dep.id == "requests"));

    //     // Now let's import requests in file1.py
    //     let file_path = te.config.base_directory.join("file1.py");
    //     let mut file = File::create(file_path).unwrap();
    //     file.write_all("import requests".as_bytes()).unwrap();

    //     let unused_deps = get_unused_dependencies(&te.config);
    //     assert!(unused_deps.is_ok());

    //     // check that there are no unused dependencies
    //     let outcome = unused_deps.unwrap();
    //     assert_eq!(outcome.success, false);
    //     assert_eq!(outcome.unused_deps.len(), 1);

    //     // Now let's import requests in file1.py
    //     let file_path = te.config.base_directory.join("file1.py");
    //     let mut file = File::create(file_path).unwrap();
    //     file.write_all("import requests\nimport pandas as pd".as_bytes())
    //         .unwrap();

    //     let unused_deps = get_unused_dependencies(&te.config);
    //     assert!(unused_deps.is_ok());
    //     assert_eq!(
    //         unused_deps.unwrap().unused_deps.len(),
    //         0,
    //         "There should be no unused dependency"
    //     );
    // }

    #[test]
    fn stem_import_correctly_stems() {
        let first_part = stem_import("os.path");
        assert_eq!(first_part.as_str(), "os");

        let first_part = stem_import("os");
        assert_eq!(first_part.as_str(), "os");

        let first_part = stem_import("");
        assert_eq!(first_part.as_str(), "");
    }

    #[test]
    fn get_used_imports_correctly_collects() {
        let te = TestEnv::new(
            &["dir1", "dir2"],
            Some(&["requirements.txt", "pyproject.toml", "file1.py"]),
        );

        let used_imports = get_imports(&te.config);
        assert!(used_imports.is_ok());

        let used_imports = used_imports.unwrap();
        assert_eq!(used_imports.len(), 0);

        let file_path = te.config.base_directory.join("file1.py");
        let mut file = File::create(file_path).unwrap();
        file.write_all(r#"import pandas as pd"#.as_bytes()).unwrap();

        let used_imports = get_imports(&te.config);
        assert!(used_imports.is_ok());

        let used_imports = used_imports.unwrap();
        assert_eq!(used_imports.len(), 1);
        assert!(used_imports.contains("pandas"));
        assert!(!used_imports.contains("sklearn"));
    }

    #[test]
    fn correct_promt_from_get_prompt() {
        let venv = Some("test-venv".to_string());
        let prompt = get_prompt(&venv);
        assert_eq!(
            prompt,
            "Detected virtual environment: `test-venv`. Is this correct?"
        );

        let venv = None;
        let prompt = get_prompt(&venv);
        assert_eq!(
            prompt,
            "WARNING: No virtual environment detected. Results may be inaccurate. Continue?"
                .red()
                .to_string()
        );
    }

    #[test]
    fn get_site_package_dir_success() {
        let te = TestEnv::new(&["dir1", "dir2"], Some(&["pyproject.toml"]));

        let site_pkgs = get_site_package_dir(&te.config).unwrap();

        assert!(!site_pkgs.paths.is_empty());

        let venv_name = env::var("VIRTUAL_ENV")
            .ok()
            .and_then(|path| path.split('/').last().map(String::from));
        assert_eq!(site_pkgs.venv, venv_name);
    }

    // #[test]

    // fn get_installed_packages_correctly_maps() {
    //     // Create a temporary environment resembling site-packages
    //     let temp_dir = tempfile::TempDir::new().unwrap();
    //     let site_packages_dir = temp_dir.path().join("site-packages");
    //     fs::create_dir(&site_packages_dir).unwrap();

    //     // Simulate a couple of installed packages with top_level.txt files
    //     let pkg1_dir = site_packages_dir.join("example_pkg1-0.1.0-info");
    //     fs::create_dir_all(&pkg1_dir).unwrap();
    //     fs::write(pkg1_dir.join("top_level.txt"), "example_pkg1\n").unwrap();

    //     let pkg2_dir = site_packages_dir.join("example_pkg2-0.2.0-info");
    //     fs::create_dir_all(&pkg2_dir).unwrap();
    //     fs::write(pkg2_dir.join("top_level.txt"), "example_pkg2\n").unwrap();

    //     // lets do another package like scikit_learn where we know the name will get remapped to sklearn
    //     let pkg3_dir = site_packages_dir.join("scikit_learn-0.24.1-info");
    //     fs::create_dir_all(&pkg3_dir).unwrap();
    //     fs::write(pkg3_dir.join("top_level.txt"), "sklearn\n").unwrap();

    //     let site_pkgs = SitePackages {
    //         paths: vec![site_packages_dir],
    //         venv: Some("test-venv".to_string()),
    //     };

    //     let installed_pkgs = get_installed_packages(site_pkgs).unwrap();

    //     assert_eq!(
    //         installed_pkgs._mapping().len(),
    //         3,
    //         "Should have found two installed packages"
    //     );

    //     // Assert that the package names and import names are correct
    //     assert!(
    //         installed_pkgs._mapping().get("example-pkg1").is_some(),
    //         "Should contain example_pkg1"
    //     );

    //     assert!(
    //         installed_pkgs
    //             ._mapping()
    //             .get("example-pkg1")
    //             .unwrap()
    //             .contains("example_pkg1"),
    //         "example-pkg1 should contain example_pkg1"
    //     );
    //     assert!(
    //         installed_pkgs._mapping().get("example-pkg2").is_some(),
    //         "Should contain example_pkg2"
    //     );

    //     assert!(
    //         installed_pkgs
    //             ._mapping()
    //             .get("example-pkg2")
    //             .unwrap()
    //             .contains("example_pkg2"),
    //         "example-pkg2 should contain example_pkg2"
    //     );

    //     assert!(
    //         installed_pkgs._mapping().get("scikit-learn").is_some(),
    //         "Should contain scikit_learn"
    //     );

    //     assert!(
    //         installed_pkgs
    //             ._mapping()
    //             .get("scikit-learn")
    //             .unwrap()
    //             .contains("sklearn"),
    //         "scikit_learn should contain sklearn"
    //     );
    //     // non-existent package
    //     assert!(
    //         !installed_pkgs._mapping().get("non-existent").is_some(),
    //         "Should not contain non-existent"
    //     );
    // }

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
}
