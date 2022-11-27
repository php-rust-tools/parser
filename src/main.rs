use php_parser_rs::{Lexer, Parser};

fn main() {
    let file = std::env::args().nth(1).unwrap();
    let contents = std::fs::read_to_string(&file).unwrap();

    println!("> Parsing {}", file);

    let mut lexer = Lexer::new(None);
    let tokens = lexer.tokenize(contents.as_bytes()).unwrap();

    let mut parser = Parser::new(None);
    let ast = parser.parse(tokens).unwrap();

    dbg!(ast);
}
