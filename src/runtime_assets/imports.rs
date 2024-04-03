use std::collections::HashSet;
use std::fs;

use std::path::PathBuf;
use std::str;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::Result;

use ignore::{self, WalkBuilder, WalkParallel, WalkState};
use rustpython_ast::Visitor;
use rustpython_parser::{ast, parse, Mode};

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

fn build_walker(config: &Config) -> Result<WalkParallel> {
    let builder = WalkBuilder::new(&config.base_directory)
        .hidden(config.ignore_hidden) // Configure visibility of hidden files
        .filter_entry(|entry| {
            // Directly filter for `.py` files
            entry.path().extension().map_or(false, |ext| ext == "py")
        })
        .build_parallel(); // Builds the walker with parallelism support

    // Optional: Add custom ignore patterns or files
    // if let Some(excludes) = &config.excludes {
    //     for exclude in excludes {
    //         builder.add_ignore(exclude); // This method does not exist; you would need a custom implementation
    //     }
    // }

    Ok(builder)
}

/// Initiates the parallel processing of Python files to extract import statements.
pub fn get_imports(config: &Config) -> Result<HashSet<String>> {
    let walker = build_walker(config)?;
    let (tx, rx) = mpsc::channel();
    let tx = Arc::new(tx);

    walker.run(move || {
        let tx = Arc::clone(&tx);
        Box::new(move |result| {
            if let Ok(entry) = result {
                // skip the root directory
                if entry.depth() == 0 {
                    return WalkState::Continue;
                }

                // Now assured to be a .py file, proceed to process.
                let path = entry.path().to_owned();
                let tx = tx.clone();
                process_file(path, tx);
            }
            ignore::WalkState::Continue
        })
    });

    // Collect all the import statements from the threads
    let mut imports = HashSet::new();
    for recieved in rx.iter() {
        imports.extend(recieved);
    }

    Ok(imports)
}

/// Processes a single Python file to extract and send its import statements through a channel.
fn process_file(path: PathBuf, tx: Arc<mpsc::Sender<HashSet<String>>>) {
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file {:?}: {}", path, e);
            return;
        }
    };

    let ast = match parse(&content, Mode::Module, "<embedded>") {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Error parsing file {:?}: {}", path, e);
            return;
        }
    };

    let mut collector = Imports {
        manifest: HashSet::new(),
    };
    ast.module()
        .unwrap()
        .body
        .into_iter()
        .for_each(|node| collector.visit_stmt(node));

    // Attempt to send collected imports, log any failure to do so.
    if tx.send(collector.manifest).is_err() {
        eprintln!("Failed to send data for file {:?}", path);
    }
}

// pub fn get_imports(config: &Config) -> Result<HashSet<String>> {
//     WalkDir::new(&config.base_directory)
//         .into_iter()
//         .filter_map(|e| e.ok())
//         .filter(|entry| {
//             let file_name = entry.file_name().to_string_lossy();

//             // Ignore hidden files and directories if `ignore_hidden` is set to true
//             file_name.ends_with(".py") && !(config.ignore_hidden && file_name.starts_with("."))
//         })
//         .try_fold(HashSet::new(), |mut acc, entry| {
//             let file_content = fs::read_to_string(entry.path())?;
//             let module = parse(&file_content, Mode::Module, "<embedded>")?;

//             let mut collector = Imports {
//                 manifest: HashSet::new(),
//             };

//             module
//                 .module()
//                 .unwrap() //Probably should change this from unwrap to something else
//                 .body
//                 .into_iter()
//                 .for_each(|node| collector.visit_stmt(node));

//             acc.extend(collector.manifest);

//             Ok(acc)
//         })
// }
