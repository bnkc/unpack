use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::str;
use std::sync::mpsc;
use std::sync::Arc;

use anyhow::Result;
use ignore::{WalkBuilder, WalkParallel, WalkState};
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
struct ImportCollector {
    imports: HashSet<String>,
}

impl Visitor for ImportCollector {
    /// This is a generic visit method that will be called for all nodes
    fn visit_stmt(&mut self, node: ast::Stmt<ast::text_size::TextRange>) {
        self.generic_visit_stmt(node);
    }
    /// This method is `overridden` to collect the dependencies into `self.deps`
    fn visit_stmt_import(&mut self, node: ast::StmtImport) {
        node.names.iter().for_each(|alias| {
            self.imports.insert(stem_import(&alias.name));
        })
    }

    /// This method is `overridden` to collect the dependencies into `self.deps`
    fn visit_stmt_import_from(&mut self, node: ast::StmtImportFrom) {
        if let Some(module) = &node.module {
            self.imports.insert(stem_import(module));
        }
    }
}

fn build_walker(config: &Config) -> Result<WalkParallel> {
    let builder = WalkBuilder::new(&config.base_directory)
        .hidden(config.ignore_hidden)
        .max_depth(config.max_depth)
        .filter_entry(|entry| entry.path().extension().map_or(false, |ext| ext == "py"))
        .build_parallel(); // Builds the walker with parallelism support

    Ok(builder)
}

/// Spawns a thread to process a Python file and extract import statements.
fn sender(path: PathBuf, tx: Arc<mpsc::Sender<HashSet<String>>>) {
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

    let mut collector = ImportCollector {
        imports: HashSet::new(),
    };
    ast.module()
        .unwrap()
        .body
        .into_iter()
        .for_each(|node| collector.visit_stmt(node));

    // Attempt to send collected imports, log any failure to do so.
    if tx.send(collector.imports).is_err() {
        eprintln!("Failed to send data for file {:?}", path);
    }
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

                let path = entry.path().to_owned();
                let tx = tx.clone();
                sender(path, tx);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    use crate::cli::{Env, OutputKind};
    use crate::runtime_assets::PackageState;

    /// Helper function to create a Python file in the temporary directory.
    fn create_file(dir: &tempfile::TempDir, filename: &str, content: &str) -> PathBuf {
        let file_path = dir.path().join(filename);
        let mut file = File::create(&file_path).expect("Failed to create file.");
        writeln!(file, "{}", content).expect("Failed to write to file.");
        file_path
    }

    /// Helper function to create a Config struct for testing.
    fn te_config(base_directory: PathBuf) -> Config {
        Config {
            base_directory,
            ignore_hidden: true,
            max_depth: None,
            package_state: PackageState::Unused,
            dep_spec_file: PathBuf::new(),
            env: Env::Test,
            output: OutputKind::Human,
        }
    }

    /// Tests the `stem_import` function for correct behavior.
    #[test]
    fn test_stem_import() {
        assert_eq!(stem_import("os.path"), "os");
        assert_eq!(stem_import("sys"), "sys");
        // Edge cases
        assert_eq!(stem_import(""), "");
        assert_eq!(stem_import("complex.import.path"), "complex");
    }

    /// Tests processing of a valid Python file with import statements.
    #[test]
    fn test_process_valid_python_file() {
        let temp_dir = tempdir().unwrap();
        create_file(
            &temp_dir,
            "test.py",
            "import os\nimport sys\nfrom collections import defaultdict",
        );

        let config = te_config(temp_dir.path().to_path_buf());

        let imports = get_imports(&config).expect("Failed to get imports");

        assert!(imports.contains("os"));
        assert!(imports.contains("sys"));
        assert!(imports.contains("collections"));
    }

    /// Tests handling of an invalid Python file.
    #[test]
    fn test_process_invalid_python_file() {
        let temp_dir = tempdir().unwrap();
        create_file(&temp_dir, "invalid.py", "This is not valid Python syntax.");
        create_file(&temp_dir, "invalid.txt", "This is not a Python file.");

        let config = te_config(temp_dir.path().to_path_buf());

        let imports = get_imports(&config);

        assert!(imports.is_ok());
        assert!(imports.unwrap().is_empty());
    }

    /// Tests handling of a directory with multiple Python files.
    /// The directory contains a mix of valid and invalid Python files.
    #[test]
    fn test_process_directory_with_multiple_files() {
        // let temp_dir = tempdir().unwrap();
        let temp_dir = tempdir().unwrap();
        let valid_python = "import os\nimport sys\nfrom collections import defaultdict";
        let valid_python2 =
            "import pandas as pd\nimport numpy as np\nimport matplotlib.pyplot as plt";
        let invalid_python = "This is not valid Python syntax.";

        // Create a mix of valid and invalid Python files
        create_file(&temp_dir, "valid.py", valid_python);
        create_file(&temp_dir, "valid2.py", valid_python2);
        create_file(&temp_dir, "invalid.py", invalid_python);

        let config = te_config(temp_dir.path().to_path_buf());

        let imports = get_imports(&config).expect("Failed to get imports");

        assert!(imports.contains("os"));
        assert!(imports.contains("sys"));
        assert!(imports.contains("collections"));
        assert!(imports.contains("pandas"));
        assert!(imports.contains("numpy"));
        assert!(imports.contains("matplotlib"));
    }

    #[test]
    fn test_ignore_non_python_files() {
        let temp_dir = tempdir().unwrap();
        create_file(&temp_dir, "test.py", "import os");
        let non_python_file_path = temp_dir.path().join("README.md");
        let mut non_python_file =
            File::create(&non_python_file_path).expect("Failed to create file.");
        writeln!(non_python_file, "This should not be processed.")
            .expect("Failed to write to file.");

        let config = te_config(temp_dir.path().to_path_buf());

        let imports = get_imports(&config).expect("Failed to get imports");

        assert!(
            imports.contains("os"),
            "The import from the Python file should be processed."
        );
        assert_eq!(
            imports.len(),
            1,
            "Only Python file imports should be processed."
        );
    }
}
