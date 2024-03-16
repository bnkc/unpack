mod defs;
mod error;
mod exit_codes;

use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use defs::{Dependency, InstalledPackages, SitePackages};
use dialoguer::Confirm;
use error::print_error;
use exit_codes::ExitCode;
use glob::glob;
use rustpython_parser::{ast, lexer::lex, parse_tokens, Mode, ParseError};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;
use toml::Table;
use toml::Value;
use walkdir::WalkDir;

const DEFAULT_PKGS: [&str; 5] = ["pip", "setuptools", "wheel", "python", "python_version"];

#[inline]
fn extract_first_part_of_import(import: &str) -> ast::Identifier {
    import.split('.').next().unwrap_or_default().into()
}

#[inline]
fn parse_python_ast(file_content: &str) -> Result<ast::Mod, ParseError> {
    parse_tokens(lex(file_content, Mode::Module), Mode::Module, "<embedded>")
}

// Recursively Collects identifiers from import statements in the specified AST.
///
/// # Arguments
///
/// * `stmts` - A reference to the Vec of ast::Stmt to collect identifiers from.
/// * `deps_set` - A reference to the HashSet to collect the identifiers into.
///
/// # Returns
///
/// A Result containing the parsed ast::Mod on success, or a ParseError on failure.
fn collect_imports(stmts: &[ast::Stmt], deps_set: &mut HashSet<ast::Identifier>) {
    stmts.iter().for_each(|stmt| match stmt {
        ast::Stmt::Import(import) => {
            import.names.iter().for_each(|alias| {
                deps_set.insert(extract_first_part_of_import(&alias.name));
            });
        }
        ast::Stmt::ImportFrom(import) => {
            if let Some(module) = &import.module {
                deps_set.insert(extract_first_part_of_import(module));
            }
        }
        ast::Stmt::FunctionDef(function_def) => collect_imports(&function_def.body, deps_set),
        ast::Stmt::ClassDef(class_def) => collect_imports(&class_def.body, deps_set),
        _ => {}
    });
}

// Attempts to read and parse Python files in the specified directory, collecting identifiers from import statements.
///
/// # Arguments
///
/// * `dir` - A reference to the PathBuf for the directory to search within.
///
/// # Returns
///
/// A Result containing a Vec of ast::Identifier on success, or an std::io::Error on failure.
pub fn get_used_deps(dir: &Path) -> Result<Vec<ast::Identifier>> {
    let walker: walkdir::IntoIter = WalkDir::new(dir).into_iter();
    let mut used_deps = HashSet::new();

    for entry in walker.filter_map(|e| e.ok()) {
        if entry.file_name().to_string_lossy().ends_with(".py") {
            let file_content = match fs::read_to_string(entry.path()) {
                Ok(content) => content,
                Err(_) => continue,
            };

            if let Ok(ast) = parse_python_ast(&file_content) {
                if let Some(module) = ast.module() {
                    collect_imports(&module.body, &mut used_deps);
                }
            }
        }
    }

    Ok(used_deps.into_iter().collect())
}

// Checks for dependency specification files in the specified directory or any parent directories.
///
///
/// # Arguments
///
/// * `base_directory` - A reference to the PathBuf for the directory to search within.
///
/// # Returns
///
/// A boolean indicating whether the dependency specification files were found.
pub fn get_deps_specification_file(base_dir: &Path) -> Result<PathBuf> {
    let file = base_dir.ancestors().find_map(|dir| {
        let files = vec!["requirements.txt", "pyproject.toml"];
        files
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

// Gets the packages from a pyproject.toml file.
///
/// # Arguments
///
/// * `file` - A reference to the Path for the pyproject.toml file to read.
///
/// # Returns
///
/// A Result containing a Vec of Dependencies on success, or an ExitCode on failure.
///
/// # Errors
///
/// * ExitCode::GeneralError - If the file could not be read or parsed.
pub fn get_deps_from_pyproject_toml(path: &PathBuf) -> Result<Vec<Dependency>> {
    let toml_str = fs::read_to_string(path)?;

    let toml: Table = match toml::from_str(&toml_str) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}", e);
            return Ok(vec![]);
        }
    };

    let deps = toml
        .get("tool")
        .and_then(|t| t.get("poetry"))
        .and_then(|p| p.get("dependencies"))
        .and_then(|d| d.as_table())
        .ok_or_else(|| anyhow!("Missing `[tool.poetry.dependencies]` section in TOML"))?;

    Ok(deps
        .iter()
        .map(|(name, _)| Dependency { name: name.clone() })
        .collect())
}

// Gets the site-packages directory + Option<venv> name for the current Python environment.
///
/// # Returns
///     
/// A Result containing a SitePackagesDir on success, or an ExitCode on failure.
///     
/// # Errors
///
/// * ExitCode::GeneralError - If the Python command failed to execute.
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

    let venv_name = env::var("VIRTUAL_ENV")
        .ok()
        .and_then(|path| path.split('/').last().map(String::from));

    // For testing purposes, we don't want to prompt the user.
    if env::var("RUNNING_TESTS").is_ok() {
        return Ok(SitePackages {
            paths: pkg_paths,
            venv_name,
        });
    }
    let message = match &venv_name {
        Some(name) => format!("Virtual environment '{}' detected. Continue?", name),
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
        venv_name,
    })
}

// Gets the installed dependencies from the site-packages directory.
///
/// # Arguments
///
/// * `site_pkgs` - A reference to the SitePackagesDir to search within.
///
/// # Returns
///     
/// A Result containing a HashMap of package names to HashSet of import names on success, or an ExitCode on failure.
///     
/// # Errors
///
/// * ExitCode::GeneralError - If the site-packages directory could not be read or the top_level.txt files could not be read.
pub fn get_installed_pkgs(site_pkgs: SitePackages) -> Result<InstalledPackages> {
    let mut installed_pkgs = InstalledPackages::new();

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

            installed_pkgs.add_pkg(pkg_name, import_names);
        }
    }
    Ok(installed_pkgs)
}

pub fn get_unused_deps(base_dir: &Path) -> Result<ExitCode> {
    let deps_file = get_deps_specification_file(&base_dir)?;
    let pyproject_packages = get_deps_from_pyproject_toml(&deps_file);

    let site_pkgs = get_site_package_dir()?;
    // println!("{:#?}", site_pkgs);
    let installed_pkgs = get_installed_pkgs(site_pkgs)?;
    println!("{:#?}", installed_pkgs);

    // let used_dependencies = get_used_deps(base_dir);

    // this is temporary
    Ok(ExitCode::Success)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::io::{self};
    use tempfile::TempDir;

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
        let first_part = extract_first_part_of_import(import);
        assert_eq!(first_part.as_str(), "os");

        let import = "os";
        let first_part = extract_first_part_of_import(import);
        assert_eq!(first_part.as_str(), "os");

        let import = "";
        let first_part = extract_first_part_of_import(import);
        assert_eq!(first_part.as_str(), "");
    }
    #[test]
    fn parse_ast_working() {
        let file_content = "import os";
        let ast = parse_python_ast(file_content);
        assert!(ast.is_ok());

        let file_content = "import os, sys";
        let ast = parse_python_ast(file_content);
        assert!(ast.is_ok());

        let file_content = "import os";
        let ast = parse_python_ast(file_content).unwrap();

        assert_eq!(ast.clone().module().unwrap().body.len(), 1);

        let body = &ast.module().unwrap().body;
        let mut temp_deps_set: HashSet<ast::Identifier> = HashSet::new();
        collect_imports(body, &mut temp_deps_set);

        assert_eq!(temp_deps_set.len(), 1);
        assert!(temp_deps_set.contains(&ast::Identifier::new("os")));
    }

    #[test]
    fn parse_ast_failing() {
        let file_content = "import os,";
        let ast = parse_python_ast(file_content);
        assert!(ast.is_err());
    }
    #[test]
    fn collect_imports_success() {
        let file_content = "import os";
        let ast = parse_python_ast(file_content).unwrap();
        let body = &ast.module().unwrap().body;
        let mut temp_deps_set: HashSet<ast::Identifier> = HashSet::new();
        collect_imports(body, &mut temp_deps_set);
        assert_eq!(temp_deps_set.len(), 1);
        assert!(temp_deps_set.contains(&ast::Identifier::new("os")));

        let file_content = "import os, sys";
        let ast = parse_python_ast(file_content).unwrap();
        let body = &ast.module().unwrap().body;
        let mut temp_deps_set: HashSet<ast::Identifier> = HashSet::new();
        collect_imports(body, &mut temp_deps_set);
        assert_eq!(temp_deps_set.len(), 2);
        assert!(temp_deps_set.contains(&ast::Identifier::new("os")));
        assert!(temp_deps_set.contains(&ast::Identifier::new("sys")));

        let file_content = "from os import path";
        let ast: ast::Mod = parse_python_ast(file_content).unwrap();
        let body = &ast.module().unwrap().body;
        let mut temp_deps_set: HashSet<ast::Identifier> = HashSet::new();
        collect_imports(body, &mut temp_deps_set);
        assert_eq!(temp_deps_set.len(), 1);
        assert!(temp_deps_set.contains(&ast::Identifier::new("os")));
    }

    #[test]
    fn collect_imports_failure() {
        let file_content = "import os,";
        let ast = parse_python_ast(file_content);
        assert!(ast.is_err());

        let file_content = "from os import path, sys";
        let ast = parse_python_ast(file_content).unwrap();
        let body = &ast.module().unwrap().body;
        let mut temp_deps_set: HashSet<ast::Identifier> = HashSet::new();
        collect_imports(body, &mut temp_deps_set);
        assert_eq!(temp_deps_set.len(), 1);
        assert!(temp_deps_set.contains(&ast::Identifier::new("os")));
    }

    #[test]
    fn get_dependency_specification_file_that_exists() {
        let temp_dir =
            create_working_directory(&["dir1", "dir2"], Some(&["pyproject.toml"])).unwrap();
        let base_directory = temp_dir.path().join("dir1");
        let file = get_deps_specification_file(&base_directory).unwrap();
        assert_eq!(file.file_name().unwrap(), "pyproject.toml");
    }

    #[test]
    fn get_dependency_specification_file_that_does_not_exist() {
        let temp_dir = create_working_directory(&["dir1", "dir2"], None).unwrap();
        let base_directory = temp_dir.path().join("dir1");
        let file = get_deps_specification_file(&base_directory);
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
        let used_dependencies = get_used_deps(&base_directory).unwrap();
        assert_eq!(used_dependencies.len(), 0);

        let temp_dir = create_working_directory(
            &["dir1", "dir2"],
            Some(&["requirements.txt", "pyproject.toml", "file1.py"]),
        )
        .unwrap();
        let base_directory = temp_dir.path().join("dir1");
        let used_dependencies = get_used_deps(&base_directory).unwrap();
        assert_eq!(used_dependencies.len(), 0);

        let temp_dir = create_working_directory(
            &["dir1", "dir2"],
            Some(&["requirements.txt", "pyproject.toml", "file1.py"]),
        )
        .unwrap();
        let base_directory = temp_dir.path().join("dir2");
        let used_dependencies = get_used_deps(&base_directory).unwrap();
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

        let used_dependencies = get_used_deps(&base_directory).unwrap();
        assert_eq!(used_dependencies.len(), 1);
        assert!(used_dependencies.contains(&ast::Identifier::new("os")));

        let temp_dir = create_working_directory(
            &["dir1", "dir2"],
            Some(&["requirements.txt", "pyproject.toml", "file1.py"]),
        )
        .unwrap();
        let base_directory = temp_dir.path().join("dir1");
        let file_path = base_directory.join("file1.py");
        let mut file = File::create(file_path).unwrap();
        file.write_all(b"import os, sys").unwrap();
        let used_dependencies = get_used_deps(&base_directory).unwrap();
        assert_eq!(used_dependencies.len(), 2);
        assert!(used_dependencies.contains(&ast::Identifier::new("os")));
    }

    // Need to write tests for get_packages_from_pyproject_toml here
    #[test]
    fn get_dependencies_from_pyproject_toml_success() {
        let temp_dir =
            create_working_directory(&["dir1", "dir2"], Some(&["pyproject.toml"])).unwrap();
        let base_directory = temp_dir.path().join("dir1");
        let file_path = base_directory.join("pyproject.toml");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(
            r#"
            [tool.poetry.dependencies]
            requests = "2.25.1"
            python = "^3.8"
            "#
            .as_bytes(),
        )
        .unwrap();

        let packages = get_deps_from_pyproject_toml(&file_path).unwrap();
        assert_eq!(packages.len(), 2);
        assert!(packages.contains(&Dependency {
            name: "requests".to_string()
        }));
        assert!(packages.contains(&Dependency {
            name: "python".to_string()
        }));
    }

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
            assert_eq!(site_packages.venv_name, venv_name);
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
            venv_name: Some("test-venv".to_string()),
        };

        let installed_pkgs = get_installed_pkgs(site_pkgs).unwrap();

        assert_eq!(
            installed_pkgs.mapping.len(),
            3,
            "Should have found two installed packages"
        );

        // Assert that the package names and import names are correct
        assert!(
            installed_pkgs.get_pkg("example-pkg1").is_some(),
            "Should contain example_pkg1"
        );

        assert!(
            installed_pkgs
                .get_pkg("example-pkg1")
                .unwrap()
                .contains("example_pkg1"),
            "example-pkg1 should contain example_pkg1"
        );
        assert!(
            installed_pkgs.get_pkg("example-pkg2").is_some(),
            "Should contain example_pkg2"
        );

        assert!(
            installed_pkgs
                .get_pkg("example-pkg2")
                .unwrap()
                .contains("example_pkg2"),
            "example-pkg2 should contain example_pkg2"
        );

        assert!(
            installed_pkgs.get_pkg("scikit-learn").is_some(),
            "Should contain scikit_learn"
        );

        assert!(
            installed_pkgs
                .get_pkg("scikit-learn")
                .unwrap()
                .contains("sklearn"),
            "scikit_learn should contain sklearn"
        );
        // non-existent package
        assert!(
            installed_pkgs.get_pkg("non-existent").is_none(),
            "Should not contain non-existent"
        );
    }
}
