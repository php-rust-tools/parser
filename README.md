# PHP-Parser

A handwritten fault-tolerant, recursive-descent parser for PHP written in Rust.

[![justforfunnoreally.dev badge](https://img.shields.io/badge/justforfunnoreally-dev-9ff)](https://justforfunnoreally.dev)

> **Warning** - this is still alpha software and the public API is still subject to change. Please use at your own risk.

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
use std::io::Result;

use php_parser_rs::parser;

const CODE: &str = r#"<?php

final class User {
    public function __construct(
        public readonly string $name,
        public readonly string $email,
        public readonly string $password,
    ) {
    }
}
"#;

fn main() -> Result<()> {
    match parser::parse(CODE) {
        Ok(ast) => {
            println!("{:#?}", ast);
        }
        Err(err) => {
            println!("{}", err.report(CODE, None, true, false)?);

            println!("parsed so far: {:#?}", err.partial);
        }
    }

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
