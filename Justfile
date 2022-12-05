default:
  @just --list

# build the library
build:
    cargo build

# build integration tests
build-integration-tests:
    BUILD_INTEGRATION_TESTS="1" cargo build

# regenerate test snapshots
snapshot:
    cargo run --bin snapshot

# detect linting problems.
lint:
    rm -f tests/integration_test.rs
    cargo fmt --all -- --check
    cargo clippy

# fix linting problems.
fix:
    rm -f tests/integration_test.rs
    cargo fmt
    cargo clippy --fix --allow-dirty --allow-staged

# dump AST for the given file.
dump file: build
    cargo run --bin php-parser-rs -- {{file}}

# run all integration tests, except third-party.
test filter='': build-integration-tests
    cargo test --all {{filter}} -- --skip third_party

# run integration tests for third-party libraries.
test-third-party: build
    cargo test third_party
