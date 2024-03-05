use rustpython_parser::{ast, lexer::lex, parse_tokens, Mode, ParseError};
use std::collections::HashSet;

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

pub(crate) fn get_deps(file_content: &str) -> Vec<ast::Identifier> {
    let ast = parse_ast(file_content).expect("Error parsing the file");

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
