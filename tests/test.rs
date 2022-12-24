use std::env;
use std::fs::read_dir;
use std::io;
use std::path::PathBuf;

use pretty_assertions::assert_str_eq;

#[test]
fn test_fixtures() -> io::Result<()> {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let tests = manifest.join("tests").join("fixtures");

    let mut entries = read_dir(tests)?
        .flatten()
        .map(|entry| entry.path())
        .filter(|entry| entry.is_dir())
        .collect::<Vec<PathBuf>>();

    entries.sort();

    for entry in entries {
        let fixture = entry.to_string_lossy();

        let code_file = entry.join("code.php");
        let ast_file = entry.join("ast.txt");
        let error_file = entry.join("error.txt");

        if !code_file.exists() {
            continue;
        }

        let code = std::fs::read_to_string(&code_file)?;

        if ast_file.exists() {
            let expected_ast = std::fs::read_to_string(&ast_file)?;
            let ast = php_parser_rs::parse(&code).unwrap();
            assert_str_eq!(
                expected_ast.trim(),
                format!("{:#?}", ast),
                "ast mismatch for fixture `{}`",
                fixture
            );

            continue;
        }

        assert!(
            error_file.exists(),
            "unable to find `error.txt` for `{}`.",
            fixture
        );

        let expected_error = std::fs::read_to_string(&error_file)?;
        let error = php_parser_rs::parse(&code).err().unwrap();

        assert_str_eq!(
            expected_error.trim(),
            (error.report(&code, Some("code.php"), false, true)?)
                .to_string()
                .trim(),
            "error mismatch for fixture `{}`",
            fixture
        );
    }

    Ok(())
}
