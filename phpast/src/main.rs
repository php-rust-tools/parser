use std::{path::PathBuf, process::exit};
use serde_json::to_string;
use structopt::StructOpt;
use trunk_lexer::Lexer;
use trunk_parser::Parser;

#[derive(Debug, StructOpt)]
#[structopt(name = "phpast", about = "Generate an abstract syntax tree from a PHP file.")]
struct Args {
    #[structopt(parse(from_os_str), help = "The input file to use.")]
    file: PathBuf,

    #[structopt(short, long, help = "Output the abstract syntax tree as JSON.")]
    json: bool,
}

fn main() {
    let args = Args::from_args();
    
    let input = match std::fs::read_to_string(args.file) {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        },
    };

    let mut lexer = Lexer::new(None);
    let tokens = lexer.tokenize(&input[..]).unwrap();

    let mut parser = Parser::new(tokens);
    let ast = match parser.parse() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        },
    };

    if args.json {
        match to_string(&ast) {
            Ok(json) => println!("{}", json),
            Err(e) => {
                eprintln!("Failed to generate JSON, error: {}", e);
                exit(1);
            }
        };
    } else {
        println!("{:#?}", ast);
    }
}
