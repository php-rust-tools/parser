use std::{vec::IntoIter, fmt::Display};
use trunk_lexer::{Token, TokenKind, Span};
use crate::{Program, Statement, Block, Expression, ast::{MethodFlag, ArrayItem}, Identifier};

macro_rules! expect {
    ($parser:expr, $expected:pat, $out:expr, $message:literal) => {
        match $parser.current.kind.clone() {
            $expected => {
                $parser.next();
                $out
            },
            _ => return Err(ParseError::ExpectedToken($message.into(), $parser.current.span)),
        }
    };
    ($parser:expr, $expected:pat, $message:literal) => {
        match $parser.current.kind.clone() {
            $expected => { $parser.next(); },
            _ => return Err(ParseError::ExpectedToken($message.into(), $parser.current.span)),
        }
    };
}

pub struct Parser {
    pub current: Token,
    pub peek: Token,
    iter: IntoIter<Token>,
}

#[allow(dead_code)]
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut this = Self {
            current: Token::default(),
            peek: Token::default(),
            iter: tokens.into_iter(),
        };

        this.next();
        this.next();
        this
    }

    fn statement(&mut self) -> Result<Statement, ParseError> {
        Ok(match &self.current.kind {
            TokenKind::InlineHtml(html) => {
                let s = Statement::InlineHtml(html.to_string());
                self.next();
                s
            },
            TokenKind::If => {
                self.next();

                expect!(self, TokenKind::LeftParen, "expected (");

                let condition = self.expression(0)?;

                expect!(self, TokenKind::RightParen, "expected )");

                // TODO: Support one-liner if statements.
                expect!(self, TokenKind::LeftBrace, "expected {");

                let mut then = Block::new();
                while ! self.is_eof() && self.current.kind != TokenKind::RightBrace {
                    then.push(self.statement()?);
                }

                // TODO: Support one-liner if statements.
                expect!(self, TokenKind::RightBrace, "expected }");

                Statement::If { condition, then }
            },
            TokenKind::Class => self.class()?,
            TokenKind::Echo => {
                self.next();

                let mut values = Vec::new();
                while ! self.is_eof() && self.current.kind != TokenKind::SemiColon {
                    values.push(self.expression(0)?);

                    // `echo` supports multiple expressions separated by a comma.
                    // TODO: Disallow trailing commas when the next token is a semi-colon.
                    if ! self.is_eof() && self.current.kind == TokenKind::Comma {
                        self.next();
                    }
                }
                expect!(self, TokenKind::SemiColon, "expected semi-colon at the end of an echo statement");
                Statement::Echo { values }
            },
            TokenKind::Return => {
                self.next();

                if let Token { kind: TokenKind::SemiColon, .. } = self.current {
                    let ret = Statement::Return { value: None };
                    expect!(self, TokenKind::SemiColon, "expected semi-colon at the end of return statement.");
                    ret
                } else {
                    let ret = Statement::Return { value: self.expression(0).ok() };
                    expect!(self, TokenKind::SemiColon, "expected semi-colon at the end of return statement.");
                    ret
                }
            },
            TokenKind::Function => {
                self.next();

                let name = expect!(self, TokenKind::Identifier(i), i, "expected identifier");

                expect!(self, TokenKind::LeftParen, "expected (");

                let mut params = Vec::new();

                while ! self.is_eof() && self.current.kind != TokenKind::RightParen {
                    // TODO: Support variable types and default values.
                    params.push(expect!(self, TokenKind::Variable(v), v, "expected variable").into());
                    
                    if let Token { kind: TokenKind::Comma, .. } = self.current {
                        self.next();
                    }
                }

                expect!(self, TokenKind::RightParen, "expected )");

                // TODO: Support return types here.

                expect!(self, TokenKind::LeftBrace, "expected {");

                let mut body = Block::new();

                while ! self.is_eof() && self.current.kind != TokenKind::RightBrace {
                    body.push(self.statement()?);
                }

                expect!(self, TokenKind::RightBrace, "expected }");

                Statement::Function { name: name.into(), params, body }
            },
            TokenKind::Var => {
                self.next();

                let var = expect!(self, TokenKind::Variable(i), i, "expected variable name");

                expect!(self, TokenKind::SemiColon, "expected semi-colon");

                Statement::Var { var }
            },
            TokenKind::SemiColon => {
                self.next();

                Statement::Noop
            },
            _ => {
                let expr = self.expression(0)?;

                expect!(self, TokenKind::SemiColon, "expected semi-colon");

                Statement::Expression { expr }
            }
        })
    }

    fn class(&mut self) -> Result<Statement, ParseError> {
        self.next();

        let name = expect!(self, TokenKind::Identifier(i), i, "expected class name");
        let mut extends: Option<Identifier> = None;

        if self.current.kind == TokenKind::Extends {
            self.next();
            extends = expect!(self, TokenKind::Identifier(i), Some(i.into()), "expected identifier");
        }

        let mut implements = Vec::new();
        if self.current.kind == TokenKind::Implements {
            self.next();

            while self.current.kind != TokenKind::LeftBrace {
                if self.current.kind == TokenKind::Comma {
                    self.next();
                }

                implements.push(expect!(self, TokenKind::Identifier(i), i.into(), "expected identifier"));
            }
        }

        expect!(self, TokenKind::LeftBrace, "expected left-brace");

        let mut body = Vec::new();
        while ! self.is_eof() && self.current.kind != TokenKind::RightBrace {
            let statement = match self.statement()? {
                Statement::Function { name, params, body } => {
                    Statement::Method { name, params, body, flags: vec![] }
                },
                Statement::Var { var } => {
                    Statement::Property { var }
                },
                s @ Statement::Method { .. } => s,
                _ => return Err(ParseError::InvalidClassStatement("Classes can only contain properties, constants and methods.".to_string(), self.current.span))
            };

            body.push(statement);
        }

        expect!(self, TokenKind::RightBrace, "expected right-brace");

        Ok(Statement::Class { name: name.into(), extends, implements, body, flag: None })
    }
    
    fn expression(&mut self, bp: u8) -> Result<Expression, ParseError> {
        if self.is_eof() {
            return Err(ParseError::UnexpectedEndOfFile);
        }

        let mut lhs = match &self.current.kind {
            TokenKind::Variable(v) => Expression::Variable(v.to_string()),
            TokenKind::Int(i) => Expression::Int(*i),
            TokenKind::Identifier(i) => Expression::Identifier(i.to_string()),
            TokenKind::LeftBracket => {
                let mut items = Vec::new();
                self.next();

                while self.current.kind != TokenKind::RightBracket {
                    let mut key = None;
                    let mut value = self.expression(0)?;

                    if self.current.kind == TokenKind::DoubleArrow {
                        self.next();

                        key = Some(value);
                        value = self.expression(0)?;
                    }

                    items.push(ArrayItem { key, value });

                    if self.current.kind == TokenKind::Comma {
                        self.next();
                    }
                }

                Expression::Array(items)
            },
            _ => todo!("expr lhs: {:?}", self.current.kind),
        };

        self.next();

        loop {
            let kind = match &self.current {
                Token { kind: TokenKind::SemiColon | TokenKind::Eof, .. }  => break,
                Token { kind, .. } => kind.clone()
            };

            if let Some(lbp) = postfix_binding_power(&kind) {
                if lbp < bp {
                    break;
                }

                self.next();

                let op = kind.clone();
                lhs = self.postfix(lhs, &op)?;

                continue;
            }

            if let Some((lbp, rbp)) = infix_binding_power(&kind) {
                if lbp < bp {
                    break;
                }

                self.next();

                let op = kind.clone();
                let rhs = self.expression(rbp)?;

                lhs = infix(lhs, op, rhs);
                continue;
            }

            break;
        }

        Ok(lhs)
    }

    fn postfix(&mut self, lhs: Expression, op: &TokenKind) -> Result<Expression, ParseError> {
        Ok(match op {
            TokenKind::LeftParen => {
                let mut args = Vec::new();
                while ! self.is_eof() && self.current.kind != TokenKind::RightParen {
                    args.push(self.expression(0)?);

                    if let Token { kind: TokenKind::Comma, .. } = self.current {
                        self.next();
                    }
                }

                expect!(self, TokenKind::RightParen, "expected )");
    
                Expression::Call(Box::new(lhs), args)
            },
            _ => todo!("postfix: {:?}", op),
        })
    }

    fn is_eof(&self) -> bool {
        self.current.kind == TokenKind::Eof
    }

    pub fn next(&mut self) {
        self.current = self.peek.clone();
        self.peek = self.iter.next().unwrap_or_default()
    }

    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let mut ast = Program::new();

        while self.current.kind != TokenKind::Eof {
            if let TokenKind::OpenTag(_) = self.current.kind {
                self.next();
                continue;
            }

            ast.push(self.statement()?);
        }

        Ok(ast.to_vec())
    }
}

fn infix(lhs: Expression, op: TokenKind, rhs: Expression) -> Expression {
    if op == TokenKind::Equals {
        return Expression::Assign(Box::new(lhs), Box::new(rhs));
    }

    Expression::Infix(Box::new(lhs), op.into(), Box::new(rhs))
}

fn infix_binding_power(t: &TokenKind) -> Option<(u8, u8)> {
    Some(match t {
        TokenKind::Plus | TokenKind::Minus => (11, 12),
        TokenKind::LessThan => (9, 10),
        TokenKind::Equals => (2, 1),
        _ => return None,
    })
}

fn postfix_binding_power(t: &TokenKind) -> Option<u8> {
    Some(match t {
        TokenKind::LeftParen => 19,
        _ => return None
    })
}

#[derive(Debug)]
pub enum ParseError {
    ExpectedToken(String, Span),
    UnexpectedEndOfFile,
    InvalidClassStatement(String, Span),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExpectedToken(message, span) => write!(f, "Parse error: {} on line {} column {}", message, span.0, span.1),
            Self::InvalidClassStatement(message, span) => write!(f, "Parse error: {} on line {} column {}", message, span.0, span.1),
            Self::UnexpectedEndOfFile => write!(f, "Parse error: unexpected end of file.")
        }
    }
}

#[cfg(test)]
mod tests {
    use trunk_lexer::Lexer;
    use crate::{Statement, Param, Expression, ast::{InfixOp, MethodFlag}};
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

    macro_rules! class {
        ($name:literal) => {
            Statement::Class {
                name: $name.to_string().into(),
                body: vec![],
                extends: None,
                implements: vec![],
                flag: None,
            }
        };
        ($name:literal, $body:expr) => {
            Statement::Class {
                name: $name.to_string().into(),
                body: $body.to_vec(),
                extends: None,
                implements: vec![],
                flag: None,
            }
        };
        ($name:literal, $extends:expr, $implements:expr, $body:expr) => {
            Statement::Class {
                name: $name.to_string().into(),
                body: $body.to_vec(),
                extends: $extends,
                implements: $implements.to_vec(),
                flag: None,
            }
        };
    }

    macro_rules! method {
        ($name:literal, $params:expr, $flags:expr, $body:expr) => {
            Statement::Method {
                name: $name.to_string().into(),
                params: $params.to_vec().into_iter().map(|p: &str| Param::from(p)).collect::<Vec<Param>>(),
                flags: $flags.to_vec(),
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

    #[test]
    fn fib() {
        assert_ast("\
        <?php

        function fib($n) {
            if ($n < 2) {
                return $n;
            }

            return fib($n - 1) + fib($n - 2);
        }", &[
            function!("fib", &["n"], &[
                Statement::If {
                    condition: Expression::Infix(
                        Box::new(Expression::Variable("n".into())),
                        InfixOp::LessThan,
                        Box::new(Expression::Int(2)),
                    ),
                    then: vec![
                        Statement::Return { value: Some(Expression::Variable("n".into())) }
                    ],
                },
                Statement::Return {
                    value: Some(Expression::Infix(
                        Box::new(Expression::Call(
                            Box::new(Expression::Identifier("fib".into())),
                            vec![
                                Expression::Infix(
                                    Box::new(Expression::Variable("n".into())),
                                    InfixOp::Sub,
                                    Box::new(Expression::Int(1)),
                                )
                            ]
                        )),
                        InfixOp::Add,
                        Box::new(Expression::Call(
                            Box::new(Expression::Identifier("fib".into())),
                            vec![
                                Expression::Infix(
                                    Box::new(Expression::Variable("n".into())),
                                    InfixOp::Sub,
                                    Box::new(Expression::Int(2)),
                                )
                            ]
                        )),
                    ))
                }
            ])
        ]);
    }

    #[test]
    fn echo() {
        assert_ast("<?php echo 1;", &[
            Statement::Echo {
                values: vec![
                    Expression::Int(1),
                ]
            }
        ]);
    }

    #[test]
    fn empty_class() {
        assert_ast("<?php class Foo {}", &[
            class!("Foo")
        ]);
    }

    #[test]
    fn class_with_basic_method() {
        assert_ast("\
        <?php
        
        class Foo {
            function bar() {
                echo 1;
            }
        }
        ", &[
            class!("Foo", &[
                method!("bar", &[], &[], &[
                    Statement::Echo { values: vec![
                        Expression::Int(1),
                    ] }
                ])
            ])
        ]);
    }

    #[test]
    fn class_with_extends() {
        assert_ast("\
        <?php
        
        class Foo extends Bar {}
        ", &[
            class!("Foo", Some("Bar".to_string().into()), &[], &[]),
        ]);
    }
    
    #[test]
    fn class_with_implements() {
        assert_ast("\
        <?php
        
        class Foo implements Bar, Baz {}
        ", &[
            class!("Foo", None, &["Bar".to_string().into(), "Baz".to_string().into()], &[]),
        ]);
    }

    #[test]
    fn noop() {
        assert_ast("<?php ;", &[
            Statement::Noop,
        ]);
    }

    fn assert_ast(source: &str, expected: &[Statement]) {
        let mut lexer = Lexer::new(None);
        let tokens = lexer.tokenize(source).unwrap();

        let mut parser = Parser::new(tokens);
        let ast = parser.parse();

        if ast.is_err() {
            panic!("{}", ast.err().unwrap());
        } else {
            assert_eq!(ast.unwrap(), expected);
        }
    }
}