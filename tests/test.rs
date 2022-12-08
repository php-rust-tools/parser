use std::env;
use std::fs::read_dir;
use std::path::PathBuf;

use pretty_assertions::assert_str_eq;

use php_parser_rs::lexer::Lexer;

static LEXER: Lexer = Lexer::new();

#[test]
fn test_fixtures() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let tests = manifest.join("tests").join("fixtures");

    let mut entries = read_dir(tests)
        .unwrap()
        .flatten()
        .map(|entry| entry.path())
        .filter(|entry| entry.is_dir())
        .collect::<Vec<PathBuf>>();

    entries.sort();

    for entry in entries {
        let fixture = entry.file_name().unwrap().to_string_lossy();

        let code_file = entry.join("code.php");
        let ast_file = entry.join("ast.txt");
        let lex_err_file = entry.join("lexer-error.txt");
        let parse_err_file = entry.join("parser-error.txt");

        if !code_file.exists() {
            continue;
        }

        let code = std::fs::read(&code_file).unwrap();

        if lex_err_file.exists() {
            let expected_error = std::fs::read_to_string(&lex_err_file).unwrap();
            let error = LEXER.tokenize(&code).err().unwrap();

            assert_str_eq!(
                expected_error.trim(),
                format!("{:?} -> {}", error, error),
                "lexer error mismatch for fixture `{}`",
                fixture
            );

            continue;
        }

        let tokens = LEXER.tokenize(&code).unwrap();

        if ast_file.exists() {
            let expected_ast = std::fs::read_to_string(&ast_file).unwrap();
            let ast = php_parser_rs::parse(tokens).unwrap();
            assert_str_eq!(
                expected_ast.trim(),
                format!("{:#?}", ast),
                "ast mismatch for fixture `{}`",
                fixture
            );

            continue;
        }

        assert!(
            parse_err_file.exists(),
            "unable to find `parser-error.txt` for `{}`.",
            fixture
        );

        let expected_error = std::fs::read_to_string(&parse_err_file).unwrap();
        let error = php_parser_rs::parse(tokens).err().unwrap();

        assert_str_eq!(
            expected_error.trim(),
            format!("{:?} -> {}", error, error),
            "parse error mismatch for fixture `{}`",
            fixture
        );
    }
}
