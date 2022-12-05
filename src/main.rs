use php_parser_rs::prelude::Lexer;
use php_parser_rs::prelude::Parser;

fn main() {
    let file = match std::env::args().nth(1) {
        Some(file) => file,
        None => {
            println!("Usage: php-parser [file]");

            ::std::process::exit(0);
        }
    };

    let contents = match std::fs::read_to_string(&file) {
        Ok(contents) => contents,
        Err(error) => {
            println!("Failed to read file: {}", error);

            ::std::process::exit(1);
        }
    };

    let lexer = Lexer::new();
    let tokens = match lexer.tokenize(contents.as_bytes()) {
        Ok(tokens) => tokens,
        Err(error) => {
            println!("{}", error);

            ::std::process::exit(1);
        }
    };

    let parser = Parser::new();
    let ast = match parser.parse(tokens) {
        Ok(ast) => ast,
        Err(error) => {
            println!("{}", error);

            ::std::process::exit(1);
        }
    };

    dbg!(ast);
}
