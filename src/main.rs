use clap::Parser;
use log::info;
use std::path::PathBuf;

use ast::get_deps;

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
    let deps = get_deps(&args.path);
    info!("Dependencies: {:?}", deps);
}
