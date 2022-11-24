use cmder::ParserMatches;
use trunk_lexer::Lexer;
use trunk_parser::Parser;

use crate::engine::eval;

pub fn run(matches: ParserMatches) {
    // FIXME: Better error handling needed.
    let file = matches.get_arg("file").unwrap();
    let contents = std::fs::read_to_string(file).unwrap();

    let mut lexer = Lexer::new(None);
    let tokens = lexer.tokenize(&contents).unwrap();

    let mut parser = Parser::new(None);
    let program = parser.parse(tokens).unwrap();

    eval(program);
}