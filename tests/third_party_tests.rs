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
            // Auto-generated parsers with syntax mistakes
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
    test_repository(
        "yii-framework",
        "https://github.com/yiisoft/yii2",
        &[
            // Outdated dependency in Yii has an actual syntax error, nothing wrong with parser.
            "vendor/sebastian/diff/tests/ParserTest.php",
        ],
    );
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
fn phabel() {
    test_repository(
        "phabel",
        "https://github.com/phabelio/phabel",
        &[
            // Uses non-standard async/await syntax
            "tests/TargetFuture/AwaitTest.php",
        ],
    );
}

#[test]
fn psalm() {
    test_repository("psalm", "https://github.com/vimeo/psalm", &[]);
}

#[test]
fn phpstan_bin() {
    test_repository("phpstan", "https://github.com/phpstan/phpstan", &[]);
}

#[test]
fn phpstan_src() {
    test_repository(
        "phpstan-src",
        "https://github.com/phpstan/phpstan-src",
        &[
            "tests/PHPStan/Rules/Classes/data/invalid-promoted-properties.php",
            "tests/PHPStan/Rules/Classes/data/trait-use-error.php",
            "tests/PHPStan/Analyser/data/multipleParseErrors.php",
            "tests/PHPStan/Analyser/data/parse-error.php",
            "tests/PHPStan/Levels/data/namedArguments.php",
            "tests/PHPStan/Rules/Classes/data/enum-sanity.php",
            "tests/PHPStan/Rules/Classes/data/instanceof.php",
            "tests/PHPStan/Rules/Functions/data/arrow-function-intersection-types.php",
            "tests/PHPStan/Rules/Functions/data/closure-intersection-types.php",
            "tests/PHPStan/Rules/Functions/data/intersection-types.php",
            "tests/PHPStan/Rules/Methods/data/abstract-method.php",
            "tests/PHPStan/Rules/Methods/data/call-method-in-enum.php",
            "tests/PHPStan/Rules/Methods/data/intersection-types.php",
            "tests/PHPStan/Rules/Methods/data/missing-method-impl.php",
            "tests/PHPStan/Rules/Methods/data/named-arguments.php",
            "tests/PHPStan/Rules/Methods/data/named-arguments.php",
            "tests/PHPStan/Rules/Functions/data/closure-typehints.php",
            "tests/PHPStan/Rules/Properties/data/intersection-types.php",
            "tests/PHPStan/Rules/Properties/data/read-only-property-phpdoc-and-native.php",
            "tests/PHPStan/Rules/Properties/data/read-only-property.php",
        ],
    );
}

#[test]
fn rector_bin() {
    test_repository(
        "rector",
        "https://github.com/rectorphp/rector",
        &[
            // uses PHP 7 $foo{$x}
            "e2e/parse-php7-code/src/Foo.php",
        ],
    );
}

#[test]
fn rector_src() {
    test_repository(
        "rector-src",
        "https://github.com/rectorphp/rector-src",
        &[
            // uses PHP 7 $foo{$x}
            "build/target-repository/e2e/parse-php7-code/src/Foo.php",
        ],
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
    test_repository("api-platform", "https://github.com/api-platform/core", &[
        "vendor/doctrine/mongodb-odm/lib/Doctrine/ODM/MongoDB/Aggregation/Stage/GraphLookup/Match.php",
        "vendor/doctrine/mongodb-odm/lib/Doctrine/ODM/MongoDB/Aggregation/Stage/Match.php",
    ]);
}

#[test]
fn joomla() {
    test_repository(
        "joomla",
        "https://github.com/joomla/joomla-cms",
        &[
            // uses PHP 7 curly brackets array/string access
            "libraries/vendor/hoa/console/Chrome/Text.php",
            "libraries/vendor/phpunit/php-code-coverage/tests/_files/Crash.php",
        ],
    );
}

#[test]
fn prestashop() {
    test_repository(
        "prestashop",
        "https://github.com/PrestaShop/PrestaShop",
        &[
            // uses PHP 7 curly brackets array/string access
            "vendor/marcusschwarz/lesserphp/lessify.inc.php",
            "vendor/greenlion/php-sql-parser/libs/codesniffer/PhOSCo/Sniffs/Commenting/FileCommentSniff.php",
            // broken file used for testing
            "vendor/phpunit/php-code-coverage/tests/_files/Crash.php"
        ],
    );
}

#[test]
fn sylius() {
    test_repository(
        "sylius",
        "https://github.com/Sylius/Sylius",
        &[
            // broken file used for testing
            "vendor/phpunit/php-code-coverage/tests/_files/Crash.php",
        ],
    );
}

#[test]
fn doctrine_orm() {
    test_repository("doctrine-orm", "https://github.com/doctrine/orm", &[]);
}

#[test]
fn doctrine_dbal() {
    test_repository("doctrine-dbal", "https://github.com/doctrine/dbal", &[]);
}

#[test]
fn phpunit() {
    test_repository(
        "phpunit",
        "https://github.com/sebastianbergmann/phpunit",
        &[],
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
            .arg("install")
            .arg("--ignore-platform-reqs")
            .arg("--no-plugins")
            .arg("--no-scripts")
            .arg("--no-interaction")
            .arg("--prefer-dist")
            .current_dir(&out_path)
            .output()
            .expect("failed to run composer");

        if !output.status.success() {
            panic!(
                "failed to run composer install in repository: {:#?}",
                output
            );
        }
    }

    let entries = read_directory(out_path.clone(), out_path, ignore);

    let thread = thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .name(name.to_string())
        .spawn(move || {
            let thread = thread::current();
            let thread_name = thread.name().unwrap();

            let mut results = vec![];
            for (name, filename) in entries {
                let code = std::fs::read(&filename).unwrap();

                match Lexer::new().tokenize(&code) {
                    Ok(tokens) => match php_parser_rs::parse(&tokens) {
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

    let result = thread
        .unwrap_or_else(|e| panic!("failed to spawn thread: {:#?}", e))
        .join()
        .unwrap_or_else(|e| panic!("failed to join thread: {:#?}", e));

    let mut fail = false;
    result
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
        let path = &entry
            .as_path()
            .strip_prefix(&root)
            .unwrap()
            .to_str()
            .unwrap();

        if path.starts_with("vendor/symfony")
            || path.starts_with("vendor/doctrine/orm")
            || path.starts_with("vendor/doctrine/dbal")
            || path.starts_with("vendor/api-platform/core")
            || path.starts_with("vendor/rector/rector")
            || path.starts_with("vendor/phpstan/phpstan")
            || path.starts_with("vendor/phpstan/php-8-stubs")
            || path.starts_with("vendor/jetbrains/phpstorm-stubs")
        {
            continue;
        }

        if entry.is_dir() {
            results.append(&mut read_directory(root.clone(), entry, ignore));

            continue;
        }

        if entry.is_file()
            && entry.extension().unwrap_or_default() == "php"
            && !ignore.contains(path)
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
