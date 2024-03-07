// another name could be prune-deps
// or prune-udeps
// or prunes-rs

mod exit_codes;

use crate::exit_codes::ExitCode;
use anyhow::Ok;
use anyhow::Result;
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
    // println!("{:?}", deps);
    // do some temp stuff
    // check if deps is empty
    // if it is, return ExitCode::HasResults(false)
    // else, return ExitCode::HasResults(true)
    if deps.is_ok() {
        let deps = deps.unwrap();
        if deps.is_empty() {
            Ok(ExitCode::HasResults(false))
        } else {
            Ok(ExitCode::HasResults(true))
        }
    } else {
        Ok(ExitCode::GeneralError)
    }
}

fn main() {
    let result = run();
    match result {
        Ok(exit_code) => {
            exit_code.exit();
        }
        Err(err) => {
            eprintln!("[fd error]: {:#}", err);
            ExitCode::GeneralError.exit();
        }
    }
}
