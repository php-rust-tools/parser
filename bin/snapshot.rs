use std::env;
use std::fs::read_dir;
use std::io;
use std::path::PathBuf;

use php_parser_rs::parse;

fn main() -> io::Result<()> {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let mut entries = read_dir(manifest.join("tests").join("fixtures"))?
        .flatten()
        .map(|entry| entry.path())
        .filter(|entry| entry.is_dir())
        .collect::<Vec<PathBuf>>();

    entries.sort();

    for entry in entries {
        let code_filename = entry.join("code.php");
        let ast_filename = entry.join("ast.txt");
        let error_filename = entry.join("error.txt");

        if !code_filename.exists() {
            continue;
        }

        if ast_filename.exists() {
            std::fs::remove_file(&ast_filename)?;
        }

        if error_filename.exists() {
            std::fs::remove_file(&error_filename)?;
        }

        let code = std::fs::read_to_string(&code_filename)?;

        match parse(&code) {
            Ok(ast) => {
                std::fs::write(ast_filename, format!("{:#?}\n", ast))?;
                println!("✅ generated `ast.txt` for `{}`", entry.to_string_lossy());
            }
            Err(error) => {
                std::fs::write(
                    error_filename,
                    format!("{}\n", error.report(&code, Some("code.php"), false, true)?),
                )?;

                println!("✅ generated `error.txt` for `{}`", entry.to_string_lossy());
            }
        }
    }

    Ok(())
}
