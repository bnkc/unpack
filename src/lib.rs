mod exit_codes;

use exit_codes::ExitCode;
use rustpython_parser::{ast, lexer::lex, parse_tokens, Mode, ParseError};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use toml::Table;
use walkdir::WalkDir;

// Extracts the first part of an import statement, which is the module name.
///
///
/// # Arguments
///
/// * `import` - A reference to the import statement to extract the module name from.
///
/// # Returns
///
/// An ast::Identifier containing the first part of the import statement.
fn extract_first_part_of_import(import: &str) -> ast::Identifier {
    import.split('.').next().unwrap_or_default().into()
}

// Parses the AST of a Python file.
///
/// # Arguments
///
/// * `file_content` - A reference to the file content to parse.
///
/// # Returns
///
/// A Result containing the parsed ast::Mod on success, or a ParseError on failure.
fn parse_ast(file_content: &str) -> Result<ast::Mod, ParseError> {
    parse_tokens(lex(file_content, Mode::Module), Mode::Module, "<embedded>")
}

// Collects identifiers from import statements in the specified AST.
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
pub fn get_used_dependencies(dir: &PathBuf) -> Result<Vec<ast::Identifier>, std::io::Error> {
    let walker = WalkDir::new(dir).into_iter();
    let mut used_dependencies = HashSet::new();

    for entry in walker.filter_map(|e| e.ok()) {
        if entry.file_name().to_string_lossy().ends_with(".py") {
            let file_content = match fs::read_to_string(entry.path()) {
                Ok(content) => content,
                Err(_) => continue,
            };

            if let Ok(ast) = parse_ast(&file_content) {
                if let Some(module) = ast.module() {
                    collect_imports(&module.body, &mut used_dependencies);
                }
            }
        }
    }

    Ok(used_dependencies.into_iter().collect())
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
pub fn check_for_dependency_specification_files(base_directory: &PathBuf) -> bool {
    base_directory.ancestors().any(|directory| {
        // Might be adding more here!!! Not sure yet
        let files = vec!["requirements.txt", "pyproject.toml"];
        files
            .iter()
            .any(|&file_name| directory.join(file_name).exists())
    })
}

#[derive(Debug)]
struct Package {
    name: String,
    version: String,
}

// get all the packages in the pyproject.toml file
pub fn get_packages_from_pyproject_toml() -> Result<Vec<String>, ExitCode> {
    let file_path = PathBuf::from("/Users/lev/Developer/ghost-website/pyproject.toml");

    let toml_str = fs::read_to_string(file_path).map_err(|_| ExitCode::GeneralError)?;
    let toml: Table = toml::from_str(&toml_str).map_err(|_| ExitCode::GeneralError)?;

    // let mut packages = Vec::new();
    if let Some(packs) = toml
        .get("tool")
        .and_then(|t| t.get("poetry"))
        .and_then(|p| p.get("dependencies"))
        .and_then(|d| d.as_table())
    {
        for (name, version) in packs {
            // println!("{}: {}", name, version);
            // check if the version is a string. if it's a hash table, we need to extract the "version" key
            // if it's a string, we can just use it
            let version = if let Some(version) = version.as_str() {
                version.to_string()
            } else if let Some(version) = version.as_table() {
                version
                    .get("version")
                    .and_then(|v| v.as_str())
                    .ok_or(ExitCode::GeneralError)?
                    .to_string()
            } else {
                return Err(ExitCode::GeneralError);
            };
            // println!("{}: {}", name, version);
            let my_p = Package {
                name: name.to_string(),
                version,
            };
            // print the Package struct
            println!("{:#?}", my_p);
        }
    }

    todo!()
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
    fn test_parse_ast() {
        let file_content = "import os";
        let ast = parse_ast(file_content);
        assert!(ast.is_ok());

        let file_content = "import os, sys";
        let ast = parse_ast(file_content);
        assert!(ast.is_ok());

        // let's do one where it returns an error
        let file_content = "import os,";
        let ast = parse_ast(file_content);
        // THIS IS IMPORTANT TO KNOW. WHEN A TOP LEVEL IMPORT FAILS, THE WHOLE FILE FAILS
        assert!(ast.is_err());

        // Let's check the actual parsing
        let file_content = "import os";
        let ast = parse_ast(file_content).unwrap();

        assert_eq!(ast.clone().module().unwrap().body.len(), 1);

        let body = &ast.module().unwrap().body;
        let mut temp_deps_set: HashSet<ast::Identifier> = HashSet::new();
        collect_imports(body, &mut temp_deps_set);

        assert_eq!(temp_deps_set.len(), 1);
        assert!(temp_deps_set.contains(&ast::Identifier::new("os")));
    }
    #[test]
    fn test_collect_imports() {
        let file_content = "import os";
        let ast = parse_ast(file_content).unwrap();
        let body = &ast.module().unwrap().body;
        let mut temp_deps_set: HashSet<ast::Identifier> = HashSet::new();
        collect_imports(body, &mut temp_deps_set);
        assert_eq!(temp_deps_set.len(), 1);
        assert!(temp_deps_set.contains(&ast::Identifier::new("os")));

        let file_content = "import os, sys";
        let ast = parse_ast(file_content).unwrap();
        let body = &ast.module().unwrap().body;
        let mut temp_deps_set: HashSet<ast::Identifier> = HashSet::new();
        collect_imports(body, &mut temp_deps_set);
        assert_eq!(temp_deps_set.len(), 2);
        assert!(temp_deps_set.contains(&ast::Identifier::new("os")));
        assert!(temp_deps_set.contains(&ast::Identifier::new("sys")));

        let file_content = "from os import path";
        let ast: ast::Mod = parse_ast(file_content).unwrap();
        let body = &ast.module().unwrap().body;
        let mut temp_deps_set: HashSet<ast::Identifier> = HashSet::new();
        collect_imports(body, &mut temp_deps_set);
        assert_eq!(temp_deps_set.len(), 1);
        assert!(temp_deps_set.contains(&ast::Identifier::new("os")));

        let file_content = "from os import path, sys";
        let ast = parse_ast(file_content).unwrap();
        let body = &ast.module().unwrap().body;
        let mut temp_deps_set: HashSet<ast::Identifier> = HashSet::new();
        collect_imports(body, &mut temp_deps_set);
        assert_eq!(temp_deps_set.len(), 1);
        assert!(temp_deps_set.contains(&ast::Identifier::new("os")));
    }

    #[test]
    fn test_for_dependency_specification_files() {
        let temp_dir = create_working_directory(
            &["dir1", "dir2"],
            Some(&["requirements.txt", "pyproject.toml"]),
        )
        .unwrap();
        let base_directory = temp_dir.path().join("dir1");
        assert!(check_for_dependency_specification_files(&base_directory));

        let temp_dir = create_working_directory(
            &["dir1", "dir2"],
            Some(&["requirements.txt", "pyproject.toml"]),
        )
        .unwrap();
        let base_directory = temp_dir.path().join("dir2");
        assert!(check_for_dependency_specification_files(&base_directory));

        // now write one where it's false
        let temp_dir = create_working_directory(&["dir1", "dir2"], None).unwrap();
        let base_directory = temp_dir.path().join("dir2");
        assert!(!check_for_dependency_specification_files(&base_directory));
    }

    #[test]
    fn test_get_used_dependencies() {
        let temp_dir = create_working_directory(
            &["dir1", "dir2"],
            Some(&["requirements.txt", "pyproject.toml"]),
        )
        .unwrap();
        let base_directory = temp_dir.path().join("dir1");
        let used_dependencies = get_used_dependencies(&base_directory).unwrap();
        assert_eq!(used_dependencies.len(), 0);

        let temp_dir = create_working_directory(
            &["dir1", "dir2"],
            Some(&["requirements.txt", "pyproject.toml", "file1.py"]),
        )
        .unwrap();
        let base_directory = temp_dir.path().join("dir1");
        let used_dependencies = get_used_dependencies(&base_directory).unwrap();
        assert_eq!(used_dependencies.len(), 0);

        let temp_dir = create_working_directory(
            &["dir1", "dir2"],
            Some(&["requirements.txt", "pyproject.toml", "file1.py"]),
        )
        .unwrap();
        let base_directory = temp_dir.path().join("dir2");
        let used_dependencies = get_used_dependencies(&base_directory).unwrap();
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

        let used_dependencies = get_used_dependencies(&base_directory).unwrap();
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
        let used_dependencies = get_used_dependencies(&base_directory).unwrap();
        assert_eq!(used_dependencies.len(), 2);
        assert!(used_dependencies.contains(&ast::Identifier::new("os")));
    }
}
