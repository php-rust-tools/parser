# PHP-Parser

A handwritten recursive-descent parser for PHP written in Rust, for fun.

<p align="center">
    <a href="https://justforfunnoreally.dev/">
        <img src="https://img.shields.io/badge/justforfunnoreally-dev-9ff">
    </a>
</p>


---

> **Warning**: This crate is not ready for any form of production use _yet_. There are still a lot of things missing from the parser, so please use at your own risk.

---

## Usage

Add `php-parser-rs` in your `Cargo.toml`'s `dependencies` section

```toml
[dependencies]
php-parser-rs = "0.0.0-b1"
```

### Example

```rust
use php_parser_rs::*;

let mut lexer = Lexer::new(None);
let tokens = lexer.tokenize(&source_code[..]).unwrap();

let mut parser = Parser::new(None);
let ast = parser.parse(tokens).unwrap();
```


## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

## Credits

* [Ryan Chandler](https://github.com/ryangjchandler)
* [All contributors](https://github.com/ryangjchandler/php-parser-rs/graphs/contributors)
