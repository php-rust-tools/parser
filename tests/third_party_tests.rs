use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use php_parser_rs::prelude::Lexer;
use php_parser_rs::prelude::Parser;

#[test]
fn third_party_php_standard_library() {
    test_repository(
        "php-standard-library",
        "https://github.com/azjezz/psl.git",
        "2.2.x",
        &["src", "tests", "examples"],
    );
}

fn test_repository(name: &str, repository: &str, branch: &str, directories: &[&str]) {
    let out_dir = env::var_os("OUT_DIR").expect("failed to get OUT_DIR");
    let out_path = PathBuf::from(out_dir).join(name);

    if !out_path.exists() {
        let output = Command::new("git")
            .arg("clone")
            .arg("--depth")
            .arg("1")
            .arg("-b")
            .arg(branch)
            .arg(repository)
            .arg(&out_path)
            .output()
            .expect("failed to  run git.");

        if !output.status.success() {
            panic!("failed to clone repository: {:#?}", output)
        }
    }

    for dir in directories {
        test_directory(out_path.clone(), out_path.join(dir));
    }
}

fn test_directory(root: PathBuf, directory: PathBuf) {
    let mut entries = fs::read_dir(directory)
        .unwrap()
        .flatten()
        .map(|entry| entry.path())
        .collect::<Vec<PathBuf>>();

    entries.sort();

    for entry in entries {
        if entry.is_dir() {
            test_directory(root.clone(), entry);

            continue;
        }

        if entry.is_file() && entry.extension().unwrap_or_default() == "php" {
            let name_entry = entry.clone();
            let fullanme_string = name_entry.to_string_lossy();
            let name = fullanme_string
                .strip_prefix(root.to_str().unwrap())
                .unwrap();

            test_file(name, entry);
        }
    }
}

fn test_file(name: &str, filename: PathBuf) {
    let code = std::fs::read_to_string(&filename).unwrap();

    Lexer::new()
        .tokenize(code.as_bytes())
        .map(|tokens| {
            Parser::new()
                .parse(tokens)
                .map(|_| {
                    println!("✅ successfully parsed file: `\"{}\"`.", name);
                })
                .unwrap_or_else(|error| {
                    panic!("❌ failed to parse file: `\"{name}\"`, error: {error:?}")
                })
        })
        .unwrap_or_else(|error| {
            panic!("❌ failed to tokenize file: `\"{name}\"`, error: {error:?}")
        });
}
