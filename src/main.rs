use php_parser_rs::lexer::Lexer;
use php_parser_rs::parser::error::ParseResult;
use php_parser_rs::parser::Parser;

fn main() -> ParseResult<()> {
    let file = match std::env::args().nth(1) {
        Some(file) => file,
        None => {
            println!("Usage: php-parser [file]");

            ::std::process::exit(0);
        }
    };

    let contents = match std::fs::read(&file) {
        Ok(contents) => contents,
        Err(error) => {
            println!("Failed to read file: {}", error);

            ::std::process::exit(1);
        }
    };

    let lexer = Lexer::new();
    let parser = Parser::new();

    let tokens = lexer.tokenize(&contents)?;
    dbg!(&tokens);

    let ast = parser.parse(tokens)?;

    dbg!(ast);

    Ok(())
}
