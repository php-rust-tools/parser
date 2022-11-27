<h3 align="center">
    php-parser-rs
</h3>

<p align="center">
    A handwritten recursive-descent parser for PHP written in Rust, for fun.
</p>

---

#### Usage

```rust
use php_parser_rs::*;

let mut lexer = Lexer::new(None);
let tokens = lexer.tokenize(&source_code[..]).unwrap();

let mut parser = Parser::new(None);
let ast = parser.parse(tokens).unwrap();
```

> **Warning**: This crate is not ready for any form of production use _yet_. There are still a lot of things missing from the parser, so please use at your own risk.

#### Contributing

All contributions to this repository are welcome. It's the perfect project for Rust beginners since we don't use many of Rust's complex features and the core concepts in the parser are purposely simple.

If you do wish to contribute, we just ask you to follow a few simple rules.

1. Create a pull request from a **non-main** branch on your fork.
2. Provide a short, but clear, description of your changes.
3. Have fun and don't take it all too seriously!

#### Credits

* [Ryan Chandler](https://github.com/ryangjchandler)
* [All contributors](https://github.com/ryangjchandler/php-parser-rs/graphs/contributors)