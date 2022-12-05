# If the first argument is "dump"...
ifeq (dump,$(firstword $(MAKECMDGOALS)))
  # use the rest as arguments for "dump"
  RUN_ARGS := $(wordlist 2,$(words $(MAKECMDGOALS)),$(MAKECMDGOALS))
  # ...and turn them into do-nothing targets
  $(eval $(RUN_ARGS):;@:)
endif# If the first argument is "dump"...


.PHONY: help

help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

lint: ## lint code for formatting issues.
	cargo fmt --all -- --check
	cargo clippy

fix: ## fix linting problems.
	cargo fmt
	cargo clippy --fix --allow-dirty --allow-staged

dump: ## dump AST for given files.
	cargo run --bin php-parser-rs -- $(RUN_ARGS)

snapshot: ## dump a snapshot for intergration tests.
	cargo run --bin snapshot

test: ## run integration tests, use filter="..." argument to filter out a specific test.
	BUILD_INTEGRATION_TESTS="1" cargo build
	cargo test --all $(filter) -- --skip third_party

test-third-party: ## run integration tests against third-party libraries.
	BUILD_INTEGRATION_TESTS="1" cargo build
	cargo test third_party
