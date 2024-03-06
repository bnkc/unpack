use clap::Parser;
use pip_udeps::get_deps;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Arguments {
    #[arg(default_value = ".")]
    pub path: PathBuf,
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Arguments::parse();
    let deps = get_deps(&args.path);
    println!("{:?}", deps);
    Ok(())
}

fn main() {
    let result = run();
    match result {
        Ok(_) => {}
        Err(err) => {
            eprintln!("[pip-udeps erro]: {:#}", err);
            // at some point code back and make these error code more meaningful
            std::process::exit(1);
        }
    }
}
