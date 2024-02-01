mod analyze;
mod ast;

use analyze::TypeChecker;
use ast::parse;
use clap::Parser;

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
    let mut type_checker = TypeChecker::new();

    type_checker.visit_mod(&ast);
}
