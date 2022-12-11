use php_parser_rs::lexer::Lexer;
use php_parser_rs::parser::error::ParseResult;

fn main() -> ParseResult<()> {
    let file = match std::env::args().nth(1) {
        Some(file) => file,
        None => {
            eprintln!("Usage: php-parser [file]");

            ::std::process::exit(0);
        }
    };

    let contents = match std::fs::read(&file) {
        Ok(contents) => contents,
        Err(error) => {
            eprintln!("Failed to read file: {}", error);

            ::std::process::exit(1);
        }
    };

    let tokens = Lexer::new().tokenize(&contents)?;
    let ast = php_parser_rs::parse(&tokens)?;

    match serde_json::to_string_pretty(&ast) {
        Ok(json) => {
            println!("{}", json);
        }
        Err(e) => {
            eprintln!("Failed to serialize AST: {}", e);

            ::std::process::exit(1);
        }
    }

    Ok(())
}
