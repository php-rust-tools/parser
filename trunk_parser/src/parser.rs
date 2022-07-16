use trunk_lexer::{Token, TokenKind};
use crate::{Program, Statement, Block};

macro_rules! expect {
    ($actual:expr, $expected:pat, $out:expr, $message:literal) => {
        match $actual {
            Some(token) => match token.kind {
                $expected => $out,
                _ => return Err(ParseError::ExpectedToken($message.into()))
            },
            None => return Err(ParseError::ExpectedToken($message.into()))
        }
    };
    ($actual:expr, $expected:pat, $message:literal) => {
        match $actual {
            Some(token) => match token.kind {
                $expected => (),
                _ => return Err(ParseError::ExpectedToken($message.into()))
            },
            None => return Err(ParseError::ExpectedToken($message.into()))
        }
    };
}

pub struct Parser {

}

impl Parser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse(&self, tokens: Vec<Token>) -> Result<Program, ParseError> {
        let mut program = Program::new();
        let mut iter = tokens.into_iter().peekable();

        while let Some(t) = iter.next() {
            match t.kind {
                TokenKind::OpenTag(_) => {},
                TokenKind::InlineHtml(html) => {
                    program.push(Statement::InlineHtml(html));
                },
                TokenKind::Function => {
                    let name = expect!(iter.next(), TokenKind::Identifier(i), i, "expected identifier");

                    expect!(iter.next(), TokenKind::LeftParen, "expected (");

                    let mut params = Vec::new();

                    while let Some(n) = iter.peek() && n.kind != TokenKind::RightParen {
                        // TODO: Support variable types and default values.
                        params.push(expect!(iter.next(), TokenKind::Variable(v), v, "expected variable").into());
                        
                        if let Some(Token { kind: TokenKind::Comma, .. }) = iter.peek() {
                            iter.next();
                        }
                    }

                    expect!(iter.next(), TokenKind::RightParen, "expected )");

                    // TODO: Support return types here.

                    expect!(iter.next(), TokenKind::LeftBrace, "expected {");

                    // TODO: Parse body here.

                    expect!(iter.next(), TokenKind::RightBrace, "expected }");

                    program.push(Statement::Function { name: name.into(), params, body: Block::new() });
                },
                _ => todo!("unhandled token: {:?}", t)
            }
        }

        Ok(program)
    }
}

#[derive(Debug)]
pub enum ParseError {
    ExpectedToken(String),
}

#[cfg(test)]
mod tests {
    use trunk_lexer::Lexer;
    use crate::{Statement, Block, Param};
    use super::Parser;

    macro_rules! function {
        ($name:literal, $params:expr, $body:expr) => {
            Statement::Function {
                name: $name.to_string().into(),
                params: $params.to_vec().into_iter().map(|p: &str| Param::from(p)).collect::<Vec<Param>>(),
                body: $body.to_vec(),
            }
        };
    }

    #[test]
    fn empty_fn() {
        assert_ast("<?php function foo() {}", &[
            function!("foo", &[], &[]),
        ]);
    }

    #[test]
    fn empty_fn_with_params() {
        assert_ast("<?php function foo($n) {}", &[
            function!("foo", &["n"], &[]),
        ]);

        assert_ast("<?php function foo($n, $m) {}", &[
            function!("foo", &["n", "m"], &[]),
        ]);
    }

    fn assert_ast(source: &str, expected: &[Statement]) {
        let mut lexer = Lexer::new(None);
        let tokens = lexer.tokenize(source).unwrap();

        let parser = Parser::new();
        let ast = parser.parse(tokens).unwrap();

        assert_eq!(ast, expected);
    }
}