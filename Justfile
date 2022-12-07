default:
  @just --list

# build the library
build:
    cargo build

# regenerate test snapshots
snapshot:
    cargo run --bin snapshot

# detect linting problems.
lint:
    cargo fmt --all -- --check
    cargo clippy

# fix linting problems.
fix:
    cargo fmt
    cargo clippy --fix --allow-dirty --allow-staged

# dump AST for the given file.
dump file:
    cargo run --bin php-parser-rs -- {{file}}

# run all integration tests, except third-party.
test filter='':
    cargo test --all {{filter}} -- --skip third_party

# run integration tests for `azjezz/psl` library.
test-psl:
    cargo test php_standard_library -- --nocapture

# run integration tests for `nikic/php-parser` library.
test-php-parser:
    cargo test nikic_php_parser -- --nocapture

# run integration tests for `symfony/symfony` framework.
test-symfony:
    cargo test symfony_framework -- --nocapture

# run integration tests for `laravel/framework` framework.
test-laravel:
    cargo test laravel_framework -- --nocapture
