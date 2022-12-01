use php_parser_rs::prelude::{Lexer, Parser};
use std::env;
use std::fs::read_dir;
use std::path::PathBuf;

static PARSER: Parser = Parser::new();
static LEXER: Lexer = Lexer::new();

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let mut entries = read_dir(manifest.join("tests"))
        .unwrap()
        .flatten()
        .map(|entry| entry.path())
        .filter(|entry| entry.is_dir())
        .collect::<Vec<PathBuf>>();

    entries.sort();

    for entry in entries {
        let code_filename = entry.join("code.php");
        let ast_filename = entry.join("ast.txt");
        let tokens_filename = entry.join("tokens.txt");
        let lexer_error_filename = entry.join("lexer-error.txt");
        let parser_error_filename = entry.join("parser-error.txt");

        if !code_filename.exists() {
            continue;
        }

        if ast_filename.exists() {
            std::fs::remove_file(&ast_filename).unwrap();
        }

        if tokens_filename.exists() {
            std::fs::remove_file(&tokens_filename).unwrap();
        }

        if lexer_error_filename.exists() {
            std::fs::remove_file(&lexer_error_filename).unwrap();
        }

        if parser_error_filename.exists() {
            std::fs::remove_file(&parser_error_filename).unwrap();
        }

        let code = std::fs::read_to_string(&code_filename).unwrap();
        let tokens = LEXER.tokenize(code.as_bytes());

        match tokens {
            Ok(tokens) => {
                std::fs::write(tokens_filename, format!("{:#?}\n", tokens)).unwrap();
                println!(
                    "✅ generated `tokens.txt` for `{}`",
                    entry.to_string_lossy()
                );

                let ast = PARSER.parse(tokens);
                match ast {
                    Ok(ast) => {
                        std::fs::write(ast_filename, format!("{:#?}\n", ast)).unwrap();
                        println!("✅ generated `ast.txt` for `{}`", entry.to_string_lossy());
                    }
                    Err(error) => {
                        std::fs::write(
                            parser_error_filename,
                            format!("{:?} -> {}\n", error, error),
                        )
                        .unwrap();
                        println!(
                            "✅ generated `parser-error.txt` for `{}`",
                            entry.to_string_lossy()
                        );
                    }
                }
            }
            Err(error) => {
                std::fs::write(lexer_error_filename, format!("{:?} -> {}\n", error, error))
                    .unwrap();
                println!(
                    "✅ generated `lexer-error.txt` for `{}`",
                    entry.to_string_lossy()
                );
            }
        }
    }
}
