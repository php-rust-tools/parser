use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::thread;

use php_parser_rs::lexer::Lexer;

enum TestResult {
    Success,
    Error(String),
}

#[test]
fn third_party_1_php_standard_library() {
    test_repository(
        "php-standard-library",
        "https://github.com/azjezz/psl.git",
        "2.2.x",
        &["src", "tests", "examples"],
        &[],
    );
}

#[test]
fn third_party_2_laravel_framework() {
    test_repository(
        "laravel-framework",
        "https://github.com/laravel/framework",
        "9.x",
        &["src", "tests"],
        &[],
    );
}

#[test]
fn third_party_3_symfony_framework() {
    test_repository(
        "symfony-framework",
        "https://github.com/symfony/symfony",
        "6.3",
        &["src/Symfony/"],
        &[
            // stub
            "src/Symfony/Bridge/ProxyManager/Tests/LazyProxy/PhpDumper/Fixtures/proxy-implem.php",
            // file contains syntax error used for testing.
            "src/Symfony/Component/Config/Tests/Fixtures/ParseError.php",
        ],
    );
}

#[test]
fn third_party_4_nikic_php_parser() {
    test_repository(
        "nikic/PHP-Parser",
        "https://github.com/nikic/PHP-Parser",
        "4.x",
        &["lib/PhpParser", "grammar", "test"],
        &[],
    );
}

#[test]
fn third_party_5_yii_framework() {
    test_repository(
        "yii-framework",
        "https://github.com/yiisoft/yii2",
        "master",
        &["framework", "tests"],
        &[],
    );
}

#[test]
fn third_party_6_spiral_framework() {
    test_repository(
        "spiral-framework",
        "https://github.com/spiral/framework",
        "master",
        &["src", "tests"],
        &[
            // file contains syntax error used for testing.
            "/src/Core/tests/Fixtures/CorruptedClass.php",
            "/src/Tokenizer/tests/Classes/BrokenClass.php",
        ],
    );
}

#[test]
fn third_party_7_mezzio_framework() {
    test_repository(
        "mezzio-framework",
        "https://github.com/mezzio/mezzio",
        "3.15.x",
        &["src", "test"],
        &[],
    );
}

fn test_repository(
    name: &str,
    repository: &str,
    branch: &str,
    directories: &[&str],
    ignore: &[&str],
) {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = manifest.join("target").join("third-party");
    if !out_dir.exists() {
        std::fs::create_dir(&out_dir).unwrap();
    }

    let out_path = out_dir.join(name);

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

    let mut entries = vec![];
    for dir in directories {
        entries.append(&mut read_directory(
            out_path.clone(),
            out_path.join(dir),
            ignore,
        ));
    }

    let mut threads = vec![];
    for (index, chunk) in entries.chunks(entries.len() / 4).enumerate() {
        let chunk = chunk.to_vec();
        let thread = thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .name(format!("{name}:{index}"))
            .spawn(move || {
                let thread = thread::current();
                let thread_name = thread.name().unwrap();

                let mut results = vec![];
                for (name, filename) in chunk {
                    let code = std::fs::read(&filename).unwrap();

                    match Lexer::new().tokenize(&code) {
                        Ok(tokens) => match php_parser_rs::parse(tokens) {
                            Ok(ast) => {
                                println!("✅ [{thread_name}][{name}]: {} statement(s).", ast.len());

                                results.push(TestResult::Success);
                            }
                            Err(error) => {
                                results.push(TestResult::Error(format!(
                                    "❌ [{thread_name}][{name}]: {error:?}"
                                )));
                            }
                        },
                        Err(error) => {
                            results.push(TestResult::Error(format!(
                                "❌ [{thread_name}][{name}]: {error:?}"
                            )));
                        }
                    }
                }

                results
            });

        threads.push(thread);
    }

    let mut results = vec![];
    for thread in threads {
        let mut result = thread
            .unwrap_or_else(|e| panic!("failed to spawn thread: {:#?}", e))
            .join()
            .unwrap_or_else(|e| panic!("failed to join thread: {:#?}", e));

        results.append(&mut result);
    }

    let mut fail = false;
    results
        .iter()
        .map(|result| match result {
            TestResult::Error(message) => {
                fail = true;

                println!("{}", message);
            }
            TestResult::Success => {}
        })
        .for_each(drop);

    if fail {
        panic!();
    }
}

fn read_directory(root: PathBuf, directory: PathBuf, ignore: &[&str]) -> Vec<(String, PathBuf)> {
    let mut results = vec![];
    let mut entries = fs::read_dir(&directory)
        .unwrap()
        .flatten()
        .map(|entry| entry.path())
        .collect::<Vec<PathBuf>>();

    entries.sort();

    for entry in entries {
        if entry.is_dir() {
            results.append(&mut read_directory(root.clone(), entry, ignore));

            continue;
        }

        if entry.is_file()
            && entry.extension().unwrap_or_default() == "php"
            && !ignore.contains(
                &entry
                    .as_path()
                    .strip_prefix(&root)
                    .unwrap()
                    .to_str()
                    .unwrap(),
            )
        {
            let name_entry = entry.clone();
            let fullanme_string = name_entry.to_string_lossy();
            let name = fullanme_string
                .strip_prefix(root.to_str().unwrap())
                .unwrap();

            results.push((name.to_string(), entry));
        }
    }

    results
}
