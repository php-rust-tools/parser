use std::{fs::canonicalize, path::PathBuf};

use cmder::ParserMatches;
use trunk_lexer::Lexer;
use trunk_parser::Parser;

use crate::engine::eval;

pub fn run(matches: ParserMatches) {
    // FIXME: Better error handling needed.
    let file = PathBuf::from(matches.get_arg("file").unwrap());
    let contents = std::fs::read_to_string(&file).unwrap();
    let abs_filename = canonicalize(&file).unwrap();

    let mut lexer = Lexer::new(None);
    let tokens = lexer.tokenize(&contents).unwrap();

    let mut parser = Parser::new(None);
    let program = parser.parse(tokens).unwrap();

    eval(abs_filename, program).ok();
}
