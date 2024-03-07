// another name could be prune-deps
// or prune-udeps
// or prunes-rs

mod exit_codes;

use crate::exit_codes::ExitCode;
use clap::Parser;
use pip_udeps::get_deps;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Arguments {
    #[arg(default_value = ".")]
    pub path: PathBuf,
}

fn run() -> Result<ExitCode> {
    let args = Arguments::parse();
    let deps = get_deps(&args.path);
    deps
    // println!("{:?}", deps);
}

fn main() {
    let result = run();
    match result {
        Ok(exit_code) => {
            exit_code.exit();
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            ExitCode::GeneralError.exit();
        }
    }
}
