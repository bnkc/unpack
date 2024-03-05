use rustpython_parser::{ast, lexer::lex, parse_tokens, Mode, ParseError};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

fn extract_first_part_of_import(import: &str) -> ast::Identifier {
    import.split('.').next().unwrap_or_default().into()
}

fn parse_ast(file_content: &str) -> Result<ast::Mod, ParseError> {
    parse_tokens(lex(file_content, Mode::Module), Mode::Module, "<embedded>")
}

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
/// A Result containing a Vec of ast::Identifier on success, or an error string on failure.
pub(crate) fn get_deps(dir: &PathBuf) -> Result<Vec<ast::Identifier>, String> {
    let walker = WalkDir::new(dir);
    let mut deps_set = HashSet::new();

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        if entry.file_name().to_string_lossy().ends_with(".py") {
            let file_content = match fs::read_to_string(entry.path()) {
                Ok(content) => content,
                Err(_) => return Err(format!("Failed to read file: {:?}", entry.path())),
            };

            match parse_ast(&file_content) {
                Ok(ast) => {
                    if let Some(module) = ast.module() {
                        collect_imports(&module.body, &mut deps_set);
                    }
                }
                Err(_) => return Err(format!("Error parsing the file: {:?}", entry.path())),
            }
        }
    }

    Ok(deps_set.into_iter().collect())
}

// // write a unit test
// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_parse_ast() {
//         let file_content = "import numpy as np";
//         let ast = parse_ast(file_content).unwrap();
//         // assert_eq!(format!("{:#?}", ast), format!("{:#?}", ""));
//     }
// }
