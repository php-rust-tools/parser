# Trunk Compiler

This package contains the initial MVP for compiling PHP code to a native executable.

The initial proof of concept is powered by the `nikic/php-parser` package since it supports the entire PHP language. The compiler itself will be moved across to the Rust one found in [`trunk_parser`](../trunk_parser/README.md) once the concept has been thought out and proved to be useful.

## Requirements

The compiler is written in PHP, so you will need the official [PHP](//php.net) engine available. You will also need to have a recent version of [Go](//go.dev) installed (recommended >1.18).