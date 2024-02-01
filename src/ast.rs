use rustpython_parser::{lexer::lex, parse_tokens, Mode};

/// Parse the given source code and return the resulting AST.
pub fn parse(source: &str) -> Result<rustpython_parser::ast::Mod, rustpython_parser::ParseError> {
    parse_tokens(lex(source, Mode::Module), Mode::Module, "<embedded>")
}
