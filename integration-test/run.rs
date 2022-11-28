use std::env;
use std::process;
use std::{fs::read_dir, path::PathBuf};

use php_parser_rs::{self, Lexer, Parser};

fn main() {
    let directory = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("integration-test");
    let mut entries = read_dir(directory)
        .unwrap()
        .flatten()
        .map(|entry| entry.path())
        .filter(|entry| entry.is_dir())
        .collect::<Vec<PathBuf>>();

    entries.sort();

    let mut errors = 0;
    for entry in entries {
        let code_filename = entry.join("code.php");
        let parser_error_filename = entry.join("parser-error.txt");
        let lexer_error_filename = entry.join("lexer-error.txt");
        if !code_filename.exists() {
            panic!("unable to locate `code.php` in `{:?}`", entry);
        }

        let code = std::fs::read_to_string(&code_filename).unwrap();

        let mut lexer = Lexer::new(None);
        let tokens = lexer.tokenize(code.as_bytes());
        match tokens {
            Ok(tokens) => {
                if lexer_error_filename.exists() {
                    println!(
                        "❕ -> `lexer-error.txt` is present for `{:?}`, but no lexer error was encountered.",
                        entry
                    );
                }

                let mut parser = Parser::new(None);
                let ast = parser.parse(tokens);

                if let Err(e) = ast {
                    if !parser_error_filename.exists() {
                        panic!("unable to locate `parser-error.txt` in `{:?}`", entry);
                    }

                    let error = std::fs::read_to_string(&parser_error_filename).unwrap();
                    if error.trim() != format!("{:#?}", e) {
                        println!("❌ -> mismatching parser error for `{:?}`", entry);
                        println!("{:#?}", e);

                        errors += 1;
                    }
                } else if parser_error_filename.exists() {
                    println!(
                        "❕ -> `parser-error.txt` is present for `{:?}`, but no parser error was encountered.",
                        entry
                    );
                }
            }
            Err(e) => {
                if !lexer_error_filename.exists() {
                    panic!("unable to locate `lexer-error.txt` in `{:?}`", entry);
                }

                let error = std::fs::read_to_string(&lexer_error_filename).unwrap();
                if error.trim() != format!("{:#?}", e) {
                    println!("❌ -> mismatching lexer error for `{:?}`", entry);
                    println!("{:#?}", e);

                    errors += 1;
                }
            }
        }

        println!("✅ -> `{:?}`", entry);
    }

    if errors > 0 {
        println!();
        println!("❌❌ {} error(s) found ❌❌", errors);

        process::exit(1)
    }
}
