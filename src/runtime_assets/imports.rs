use std::collections::HashSet;
use std::fs;

use std::str;

use anyhow::Result;
use rustpython_ast::Visitor;
use rustpython_parser::{ast, parse, Mode};
use walkdir::WalkDir;

use crate::config::Config;

/// Extract the first part of an import statement
///  e.g. `os.path` -> `os`
#[inline]
fn stem_import(import: &str) -> String {
    import.split('.').next().unwrap_or_default().into()
}
/// Collects all the dependencies from the AST
struct Imports {
    manifest: HashSet<String>,
}

impl Visitor for Imports {
    /// This is a generic visit method that will be called for all nodes
    fn visit_stmt(&mut self, node: ast::Stmt<ast::text_size::TextRange>) {
        self.generic_visit_stmt(node);
    }
    /// This method is `overridden` to collect the dependencies into `self.deps`
    fn visit_stmt_import(&mut self, node: ast::StmtImport) {
        node.names.iter().for_each(|alias| {
            self.manifest.insert(stem_import(&alias.name));
        })
    }

    /// This method is `overridden` to collect the dependencies into `self.deps`
    fn visit_stmt_import_from(&mut self, node: ast::StmtImportFrom) {
        if let Some(module) = &node.module {
            self.manifest.insert(stem_import(module));
        }
    }
}

pub fn get_imports(config: &Config) -> Result<HashSet<String>> {
    WalkDir::new(&config.base_directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| {
            let file_name = entry.file_name().to_string_lossy();

            // Ignore hidden files and directories if `ignore_hidden` is set to true
            file_name.ends_with(".py") && !(config.ignore_hidden && file_name.starts_with("."))
        })
        .try_fold(HashSet::new(), |mut acc, entry| {
            let file_content = fs::read_to_string(entry.path())?;
            let module = parse(&file_content, Mode::Module, "<embedded>")?;

            let mut collector = Imports {
                manifest: HashSet::new(),
            };

            module
                .module()
                .unwrap() //Probably should change this from unwrap to something else
                .body
                .into_iter()
                .for_each(|node| collector.visit_stmt(node));

            acc.extend(collector.manifest);

            Ok(acc)
        })
}
