use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;

use php_parser_rs::lexer::Lexer;

enum TestResult {
    Success,
    Error(String),
}

#[test]
fn php_standard_library() {
    test_repository(Repository::new(
        "php-standard-library",
        "https://github.com/azjezz/psl.git",
        vec![],
    ));
}

#[test]
fn laravel_framework() {
    test_repository(Repository::new(
        "laravel-framework",
        "https://github.com/laravel/framework",
        vec!["tests/Foundation/fixtures/bad-syntax-strategy.php"],
    ));
}

#[test]
fn hyperf_framework() {
    test_repository(Repository::new(
        "hyperf-framework",
        "https://github.com/hyperf/hyperf",
        vec![
            // files are Hack, not PHP.
            "vendor/nikic/fast-route/test/HackTypechecker/fixtures/all_options.php",
            "vendor/nikic/fast-route/test/HackTypechecker/fixtures/empty_options.php",
            "vendor/nikic/fast-route/test/HackTypechecker/fixtures/no_options.php",
        ],
    ));
}

#[test]
fn symfony_framework() {
    test_repository(Repository::new(
        "symfony-framework",
        "https://github.com/symfony/symfony",
        vec![
            // stub
            "src/Symfony/Bridge/ProxyManager/Tests/LazyProxy/PhpDumper/Fixtures/proxy-implem.php",
            // file contains syntax error used for testing.
            "src/Symfony/Component/Config/Tests/Fixtures/ParseError.php",
            // file contains unintentional error upstream, waiting for fix.
            "src/Symfony/Component/VarExporter/LazyProxyTrait.php",
            // this file contains XML, not PHP.
            "src/Symfony/Component/DependencyInjection/Tests/Fixtures/xml/xml_with_wrong_ext.php",
        ],
    ));
}

#[test]
fn nikic_php_parser() {
    test_repository(Repository::new(
        "nikic/PHP-Parser",
        "https://github.com/nikic/PHP-Parser",
        vec![
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
    ));
}

#[test]
fn yii_framework() {
    test_repository(Repository::new(
        "yii-framework",
        "https://github.com/yiisoft/yii2",
        vec![
            // Outdated dependency in Yii has an actual syntax error, nothing wrong with parser.
            "vendor/sebastian/diff/tests/ParserTest.php",
        ],
    ));
}

#[test]
fn spiral_framework() {
    test_repository(Repository::new(
        "spiral-framework",
        "https://github.com/spiral/framework",
        vec![
            // file contains syntax error used for testing.
            "src/Core/tests/Fixtures/CorruptedClass.php",
            "src/Tokenizer/tests/Classes/BrokenClass.php",
        ],
    ));
}

#[test]
fn mezzio_framework() {
    test_repository(Repository::new(
        "mezzio-framework",
        "https://github.com/mezzio/mezzio",
        vec![
            // files are Hack, not PHP.
            "vendor/nikic/fast-route/test/HackTypechecker/fixtures/all_options.php",
            "vendor/nikic/fast-route/test/HackTypechecker/fixtures/empty_options.php",
            "vendor/nikic/fast-route/test/HackTypechecker/fixtures/no_options.php",
        ],
    ));
}

#[test]
fn symfony_polyfill() {
    test_repository(Repository::new(
        "symfony-polyfill",
        "https://github.com/symfony/polyfill",
        vec![],
    ));
}

#[test]
fn madelineproto() {
    test_repository(Repository::new(
        "MadelineProto",
        "https://github.com/danog/MadelineProto",
        vec![],
    ));
}

#[test]
fn phabel() {
    test_repository(Repository::new(
        "phabel",
        "https://github.com/phabelio/phabel",
        vec![
            // Uses non-standard async/await syntax
            "tests/TargetFuture/AwaitTest.php",
        ],
    ));
}

#[test]
fn psalm() {
    test_repository(Repository::new(
        "psalm",
        "https://github.com/vimeo/psalm",
        vec![],
    ));
}

#[test]
fn phpstan_bin() {
    test_repository(Repository::new(
        "phpstan",
        "https://github.com/phpstan/phpstan",
        vec![],
    ));
}

#[test]
fn phpstan_src() {
    test_repository(Repository::new(
        "phpstan-src",
        "https://github.com/phpstan/phpstan-src",
        vec![
            // borken files used for testing.
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
            "tests/PHPStan/Analyser/data/bug-7135.php",
            "tests/PHPStan/Rules/Classes/data/first-class-instantiation-callable.php",
            "tests/PHPStan/Rules/Classes/data/instantiation-callable.php",
        ],
    ));
}

#[test]
fn rector_bin() {
    test_repository(Repository::new(
        "rector",
        "https://github.com/rectorphp/rector",
        vec![
            // uses PHP 7 $foo{$x}
            "e2e/parse-php7-code/src/Foo.php",
        ],
    ));
}

#[test]
fn rector_src() {
    test_repository(Repository::new(
        "rector-src",
        "https://github.com/rectorphp/rector-src",
        vec![
            // uses PHP 7 $foo{$x}
            "build/target-repository/e2e/parse-php7-code/src/Foo.php",
        ],
    ));
}

#[test]
fn composer() {
    test_repository(Repository::new(
        "composer",
        "https://github.com/composer/composer",
        vec![],
    ));
}

#[test]
fn wordpress() {
    test_repository(Repository::new(
        "wordpress",
        "https://github.com/WordPress/WordPress",
        vec![],
    ));
}

#[test]
fn chubbyphp_framework() {
    test_repository(Repository::new(
        "chubbyphp-framework",
        "https://github.com/chubbyphp/chubbyphp-framework",
        vec![],
    ));
}

#[test]
fn silverstripe_framework_core() {
    test_repository(Repository::new(
        "silverstripe-framework",
        "https://github.com/silverstripe/silverstripe-framework",
        vec![],
    ));
}

#[test]
fn silverstripe_framework_cms() {
    test_repository(Repository::new(
        "silverstripe-cms",
        "https://github.com/silverstripe/silverstripe-cms",
        vec![],
    ));
}

#[test]
fn roundcubemail_web() {
    test_repository(Repository::new(
        "roundcubemail",
        "https://github.com/roundcube/roundcubemail",
        vec![],
    ));
}

#[test]
fn phpmyadmin_web() {
    test_repository(Repository::new(
        "phpmyadmin",
        "https://github.com/phpmyadmin/phpmyadmin",
        vec![
            // files are Hack, not PHP.
            "vendor/nikic/fast-route/test/HackTypechecker/fixtures/all_options.php",
            "vendor/nikic/fast-route/test/HackTypechecker/fixtures/empty_options.php",
            "vendor/nikic/fast-route/test/HackTypechecker/fixtures/no_options.php",
        ],
    ));
}

#[test]
fn phpbb() {
    test_repository(Repository::new(
        "phpbb",
        "https://github.com/phpbb/phpbb",
        vec![],
    ));
}

#[test]
fn drupal() {
    test_repository(Repository::new(
        "drupal",
        "https://github.com/drupal/core",
        vec![],
    ));
}

#[test]
fn api_platform() {
    test_repository(Repository::new(
        "api-platform",
        "https://github.com/api-platform/core",
        vec![
            "vendor/doctrine/mongodb-odm/lib/Doctrine/ODM/MongoDB/Aggregation/Stage/GraphLookup/Match.php",
            "vendor/doctrine/mongodb-odm/lib/Doctrine/ODM/MongoDB/Aggregation/Stage/Match.php",
        ],
    ));
}

#[test]
fn joomla() {
    test_repository(Repository::new(
        "joomla",
        "https://github.com/joomla/joomla-cms",
        vec![
            // uses PHP 7 curly brackets array/string access
            "libraries/vendor/hoa/console/Chrome/Text.php",
            "libraries/vendor/phpunit/php-code-coverage/tests/_files/Crash.php",
        ],
    ));
}

#[test]
fn prestashop() {
    test_repository(Repository::new(
        "prestashop",
        "https://github.com/PrestaShop/PrestaShop",
        vec![
            // uses PHP 7 curly brackets array/string access
            "vendor/marcusschwarz/lesserphp/lessify.inc.php",
            "vendor/greenlion/php-sql-parser/libs/codesniffer/PhOSCo/Sniffs/Commenting/FileCommentSniff.php",
            // broken file used for testing
            "vendor/phpunit/php-code-coverage/tests/_files/Crash.php"
        ],
    ));
}

#[test]
fn sylius() {
    test_repository(Repository::new(
        "sylius",
        "https://github.com/Sylius/Sylius",
        vec![
            // broken file used for testing
            "vendor/phpunit/php-code-coverage/tests/_files/Crash.php",
        ],
    ));
}

#[test]
fn doctrine_orm() {
    test_repository(Repository::new(
        "doctrine-orm",
        "https://github.com/doctrine/orm",
        vec![],
    ));
}

#[test]
fn doctrine_dbal() {
    test_repository(Repository::new(
        "doctrine-dbal",
        "https://github.com/doctrine/dbal",
        vec![],
    ));
}

#[test]
fn phpunit() {
    test_repository(Repository::new(
        "phpunit",
        "https://github.com/sebastianbergmann/phpunit",
        vec![],
    ));
}

struct Repository<'a> {
    name: &'a str,
    repository: &'a str,
    ignore: Vec<&'a str>,
}

impl<'a> Repository<'a> {
    fn new(name: &'a str, repository: &'a str, ignore: Vec<&'a str>) -> Self {
        Self {
            name,
            repository,
            ignore,
        }
    }

    fn entries(&self) -> Vec<(String, PathBuf)> {
        let out_dir = self.create_output_dir();
        let out_path = self.clone_repository(&out_dir);
        self.install_composer(&out_path);
        self.read_directory(out_path.clone(), out_path)
    }

    fn create_output_dir(&self) -> PathBuf {
        let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let out_dir = manifest.join("target/third-party");
        if !out_dir.exists() {
            fs::create_dir(&out_dir).unwrap();
        }
        out_dir
    }

    fn clone_repository(&self, out_dir: &Path) -> PathBuf {
        let out_path = out_dir.join(self.name);

        if !out_path.exists() {
            let output = Command::new("git")
                .arg("clone")
                .args(["--depth", "1"])
                .arg(self.repository)
                .arg(&out_path)
                .output()
                .expect("failed to  run git.");

            if !output.status.success() {
                panic!("failed to clone repository: {:#?}", output)
            }
        }

        out_path
    }

    fn install_composer(&self, out_path: &Path) {
        let composer_json = out_path.join("composer.json");
        let autoload = out_path.join("vendor/autoload.php");

        if composer_json.exists() && !autoload.exists() {
            let output = Command::new("composer")
                .arg("install")
                .arg("--ignore-platform-reqs")
                .arg("--no-plugins")
                .arg("--no-scripts")
                .arg("--no-interaction")
                .arg("--prefer-dist")
                .current_dir(out_path)
                .output()
                .expect("failed to run composer");

            if !output.status.success() {
                panic!(
                    "failed to run composer install in repository: {:#?}",
                    output
                );
            }
        }
    }

    fn read_directory(&self, root: PathBuf, directory: PathBuf) -> Vec<(String, PathBuf)> {
        let mut results = vec![];
        let mut entries = fs::read_dir(directory)
            .unwrap()
            .flatten()
            .map(|entry| entry.path())
            .collect::<Vec<PathBuf>>();

        entries.sort();

        let ignored_prefixes = [
            "vendor/symfony",
            "vendor/doctrine/orm",
            "vendor/doctrine/dbal",
            "vendor/api-platform/core",
            "vendor/rector/rector",
            "vendor/phpstan/phpstan",
            "vendor/phpstan/php-8-stubs",
            "vendor/jetbrains/phpstorm-stubs",
        ];

        for entry in entries {
            let path = &entry
                .as_path()
                .strip_prefix(&root)
                .unwrap()
                .to_str()
                .unwrap();

            if self.name != "php-standard-library"
                && ignored_prefixes.iter().any(|p| path.starts_with(*p))
            {
                continue;
            }

            if entry.is_dir() {
                results.append(&mut self.read_directory(root.clone(), entry));

                continue;
            }

            if entry.extension().unwrap_or_default() == "php" && !self.ignore.contains(path) {
                let fullname_string = &entry.to_string_lossy();
                let name = fullname_string
                    .strip_prefix(root.to_str().unwrap())
                    .unwrap();

                results.push((name.to_string(), entry));
            }
        }

        results
    }
}

fn test_repository(repository: Repository) {
    let entries = repository.entries();

    let thread = thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .name(repository.name.to_string())
        .spawn(move || {
            let thread = thread::current();
            let thread_name = thread.name().unwrap();

            let mut results = vec![];
            for (name, filename) in entries {
                let code = fs::read(filename).unwrap();

                match Lexer::new().tokenize(&code) {
                    Ok(tokens) => match php_parser_rs::construct(&tokens) {
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
