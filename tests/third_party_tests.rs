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
fn php_standard_library() {
    test_repository(
        "php-standard-library",
        "https://github.com/azjezz/psl.git",
        &[],
    );
}

#[test]
fn laravel_framework() {
    test_repository(
        "laravel-framework",
        "https://github.com/laravel/framework",
        &[
            // file contains syntax error for testing.
            "tests/Foundation/fixtures/bad-syntax-strategy.php",
        ],
    );
}

#[test]
fn symfony_framework() {
    test_repository(
        "symfony-framework",
        "https://github.com/symfony/symfony",
        &[
            // stub
            "src/Symfony/Bridge/ProxyManager/Tests/LazyProxy/PhpDumper/Fixtures/proxy-implem.php",
            // file contains syntax error used for testing.
            "src/Symfony/Component/Config/Tests/Fixtures/ParseError.php",
            // file contains unintentional error upstream, waiting for fix.
            "src/Symfony/Component/VarExporter/LazyProxyTrait.php",
        ],
    );
}

#[test]
fn nikic_php_parser() {
    test_repository(
        "nikic/PHP-Parser",
        "https://github.com/nikic/PHP-Parser",
        &[
            "vendor/ircmaxell/php-yacc/examples/00-basic-usage/parser.template.php",
            "vendor/ircmaxell/php-yacc/examples/01-expression-support/parser.template.php",
            "vendor/ircmaxell/php-yacc/examples/02-complex-expression-support/parser.template.php",
            "vendor/ircmaxell/php-yacc/examples/10-php7/parser.kmyacc.php",
            "vendor/ircmaxell/php-yacc/examples/10-php7/parser.phpyacc.php",
            "vendor/ircmaxell/php-yacc/examples/10-php7/parser.template.php",
            "vendor/ircmaxell/php-yacc/examples/20-custom-parser/parser.kmyacc.php",
            "vendor/ircmaxell/php-yacc/examples/20-custom-parser/parser.phpyacc.php",
            "vendor/ircmaxell/php-yacc/examples/20-custom-parser/parser.template.php",
        ],
    );
}

#[test]
fn yii_framework() {
    test_repository("yii-framework", "https://github.com/yiisoft/yii2", &[]);
}

#[test]
fn spiral_framework() {
    test_repository(
        "spiral-framework",
        "https://github.com/spiral/framework",
        &[
            // file contains syntax error used for testing.
            "src/Core/tests/Fixtures/CorruptedClass.php",
            "src/Tokenizer/tests/Classes/BrokenClass.php",
        ],
    );
}

#[test]
fn mezzio_framework() {
    test_repository("mezzio-framework", "https://github.com/mezzio/mezzio", &[]);
}

#[test]
fn symfony_polyfill() {
    test_repository(
        "symfony-polyfill",
        "https://github.com/symfony/polyfill",
        &[],
    );
}

#[test]
fn madelineproto() {
    test_repository(
        "MadelineProto",
        "https://github.com/danog/MadelineProto",
        &[],
    );
}

#[test]
fn composer() {
    test_repository("composer", "https://github.com/composer/composer", &[]);
}

#[test]
fn wordpress() {
    test_repository("wordpress", "https://github.com/WordPress/WordPress", &[]);
}

#[test]
fn chubbyphp_framework() {
    test_repository(
        "chubbyphp-framework",
        "https://github.com/chubbyphp/chubbyphp-framework",
        &[],
    );
}

#[test]
fn silverstripe_framework_core() {
    test_repository(
        "silverstripe-framework",
        "https://github.com/silverstripe/silverstripe-framework",
        &[],
    );
}

#[test]
fn silverstripe_framework_cms() {
    test_repository(
        "silverstripe-cms",
        "https://github.com/silverstripe/silverstripe-cms",
        &[],
    );
}

#[test]
fn roundcubemail_web() {
    test_repository(
        "roundcubemail",
        "https://github.com/roundcube/roundcubemail",
        &[],
    );
}

#[test]
fn phpmyadmin_web() {
    test_repository(
        "phpmyadmin",
        "https://github.com/phpmyadmin/phpmyadmin",
        &[],
    );
}

#[test]
fn phpbb() {
    test_repository("phpbb", "https://github.com/phpbb/phpbb", &[]);
}

#[test]
fn drupal() {
    test_repository("drupal", "https://github.com/drupal/core", &[]);
}

#[test]
fn api_platform() {
    test_repository("api-platform", "https://github.com/api-platform/core", &[]);
}

#[test]
fn joomla() {
    test_repository("joomla", "https://github.com/joomla/joomla-cms", &[]);
}

#[test]
fn prestashop() {
    test_repository(
        "prestashop",
        "https://github.com/PrestaShop/PrestaShop",
        &[],
    );
}

#[test]
fn sylius() {
    test_repository("sylius", "https://github.com/Sylius/Sylius", &[]);
}

#[test]
fn doctrine_orm() {
    test_repository("doctrine-orm", "https://github.com/doctrine/orm", &[]);
}

#[test]
fn doctrine_dbal() {
    test_repository(
        "doctrine-dbal",
        "https://github.com/doctrine/dbal",
        &[
            "vendor/jetbrains/phpstorm-stubs/Core/Core_c.php",
            "vendor/jetbrains/phpstorm-stubs/eio/eio.php",
            "vendor/jetbrains/phpstorm-stubs/event/event.php",
        ],
    );
}

fn test_repository(name: &str, repository: &str, ignore: &[&str]) {
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
            .arg(repository)
            .arg(&out_path)
            .output()
            .expect("failed to  run git.");

        if !output.status.success() {
            panic!("failed to clone repository: {:#?}", output)
        }
    }

    let composer_json = out_path.join("composer.json");
    let autoload = out_path.join("vendor").join("autoload.php");

    if composer_json.exists() && !autoload.exists() {
        let output = Command::new("composer")
            .arg("update")
            .arg("--ignore-platform-reqs")
            .arg("--no-plugins")
            .current_dir(&out_path)
            .output()
            .expect("failed to run composer");

        if !output.status.success() {
            panic!(
                "failed to run composer install in repository: {:#?}",
                output
            )
        }
    }

    let entries = read_directory(out_path.clone(), out_path, ignore);

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
                            Ok(_) => {
                                results.push(TestResult::Success);
                            }
                            Err(error) => {
                                results.push(TestResult::Error(format!(
                                    "❌ [{thread_name}][{name}]: {error} ({error:?})"
                                )));
                            }
                        },
                        Err(error) => {
                            results.push(TestResult::Error(format!(
                                "❌ [{thread_name}][{name}]: {error} ({error:?})"
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
