use log::error;
use rustpython_parser::{ast, lexer::lex, parse_tokens, Mode};
use std::collections::HashSet;

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
                        deps_set.insert(alias.name.clone());
                    }
                }
                ast::Stmt::ImportFrom(import) => {
                    if let Some(module) = &import.module {
                        let parts: Vec<&str> = module.split('.').collect();
                        // we only care about the first part of the import
                        deps_set.insert(ast::Identifier::new(parts[0].to_string()));
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
