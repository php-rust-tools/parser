use std::{path::PathBuf, process::exit};
use serde_json::to_string;
use structopt::StructOpt;
use trunk_lexer::Lexer;
use trunk_parser::Parser;

#[derive(Debug, StructOpt)]
#[structopt(name = "phpast", about = "Generate an abstract syntax tree from a PHP file.")]
struct Args {
    #[structopt(parse(from_os_str), help = "The input file to use.")]
    file: Option<PathBuf>,

    #[structopt(short, long, help = "Output the abstract syntax tree as JSON.")]
    json: bool,

    #[structopt(short, long, help = "Only execute the lexer on the source file.")]
    lexer: bool,

    #[structopt(short, long, help = "Provide a string to execute.")]
    run: Option<String>,

    #[structopt(short, long, help = "Dump tokens.")]
    dump_tokens: bool,

    #[structopt(short, long, help = "Print the AST.")]
    print: bool,
}

fn main() {
    let args = Args::from_args();
    
    let input = if args.file.is_some() {
        match std::fs::read_to_string(args.file.unwrap()) {
            Ok(contents) => contents,
            Err(e) => {
                eprintln!("{}", e);
                exit(1);
            },
        }
    } else if args.run.is_some() {
        args.run.unwrap()
    } else {
        panic!("boo!");
    };

    let mut lexer = Lexer::new(None);
    let tokens = lexer.tokenize(&input[..]).unwrap();

    if args.dump_tokens {
        dbg!(&tokens);
    }

    if args.lexer {
        return;
    }

    let mut parser = Parser::new(None);
    let ast = match parser.parse(tokens) {
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
    } else if args.print {
        println!("{:#?}", ast);
    }
}
