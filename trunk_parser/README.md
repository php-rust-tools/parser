<h3 align="center">
    Trunk Parser
</h3>

<p align="center">
    A handwritten recursive-descent parser for PHP code.
</p>

---

#### Overview

The parser produces an abstract syntax tree containing `Statement` and `Expression` types describing the PHP code provided.

#### Usage

```rust
use trunk_lexer::*;
use trunk_parser::*;

let mut lexer = Lexer::new(None);
let tokens = lexer.tokenize(&source_code[..]).unwrap();

let mut parser = Parser::new(None);
let ast = parser.parse(tokens).unwrap();
```

The resulting `ast` is a `Vec<trunk_parser::Statement>` and can easily be iterated or converted into a dedicated iterator type.