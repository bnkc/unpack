use clap::Parser;
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

fn main() {
    let args = Arguments::parse();
    let deps = get_deps(&args.path);
    println!("{:?}", deps)
}
