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
