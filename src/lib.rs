mod exit_codes;

use rustpython_parser::{ast, lexer::lex, parse_tokens, Mode, ParseError};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
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
pub fn get_deps(dir: &PathBuf) -> Result<Vec<ast::Identifier>, std::io::Error> {
    let walker = WalkDir::new(dir).into_iter();
    let mut deps_set = HashSet::new();

    for entry in walker.filter_map(|e| e.ok()) {
        if entry.file_name().to_string_lossy().ends_with(".py") {
            let file_content = match fs::read_to_string(entry.path()) {
                Ok(content) => content,
                Err(_) => continue,
            };

            if let Ok(ast) = parse_ast(&file_content) {
                if let Some(module) = ast.module() {
                    collect_imports(&module.body, &mut deps_set);
                }
            }
        }
    }

    Ok(deps_set.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let ast = parse_ast(file_content).unwrap();
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
    fn test_get_deps() {
        // let dir = PathBuf::from("tests/fixtures");
        // let deps = get_deps(&dir).unwrap();
        // assert_eq!(deps.len(), 2);
        // assert_eq!(deps[0].name, "os");
        // assert_eq!(deps[1].name, "sys");
    }
}
