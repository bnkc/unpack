mod config;
mod mappings;

use clap::Parser;
use config::parse;
use rustpython_parser::ast::located::Expr;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(short, long, value_parser)]
    file: String,
}

fn main() {
    let cli: Cli = Cli::parse();
    let file: String = std::fs::read_to_string(cli.file).unwrap(); // This is a temp solution
    let ast: rustpython_parser::ast::Mod = parse(&file).unwrap();
    println!("{:#?}", ast);

    // map the types from python to rust
}
