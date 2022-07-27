use std::{vec::IntoIter, fmt::{Display}};
use trunk_lexer::{Token, TokenKind, Span};
use crate::{Program, Statement, Block, Expression, ast::{ArrayItem, Use, MethodFlag, ClassFlag, ElseIf, UseKind}, Identifier, Type};

type ParseResult<T> = Result<T, ParseError>;

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

mod params;
mod block;
mod punc;
mod ident;

#[derive(PartialEq)]
enum FunctionKind {
    Named,
    Closure,
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

    fn type_string(&mut self) -> ParseResult<Type> {
        if self.current.kind == TokenKind::Question {
            self.next();
            let t = expect!(self, TokenKind::Identifier(s) | TokenKind::QualifiedIdentifier(s) | TokenKind::FullyQualifiedIdentifier(s), s, "expected identifier");
            return Ok(Type::Nullable(t));
        }

        let id = expect!(self, TokenKind::Identifier(s) | TokenKind::QualifiedIdentifier(s) | TokenKind::FullyQualifiedIdentifier(s), s, "expected identifier");

        if self.current.kind == TokenKind::Pipe {
            self.next();

            let mut types = vec![id];

            while ! self.is_eof() {
                let id = expect!(self, TokenKind::Identifier(s) | TokenKind::QualifiedIdentifier(s) | TokenKind::FullyQualifiedIdentifier(s), s, "expected identifier");
                types.push(id);

                if self.current.kind != TokenKind::Pipe {
                    break;
                }
            }

            return Ok(Type::Union(types))
        }

        if self.current.kind == TokenKind::Ampersand {
            self.next();

            let mut types = vec![id];

            while ! self.is_eof() {
                let id = expect!(self, TokenKind::Identifier(s) | TokenKind::QualifiedIdentifier(s) | TokenKind::FullyQualifiedIdentifier(s), s, "expected identifier");
                types.push(id);

                if self.current.kind != TokenKind::Ampersand {
                    break;
                }
            }

            return Ok(Type::Intersection(types))
        }

        Ok(Type::Plain(id))
    }

    fn statement(&mut self) -> ParseResult<Statement> {
        Ok(match &self.current.kind {
            TokenKind::InlineHtml(html) => {
                let s = Statement::InlineHtml(html.to_string());
                self.next();
                s
            },
            TokenKind::Comment(comment) => {
                let s = Statement::Comment { comment: comment.to_string() };
                self.next();
                s
            },
            TokenKind::Abstract => {
                self.next();  

                match self.class()? {
                    Statement::Class { name, extends, implements, body, .. } => {
                        Statement::Class { name, extends, implements, body, flag: Some(ClassFlag::Abstract) }
                    },
                    _ => unreachable!(),
                }
            },
            TokenKind::Final => {
                self.next();  

                match self.class()? {
                    Statement::Class { name, extends, implements, body, .. } => {
                        Statement::Class { name, extends, implements, body, flag: Some(ClassFlag::Final) }
                    },
                    _ => unreachable!(),
                }
            },
            TokenKind::Trait => {
                self.next();

                let name = self.ident()?;

                self.lbrace()?;

                let mut body = Block::new();
                while self.current.kind != TokenKind::RightBrace {
                    match self.class_statement()? {
                        Statement::Constant { .. } => {
                            return Err(ParseError::TraitCannotContainConstant(self.current.span))
                        },
                        s => {
                            body.push(s);
                        },
                    }
                }

                self.rbrace()?;

                Statement::Trait { name: name.into(), body }
            },
            TokenKind::Interface => {
                self.next();

                let name = expect!(self, TokenKind::Identifier(n), n, "expected identifier");

                let mut extends = vec![];
                if self.current.kind == TokenKind::Extends {
                    self.next();

                    while self.current.kind != TokenKind::LeftBrace {
                        if self.current.kind == TokenKind::Comma {
                            self.next();
                        }

                        let e = expect!(self, TokenKind::Identifier(i) | TokenKind::QualifiedIdentifier(i) | TokenKind::FullyQualifiedIdentifier(i), i, "expected identifier");

                        extends.push(e.into());
                    }
                }

                self.lbrace()?;

                let mut body = Block::new();
                while self.current.kind != TokenKind::RightBrace {
                    match self.current.kind {
                        TokenKind::Public => {
                            self.next();

                            self.next();

                            let name = self.ident()?;

                            self.lparen()?;

                            let params = self.param_list()?;

                            self.rparen()?;

                            let mut return_type = None;

                            if self.current.kind == TokenKind::Colon {
                                self.next();

                                return_type = Some(self.type_string()?);
                            }

                            self.semi()?;

                            body.push(Statement::Method { name: name.into(), params, body: vec![], return_type, flags: vec![MethodFlag::Public] })
                        },
                        TokenKind::Function => {
                            self.next();

                            let name = self.ident()?;

                            self.lparen()?;

                            let params = self.param_list()?;

                            self.rparen()?;

                            let mut return_type = None;

                            if self.current.kind == TokenKind::Colon {
                                self.next();

                                return_type = Some(self.type_string()?);
                            }

                            self.semi()?;

                            body.push(Statement::Method { name: name.into(), params, body: vec![], return_type, flags: vec![] })
                        },
                        _ => return Err(ParseError::UnexpectedToken(self.current.kind.to_string(), self.current.span)),
                    }
                }

                self.rbrace()?;

                Statement::Interface { name: name.into(), extends, body }
            },
            TokenKind::Use => {
                self.next();

                let mut uses = Vec::new();
                while ! self.is_eof() {
                    let name = expect!(self, TokenKind::Identifier(i) | TokenKind::QualifiedIdentifier(i) | TokenKind::FullyQualifiedIdentifier(i), i, "expected identifier in use statement");
                    let mut alias = None;

                    if self.current.kind == TokenKind::As {
                        self.next();
                        alias = Some(self.ident()?.into());
                    }

                    uses.push(Use { name: name.into(), alias });

                    if self.current.kind == TokenKind::Comma {
                        self.next();
                        continue;
                    }

                    self.semi()?;
                    break;
                }

                Statement::Use { uses, kind: UseKind::Normal }
            },
            TokenKind::Switch => {
                self.next();

                self.lparen()?;

                let condition = self.expression(0)?;

                self.rparen()?;

                self.lbrace()?;
                self.rbrace()?;
                self.semi()?;

                Statement::Switch { condition }
            },
            TokenKind::Namespace => {
                self.next();
                
                let name = expect!(self, TokenKind::Identifier(i) | TokenKind::QualifiedIdentifier(i), i, "expected identifier after namespace");

                let mut braced = false;
                if self.current.kind == TokenKind::LeftBrace {
                    braced = true;
                    self.next();
                } else {
                    self.semi()?;
                }

                let body = if braced {
                    self.block(&TokenKind::RightBrace)?
                } else {
                    Block::new()
                };

                if braced {
                    self.rbrace()?;
                }

                Statement::Namespace { name, body }
            },
            TokenKind::If => {
                self.next();

                self.lparen()?;

                let condition = self.expression(0)?;

                self.rparen()?;

                let body_end_token = if self.current.kind == TokenKind::LeftBrace {
                    self.next();

                    TokenKind::RightBrace
                } else {
                    TokenKind::SemiColon
                };

                let then = self.block(&body_end_token)?;

                if body_end_token == TokenKind::RightBrace {
                    self.rbrace()?;
                }

                let mut else_ifs: Vec<ElseIf> = Vec::new();
                loop {
                    if self.current.kind == TokenKind::ElseIf {
                        self.next();

                        self.lparen()?;

                        let condition = self.expression(0)?;

                        self.rparen()?;

                        self.lbrace()?;

                        let body = self.block(&TokenKind::RightBrace)?;

                        self.rbrace()?;

                        else_ifs.push(ElseIf { condition, body });
                    } else {
                        break;
                    }
                }

                if self.current.kind != TokenKind::Else {
                    return Ok(Statement::If { condition, then, else_ifs, r#else: None });
                }

                expect!(self, TokenKind::Else, "expected else");

                self.lbrace()?;

                let r#else = self.block(&TokenKind::RightBrace)?;

                self.rbrace()?;

                Statement::If { condition, then, else_ifs, r#else: Some(r#else) }
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
                self.semi()?;
                Statement::Echo { values }
            },
            TokenKind::Continue => {
                self.next();

                let mut num = None;
                if self.current.kind != TokenKind::SemiColon {
                    num = Some(self.expression(0)?);
                }

                dbg!(&self.current.kind);

                self.semi()?;

                Statement::Continue { num }
            },
            TokenKind::Break => {
                self.next();

                let mut num = None;
                if self.current.kind != TokenKind::SemiColon {
                    num = Some(self.expression(0)?);
                }

                dbg!(&self.current.kind);

                self.semi()?;

                Statement::Break { num }
            },
            TokenKind::Return => {
                self.next();

                if let Token { kind: TokenKind::SemiColon, .. } = self.current {
                    let ret = Statement::Return { value: None };
                    self.semi()?;
                    ret
                } else {
                    let ret = Statement::Return { value: self.expression(0).ok() };
                    self.semi()?;
                    ret
                }
            },
            TokenKind::Function => self.function(FunctionKind::Named)?,
            TokenKind::SemiColon => {
                self.next();

                Statement::Noop
            },
            _ => {
                let expr = self.expression(0)?;

                self.semi()?;

                Statement::Expression { expr }
            }
        })
    }

    fn function(&mut self, kind: FunctionKind) -> ParseResult<Statement> {
        self.next();

        let name = if kind == FunctionKind::Named {
            self.ident()?
        } else {
            String::new()
        };

        self.lparen()?;

        let params = self.param_list()?;

        self.rparen()?;

        let mut return_type = None;

        if self.current.kind == TokenKind::Colon {
            self.next();

            return_type = Some(self.type_string()?);
        }

        self.lbrace()?;

        let body = self.block(&TokenKind::RightBrace)?;

        self.rbrace()?;

        Ok(Statement::Function { name: name.into(), params, body, return_type })
    }

    fn class(&mut self) -> ParseResult<Statement> {
        self.next();

        let name = self.ident()?;
        let mut extends: Option<Identifier> = None;

        if self.current.kind == TokenKind::Extends {
            self.next();
            extends = Some(self.ident()?.into());
        }

        let mut implements = Vec::new();
        if self.current.kind == TokenKind::Implements {
            self.next();

            while self.current.kind != TokenKind::LeftBrace {
                if self.current.kind == TokenKind::Comma {
                    self.next();
                }

                implements.push(self.ident()?.into());
            }
        }

        self.lbrace()?;

        let mut body = Vec::new();
        while self.current.kind != TokenKind::RightBrace && ! self.is_eof() {
            body.push(self.class_statement()?);
        }

        self.rbrace()?;

        Ok(Statement::Class { name: name.into(), extends, implements, body, flag: None })
    }
    
    fn class_statement(&mut self) -> ParseResult<Statement> {
        match self.current.kind {
            TokenKind::Use => {
                self.next();

                let mut traits = Vec::new();

                while self.current.kind != TokenKind::SemiColon {
                    if self.current.kind == TokenKind::Comma {
                        self.next();
                    }

                    let t = expect!(self, TokenKind::Identifier(i) | TokenKind::QualifiedIdentifier(i) | TokenKind::FullyQualifiedIdentifier(i), i, "expected identifier");
                    traits.push(t.into());
                }

                self.semi()?;

                Ok(Statement::TraitUse { traits })
            },
            TokenKind::Const => {
                self.next();

                let name = expect!(self, TokenKind::Identifier(c), c, "expected constant name");

                expect!(self, TokenKind::Equals, "expected =");

                let value = self.expression(0)?;

                self.semi()?;

                Ok(Statement::Constant { name: name.into(), value, flags: vec![] })
            },
            TokenKind::Var => {
                self.next();

                let mut var_type = None;

                if ! matches!(self.current.kind, TokenKind::Variable(_)) {
                    var_type = Some(self.type_string()?);
                }

                let var = expect!(self, TokenKind::Variable(i), i, "expected variable name");
                let mut value = None;

                if self.current.kind == TokenKind::Equals {
                    self.next();

                    value = Some(self.expression(0)?);
                }

                self.semi()?;

                Ok(Statement::Var { var, value, r#type: var_type })
            },
            TokenKind::Final | TokenKind::Abstract | TokenKind::Public | TokenKind::Private | TokenKind::Protected | TokenKind::Static => {
                let mut flags = vec![self.current.kind.clone()];
                self.next();

                while ! self.is_eof() && [TokenKind::Final, TokenKind::Abstract, TokenKind::Public, TokenKind::Private, TokenKind::Protected, TokenKind::Static].contains(&self.current.kind) {
                    if flags.contains(&self.current.kind) {
                        return Err(ParseError::UnexpectedToken(self.current.kind.to_string(), self.current.span));
                    }

                    flags.push(self.current.kind.clone());
                    self.next();
                }

                if flags.contains(&TokenKind::Final) && flags.contains(&TokenKind::Abstract) {
                    return Err(ParseError::InvalidAbstractFinalFlagCombination(self.current.span));
                }

                match self.current.kind {
                    TokenKind::Const => {
                        if flags.contains(&TokenKind::Static) {
                            return Err(ParseError::ConstantCannotBeStatic(self.current.span));
                        }

                        if flags.contains(&TokenKind::Final) && flags.contains(&TokenKind::Private) {
                            return Err(ParseError::ConstantCannotBePrivateFinal(self.current.span));
                        }

                        self.next();

                        let name = expect!(self, TokenKind::Identifier(c), c, "expected constant name");

                        expect!(self, TokenKind::Equals, "expected =");
        
                        let value = self.expression(0)?;
        
                        self.semi()?;
        
                        Ok(Statement::Constant { name: name.into(), value, flags: flags.into_iter().map(|f| f.into()).collect() })
                    },
                    TokenKind::Function => {
                        if flags.contains(&TokenKind::Abstract) {
                            self.next();

                            let name = self.ident()?;

                            self.lparen()?;

                            let params = self.param_list()?;

                            self.rparen()?;

                            let mut return_type = None;

                            if self.current.kind == TokenKind::Colon {
                                self.next();

                                return_type = Some(self.type_string()?);
                            }

                            self.semi()?;

                            Ok(Statement::Method { name: name.into(), params, body: vec![], return_type, flags: flags.iter().map(|t| t.clone().into()).collect() })
                        } else {
                            match self.function(FunctionKind::Named)? {
                                Statement::Function { name, params, body, return_type } => {
                                    Ok(Statement::Method { name, params, body, flags: flags.iter().map(|t| t.clone().into()).collect(), return_type })
                                },
                                _ => unreachable!()
                            }
                        }
                    },
                    TokenKind::Question | TokenKind::Identifier(_) | TokenKind::QualifiedIdentifier(_) | TokenKind::FullyQualifiedIdentifier(_) => {
                        let prop_type = self.type_string()?;
                        let var = expect!(self, TokenKind::Variable(v), v, "expected variable name");
                        let mut value = None;

                        if self.current.kind == TokenKind::Equals {
                            self.next();
                            value = Some(self.expression(0)?);
                        }

                        // TODO: Support comma-separated property declarations.
                        //       nikic/php-parser does this with a single Property statement
                        //       that is capable of holding multiple property declarations.
                        self.semi()?;

                        Ok(Statement::Property { var, value, r#type: Some(prop_type), flags: flags.into_iter().map(|f| f.into()).collect() })
                    },
                    TokenKind::Variable(_) => {
                        let var = expect!(self, TokenKind::Variable(v), v, "expected variable name");
                        let mut value = None;

                        if self.current.kind == TokenKind::Equals {
                            self.next();
                            value = Some(self.expression(0)?);
                        }

                        self.semi()?;

                        Ok(Statement::Property { var, value, r#type:None, flags: flags.into_iter().map(|f| f.into()).collect() })
                    },
                    _ => Err(ParseError::UnexpectedToken(self.current.kind.to_string(), self.current.span))
                }
            },
            TokenKind::Function => {
                match self.function(FunctionKind::Named)? {
                    Statement::Function { name, params, body, return_type } => {
                        Ok(Statement::Method { name, params, body, flags: vec![], return_type })
                    },
                    _ => unreachable!(),
                }
            },
            // TODO: Support use statements.
            _ => Err(ParseError::UnexpectedToken(format!("{}", self.current.kind), self.current.span))
        }
    }

    fn expression(&mut self, bp: u8) -> Result<Expression, ParseError> {
        if self.is_eof() {
            return Err(ParseError::UnexpectedEndOfFile);
        }

        let mut lhs = match &self.current.kind {
            TokenKind::Variable(v) => Expression::Variable(v.to_string()),
            TokenKind::Int(i) => Expression::Int(*i),
            TokenKind::Identifier(i) => Expression::Identifier(i.to_string()),
            TokenKind::ConstantString(s) => Expression::ConstantString(s.to_string()),
            TokenKind::LeftParen => {
                self.next();

                let e = self.expression(0)?;

                if self.current.kind != TokenKind::RightParen {
                    return Err(ParseError::ExpectedToken("expected right paren".into(), self.current.span));
                }

                e
            },
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
            TokenKind::Function => {
                // TODO: Support `use (...)` as part of long closure.
                match self.function(FunctionKind::Closure)? {
                    Statement::Function { params, body, return_type, .. } => {
                        Expression::Closure(params, return_type, body)
                    },
                    _ => unreachable!()
                }
            },
            TokenKind::Fn => {
                self.next();

                self.lparen()?;

                let params = self.param_list()?;

                self.rparen()?;
        
                let mut return_type = None;
        
                if self.current.kind == TokenKind::Colon {
                    self.next();
        
                    return_type = Some(self.type_string()?);
                }
                
                expect!(self, TokenKind::DoubleArrow, "expected =>");

                let value = self.expression(0)?;

                Expression::ArrowFunction(params, return_type, Box::new(value))
            },
            TokenKind::New => {
                self.next();

                // TODO: Support dynamic instantiation targets here.
                let target = self.expression(20)?;

                let mut args = vec![];
                if self.current.kind == TokenKind::LeftParen {
                    self.lparen()?;

                    while self.current.kind != TokenKind::RightParen {
                        let value = self.expression(0)?;

                        args.push(value);

                        if self.current.kind == TokenKind::Comma {
                            self.next();
                        }
                    }

                    self.rparen()?;
                }

                Expression::New(Box::new(target), args)
            },
            _ => todo!("expr lhs: {:?}", self.current.kind),
        };

        if self.current.kind == TokenKind::SemiColon {
            return Ok(lhs);
        }

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

                self.rparen()?;
    
                Expression::Call(Box::new(lhs), args)
            },
            TokenKind::Arrow => {
                // TODO: Add support for dynamic property fetch or method call here.
                let property = self.ident()?;

                if self.current.kind == TokenKind::LeftParen {
                    self.next();

                    let mut args = Vec::new();
                    while self.current.kind != TokenKind::RightParen {
                        let arg = self.expression(0)?;

                        if self.current.kind == TokenKind::Comma {
                            self.next();
                        }

                        args.push(arg);
                    }

                    self.rparen()?;

                    Expression::MethodCall(Box::new(lhs), property.into(), args)
                } else {
                    Expression::PropertyFetch(Box::new(lhs), property.into())
                }
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
        TokenKind::Asterisk | TokenKind::Slash => (13, 14),
        TokenKind::Plus | TokenKind::Minus => (11, 12),
        TokenKind::Dot => (11, 11),
        TokenKind::LessThan => (9, 10),
        TokenKind::Equals => (2, 1),
        _ => return None,
    })
}

fn postfix_binding_power(t: &TokenKind) -> Option<u8> {
    Some(match t {
        TokenKind::LeftParen => 19,
        TokenKind::Arrow => 18,
        _ => return None
    })
}

#[derive(Debug)]
pub enum ParseError {
    ExpectedToken(String, Span),
    UnexpectedToken(String, Span),
    UnexpectedEndOfFile,
    InvalidClassStatement(String, Span),
    InvalidAbstractFinalFlagCombination(Span),
    ConstantCannotBeStatic(Span),
    ConstantCannotBePrivateFinal(Span),
    TraitCannotContainConstant(Span),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExpectedToken(message, span) => write!(f, "Parse error: {} on line {} column {}", message, span.0, span.1),
            Self::InvalidClassStatement(message, span) => write!(f, "Parse error: {} on line {} column {}", message, span.0, span.1),
            Self::UnexpectedEndOfFile => write!(f, "Parse error: unexpected end of file."),
            Self::UnexpectedToken(message, span) => write!(f, "Parse error: unexpected token {} on line {} column {}", message, span.0, span.1),
            Self::InvalidAbstractFinalFlagCombination(span) => write!(f, "Parse error: final cannot be used on an abstract class member on line {}", span.0),
            Self::ConstantCannotBeStatic(span) => write!(f, "Parse error: class constant cannot be marked static on line {}", span.0),
            Self::ConstantCannotBePrivateFinal(span) => write!(f, "Parse error: private class constant cannot be marked final since it is not visible to other classes on line {}", span.0),
            Self::TraitCannotContainConstant(span) => write!(f, "Parse error: traits cannot contain constants on line {}", span.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use trunk_lexer::Lexer;
    use crate::{Statement, Param, Expression, ast::{InfixOp, ElseIf}, Type, Identifier};
    use super::Parser;

    macro_rules! function {
        ($name:literal, $params:expr, $body:expr) => {
            Statement::Function {
                name: $name.to_string().into(),
                params: $params.to_vec().into_iter().map(|p: &str| Param::from(p)).collect::<Vec<Param>>(),
                body: $body.to_vec(),
                return_type: None,
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
                return_type: None,
            }
        };
    }

    macro_rules! expr {
        ($expr:expr) => {
            Statement::Expression {
                expr: $expr,
            }
        };
    }

    #[test]
    fn paren_expression() {
        assert_ast("<?php (1 + 2);", &[
            Statement::Expression { expr: Expression::Infix(
                Box::new(Expression::Int(1)),
                InfixOp::Add,
                Box::new(Expression::Int(2))
            ) }
        ]);
    }

    #[test]
    fn breaks() {
        assert_ast("<?php break;", &[
            Statement::Break { num: None }
        ]);

        assert_ast("<?php break 2;", &[
            Statement::Break { num: Some(Expression::Int(2)) }
        ]);
    }

    #[test]
    fn continues() {
        assert_ast("<?php continue;", &[
            Statement::Continue { num: None }
        ]);

        assert_ast("<?php continue 2;", &[
            Statement::Continue { num: Some(Expression::Int(2)) }
        ]);
    }

    #[test]
    fn math_precedence() {
        assert_ast("<?php 1 + 2 * 3 / 4 - 5;", &[
            expr!(Expression::Infix(
                Box::new(Expression::Infix(
                    Box::new(Expression::Int(1)),
                    InfixOp::Add,
                    Box::new(Expression::Infix(
                        Box::new(Expression::Infix(
                            Box::new(Expression::Int(2)),
                            InfixOp::Mul,
                            Box::new(Expression::Int(3))
                        )),
                        InfixOp::Div,
                        Box::new(Expression::Int(4))
                    ))
                )),
                InfixOp::Sub,
                Box::new(Expression::Int(5))
            ))
        ]);
    }

    #[test]
    fn property_fetch() {
        assert_ast("<?php $foo->bar; $foo->bar->baz;", &[
            expr!(Expression::PropertyFetch(
                Box::new(Expression::Variable("foo".into())),
                Identifier::from("bar")
            )),
            expr!(Expression::PropertyFetch(
                Box::new(Expression::PropertyFetch(
                    Box::new(Expression::Variable("foo".into())),
                    Identifier::from("bar")
                )),
                Identifier::from("baz")
            ))
        ]);
    }

    #[test]
    fn method_calls() {
        assert_ast("<?php $foo->bar();", &[
            expr!(Expression::MethodCall(
                Box::new(Expression::Variable("foo".into())),
                Identifier::from("bar"),
                vec![]
            ))
        ]);

        assert_ast("<?php $foo->bar()->baz();", &[
            expr!(Expression::MethodCall(
                Box::new(Expression::MethodCall(
                    Box::new(Expression::Variable("foo".into())),
                    Identifier::from("bar"),
                    vec![]
                )),
                Identifier::from("baz"),
                vec![]
            ))
        ]);

        assert_ast("<?php $foo->bar()();", &[
            expr!(Expression::Call(
                Box::new(Expression::MethodCall(
                    Box::new(Expression::Variable("foo".into())),
                    Identifier::from("bar"),
                    vec![]
                )),
                vec![]
            ))
        ]);
    }

    #[test]
    fn concat() {
        assert_ast("<?php 'foo' . 'bar' . 'baz';", &[
            expr!(Expression::Infix(
                Box::new(Expression::ConstantString("foo".into())),
                InfixOp::Concat,
                Box::new(Expression::Infix(
                    Box::new(Expression::ConstantString("bar".into())),
                    InfixOp::Concat,
                    Box::new(Expression::ConstantString("baz".into())),
                ))
            ))
        ]);
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
                    else_ifs: vec![],
                    r#else: None
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
    fn one_liner_if_statement() {
        assert_ast("<?php if($foo) return $foo;", &[
                Statement::If {
                    condition: Expression::Variable("foo".into()),
                    then: vec![
                        Statement::Return { value: Some(Expression::Variable("foo".into())) }
                    ],
                    else_ifs: vec![],
                    r#else: None
                },
        ]);
    }

    #[test]
    fn if_else_statement() {
        assert_ast("<?php if($foo) { return $foo; } else { return $foo; }", &[
                Statement::If {
                    condition: Expression::Variable("foo".into()),
                    then: vec![
                        Statement::Return { value: Some(Expression::Variable("foo".into())) }
                    ],
                    else_ifs: vec![],
                    r#else: Some(vec![
                        Statement::Return { value: Some(Expression::Variable("foo".into())) }
                    ])
                },
        ]);
    }

    #[test]
    fn if_elseif_else_statement() {
        assert_ast("<?php if($foo) { return $foo; } elseif($foo) { return $foo; } else { return $foo; }", &[
                Statement::If {
                    condition: Expression::Variable("foo".into()),
                    then: vec![
                        Statement::Return { value: Some(Expression::Variable("foo".into())) }
                    ],
                    else_ifs: vec![
                        ElseIf {
                            condition: Expression::Variable("foo".into()),
                            body: vec![
                                Statement::Return { value: Some(Expression::Variable("foo".into())) }
                            ]
                        }
                    ],
                    r#else: Some(vec![
                        Statement::Return { value: Some(Expression::Variable("foo".into())) }
                    ])
                },
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
    fn plain_typestrings_test() {
        assert_ast("<?php function foo(string $b) {}", &[
            Statement::Function {
                name: "foo".to_string().into(),
                params: vec![
                    Param {
                        name: Expression::Variable("b".into()),
                        r#type: Some(Type::Plain("string".into()))
                    }
                ],
                body: vec![],
                return_type: None,
            }
        ]);
    }

    #[test]
    fn nullable_typestrings_test() {
        assert_ast("<?php function foo(?string $b) {}", &[
            Statement::Function {
                name: "foo".to_string().into(),
                params: vec![
                    Param {
                        name: Expression::Variable("b".into()),
                        r#type: Some(Type::Nullable("string".into()))
                    }
                ],
                body: vec![],
                return_type: None,
            }
        ]);
    }

    #[test]
    fn union_typestrings_test() {
        assert_ast("<?php function foo(int|float $b) {}", &[
            Statement::Function {
                name: "foo".to_string().into(),
                params: vec![
                    Param {
                        name: Expression::Variable("b".into()),
                        r#type: Some(Type::Union(vec![
                            "int".into(),
                            "float".into()
                        ]))
                    }
                ],
                body: vec![],
                return_type: None,
            },
        ]);
    }

    #[test]
    fn intersection_typestrings_test() {
        assert_ast("<?php function foo(Foo&Bar $b) {}", &[
            Statement::Function {
                name: "foo".to_string().into(),
                params: vec![
                    Param {
                        name: Expression::Variable("b".into()),
                        r#type: Some(Type::Intersection(vec![
                            "Foo".into(),
                            "Bar".into()
                        ]))
                    }
                ],
                body: vec![],
                return_type: None,
            }
        ]);
    }

    #[test]
    fn function_return_types() {
        assert_ast("<?php function foo(): string {}", &[
            Statement::Function {
                name: "foo".to_string().into(),
                params: vec![],
                body: vec![],
                return_type: Some(Type::Plain("string".into()))
            }
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