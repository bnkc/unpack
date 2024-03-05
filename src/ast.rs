use log::error;
use rustpython_parser::{ast, lexer::lex, parse_tokens, Mode};
use std::collections::HashSet;

fn extract_first_part_of_import(import: &str) -> ast::Identifier {
    let first_part: &str = import.split('.').next().unwrap_or("");
    ast::Identifier::new(first_part.to_string())
}

fn parse_ast(
    file_content: &str,
) -> Result<rustpython_parser::ast::Mod, rustpython_parser::ParseError> {
    parse_tokens(lex(file_content, Mode::Module), Mode::Module, "<embedded>")
}

// Function to recursively collect imports
fn collect_imports(stmts: &[ast::Stmt], deps_set: &mut HashSet<ast::Identifier>) {
    for stmt in stmts {
        match stmt {
            ast::Stmt::Import(import) => {
                for alias in &import.names {
                    let first_part = extract_first_part_of_import(&alias.name);
                    deps_set.insert(first_part);
                }
            }
            ast::Stmt::ImportFrom(import) => {
                if let Some(module) = &import.module {
                    let first_part = extract_first_part_of_import(module);
                    deps_set.insert(first_part);
                }
            }
            ast::Stmt::FunctionDef(function_def) => {
                collect_imports(&function_def.body, deps_set);
            }
            // ast::Stmt::FunctionDef(function_def) => {
            //     collect_imports(&function_def.body, deps_set);
            // }
            _ => {}
        }
    }
}

pub(crate) fn get_deps(file_content: &str) -> Vec<ast::Identifier> {
    let ast = match parse_ast(file_content) {
        Ok(ast) => ast,
        Err(e) => {
            error!("Error parsing the AST: {}", e);
            return vec![];
        }
    };
    let sample_ast = ast.clone();
    println!("{:#?}", sample_ast.module().unwrap().body);

    let mut deps_set = HashSet::new();
    if let Some(modele) = ast.module() {
        collect_imports(&modele.body, &mut deps_set);
    }

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
