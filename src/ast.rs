use log::error;
use rustpython_parser::{
    ast::{self, Identifier},
    lexer::lex,
    parse_tokens, Mode,
};
use std::collections::HashSet;

// we should move this to utils or to lib or something. this does not belong here
fn extract_first_part_of_import(import: &str) -> ast::Identifier {
    let first_part = import.split('.').next().unwrap_or(""); // Safely handle cases where there might not be a '.'
    ast::Identifier::new(first_part.to_string())
}

fn parse_ast(
    file_content: &str,
) -> Result<rustpython_parser::ast::Mod, rustpython_parser::ParseError> {
    parse_tokens(lex(file_content, Mode::Module), Mode::Module, "<embedded>")
}

pub(crate) fn get_deps(file_content: &str) -> Vec<ast::Identifier> {
    let ast = match parse_ast(file_content) {
        Ok(ast) => ast,
        Err(e) => {
            error!("Error parsing the AST: {}", e);
            return vec![];
        }
    };

    let mut deps_set = HashSet::new();
    if let Some(module) = ast.module() {
        for stmt in module.body {
            match stmt {
                ast::Stmt::Import(import) => {
                    for alias in &import.names {
                        let first_part = extract_first_part_of_import(&alias.name);
                        deps_set.insert(first_part);
                    }
                }
                ast::Stmt::ImportFrom(import) => {
                    if let Some(module) = &import.module {
                        let first_part = extract_first_part_of_import(&module);
                        deps_set.insert(first_part);
                    }
                }
                _ => {}
            }
        }
    }
    // Convert HashSet back into Vec
    deps_set.into_iter().collect()
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
