use clap::Parser;
use log::{error, info};
use std::path::PathBuf;

use ast::get_deps;
use std::fs;
use walkdir::WalkDir;

mod analyze;
mod ast;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Arguments {
    #[arg(default_value = ".")]
    pub path: PathBuf,
}

fn setup_logging() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
}

fn main() {
    setup_logging();

    info!("Starting the application");
    let args = Arguments::parse();
    println!("{:?}", args);
    // let file_content = read_file(&cli.file).unwrap();
    // let file_content = "from sklearn.data import datasets";

    // let deps = get_deps(&file_content);
    // println!("{:#?}", deps);

    // Assuming `analyze::TypeChecker` and `ast::parse` are updated to return `Result`
    // let type_check_result = analyze::TypeChecker::new().check(&ast)?;

    // info!("Successfully analyzed the AST: {:#?}", type_check_result);
    // Ok(())
}

fn read_file(path: &str) -> Result<String, std::io::Error> {
    fs::read_to_string(path).map_err(|e| {
        error!("E`rror reading file {}: {}", path, e);
        e.into()
    })
}
