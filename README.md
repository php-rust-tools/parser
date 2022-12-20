# PHP-Parser

A handwritten recursive-descent parser for PHP written in Rust.

[![justforfunnoreally.dev badge](https://img.shields.io/badge/justforfunnoreally-dev-9ff)](https://justforfunnoreally.dev)

> **Warning**: This crate is not ready for any form of production use _yet_. There are still a lot of things missing from the parser, so please use at your own risk.

---

## Usage

Add `php-parser-rs` in your `Cargo.toml`'s `dependencies` section

```toml
[dependencies]
php-parser-rs = { git = "https://github.com/php-rust-tools/parser" }
```

or use `cargo add`

```sh
cargo add php-parser-rs --git https://github.com/php-rust-tools/parser
```

### Example

```rust
use php_parser_rs::parse;

fn main() -> ParseResult<()> {
    let code = "
<?php

function hello(): void {
    echo 'Hello, World!';
}

hello();
";

    let ast = parse(code.as_bytes())?;

    dbg!(ast);

    Ok(())
}
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
