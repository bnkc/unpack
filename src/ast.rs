use rustpython_parser::{ast::Identifier, lexer::lex, parse_tokens, Mode};

pub(crate) fn parse_ast(
    file_content: &str,
) -> Result<rustpython_parser::ast::Mod, rustpython_parser::ParseError> {
    parse_tokens(lex(file_content, Mode::Module), Mode::Module, "<embedded>")
}

pub(crate) fn get_deps(ast: rustpython_parser::ast::Mod) -> Vec<Identifier> {
    let mut deps = vec![];
    let module = match ast.module() {
        Some(body) => body,
        None => return deps,
    };

    for stmt in module.body {
        match stmt {
            rustpython_parser::ast::Stmt::Import(import) => {
                for alias in &import.names {
                    deps.push(alias.name.clone());
                }
            }
            rustpython_parser::ast::Stmt::ImportFrom(import) => {
                for alias in &import.names {
                    deps.push(alias.name.clone());
                }
            }
            _ => {}
        }
    }
    deps
}

// write a unit test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ast() {
        let file_content = "import numpy as np";
        let ast = parse_ast(file_content).unwrap();
        // assert_eq!(format!("{:#?}", ast), format!("{:#?}", ""));
    }
}
