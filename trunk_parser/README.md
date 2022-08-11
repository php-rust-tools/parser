# Trunk Parser

This crate provides a handwritten recursive descent parser targeting versions of PHP >=8.0.

It produces an abstract syntax tree containing `Statement` and `Expression` types which can be traversed to analyse code, compile into a lower level language, or reconstructed back into PHP code.

Alongside the regular PHP mode, there will be an additional Trunk compatibility mode that introduces some extra niceties to help support the development of the Trunk project. This will include things such as:
* Variable type declarations using a familiar `var [type] $name` syntax.
* Optionally enforce type declarations where appropriate (class properties, function parameters, return types).

## Usage

```rust
use trunk_lexer::*;
use trunk_parser::*;

let mut lexer = Lexer::new(None);
let tokens = lexer.tokenize(&source_code[..]).unwrap();

let mut parser = Parser::new(None);
let ast = parser.parse(tokens).unwrap();
```

The resulting `ast` is a `Vec<trunk_parser::Statement>` and can easily be iterated or converted into a dedicated iterator type.