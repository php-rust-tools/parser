use crate::{
    ast::{
        Arg, ArrayItem, BackedEnumType, ClassFlag, ClosureUse, Constant, DeclareItem, ElseIf,
        IncludeKind, MagicConst, MethodFlag, StaticVar, Use, UseKind,
    },
    Block, Case, Catch, Expression, Identifier, MatchArm, Program, Statement, Type,
};
use std::{fmt::Display, vec::IntoIter};
use trunk_lexer::{Span, Token, TokenKind};

use self::precedence::{Associativity, Precedence};

type ParseResult<T> = Result<T, ParseError>;

macro_rules! expect {
    ($parser:expr, $expected:pat, $out:expr, $message:literal) => {{
        $parser.skip_comments();
        match $parser.current.kind.clone() {
            $expected => {
                $parser.next();
                $out
            }
            _ => {
                return Err(ParseError::ExpectedToken(
                    $message.into(),
                    $parser.current.span,
                ))
            }
        }
    }};
    ($parser:expr, $expected:pat, $message:literal) => {{
        $parser.skip_comments();
        match $parser.current.kind.clone() {
            $expected => {
                $parser.next();
            }
            _ => {
                return Err(ParseError::ExpectedToken(
                    $message.into(),
                    $parser.current.span,
                ))
            }
        }
    }};
}

mod block;
mod comments;
mod ident;
mod params;
mod precedence;
mod punc;

pub struct ParserConfig {
    force_type_strings: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            force_type_strings: false,
        }
    }
}

pub struct Parser {
    config: ParserConfig,
    pub current: Token,
    pub peek: Token,
    iter: IntoIter<Token>,
    comments: Vec<Token>,
}

#[allow(dead_code)]
impl Parser {
    pub fn new(config: Option<ParserConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            current: Token::default(),
            peek: Token::default(),
            iter: vec![].into_iter(),
            comments: vec![],
        }
    }

    pub fn parse(&mut self, tokens: Vec<Token>) -> Result<Program, ParseError> {
        self.iter = tokens.into_iter();
        self.next();
        self.next();

        let mut ast = Program::new();

        while self.current.kind != TokenKind::Eof {
            if matches!(
                self.current.kind,
                TokenKind::OpenTag(_) | TokenKind::CloseTag
            ) {
                self.next();
                continue;
            }

            self.gather_comments();

            if self.is_eof() {
                break;
            }

            ast.push(self.statement()?);

            self.clear_comments();
        }

        Ok(ast.to_vec())
    }

    fn type_string(&mut self) -> ParseResult<Type> {
        if self.current.kind == TokenKind::Question {
            self.next();
            let t = self.type_with_static()?;
            return Ok(Type::Nullable(t));
        }

        let id = self.type_with_static()?;

        if self.current.kind == TokenKind::Pipe {
            self.next();

            let mut types = vec![id];

            while !self.is_eof() {
                let id = self.type_with_static()?;
                types.push(id);

                if self.current.kind != TokenKind::Pipe {
                    break;
                } else {
                    self.next();
                }
            }

            return Ok(Type::Union(types));
        }

        if self.current.kind == TokenKind::Ampersand {
            self.next();

            let mut types = vec![id];

            while !self.is_eof() {
                let id = self.type_with_static()?;
                types.push(id);

                if self.current.kind != TokenKind::Ampersand {
                    break;
                } else {
                    self.next();
                }
            }

            return Ok(Type::Intersection(types));
        }

        Ok(match &id[..] {
            b"void" => Type::Void,
            _ => Type::Plain(id),
        })
    }

    fn statement(&mut self) -> ParseResult<Statement> {
        self.skip_comments();

        let statement = match &self.current.kind {
            TokenKind::HaltCompiler => {
                self.next();

                let content = if let TokenKind::InlineHtml(content) = self.current.kind.clone() {
                    self.next();
                    Some(content)
                } else {
                    None
                };

                Statement::HaltCompiler { content }
            }
            TokenKind::Declare => {
                self.next();
                self.lparen()?;

                let mut declares = Vec::new();
                while self.current.kind != TokenKind::RightParen {
                    let key = self.ident()?;

                    expect!(self, TokenKind::Equals, "expected =");

                    let value = self.expression(Precedence::Lowest)?;

                    self.optional_comma()?;

                    declares.push(DeclareItem {
                        key: key.into(),
                        value,
                    });
                }

                self.rparen()?;

                let body = if self.current.kind == TokenKind::LeftBrace {
                    self.next();
                    let b = self.block(&TokenKind::RightBrace)?;
                    self.rbrace()?;
                    b
                } else {
                    self.semi()?;
                    vec![]
                };

                Statement::Declare { declares, body }
            }
            TokenKind::Global => {
                self.next();

                let mut vars = vec![];
                while self.current.kind != TokenKind::SemiColon {
                    vars.push(self.var()?.into());

                    self.optional_comma()?;
                }

                self.semi()?;
                Statement::Global { vars }
            }
            TokenKind::Static if matches!(self.peek.kind, TokenKind::Variable(_)) => {
                self.next();

                let mut vars = vec![];

                while self.current.kind != TokenKind::SemiColon {
                    let var = Expression::Variable { name: self.var()? };
                    let mut default = None;

                    if self.current.kind == TokenKind::Equals {
                        expect!(self, TokenKind::Equals, "expected =");
                        default = Some(self.expression(Precedence::Lowest)?);
                    }

                    self.optional_comma()?;

                    vars.push(StaticVar { var, default })
                }

                self.semi()?;

                Statement::Static { vars }
            }
            TokenKind::InlineHtml(html) => {
                let s = Statement::InlineHtml(html.clone());
                self.next();
                s
            }
            TokenKind::Comment(comment) => {
                let s = Statement::Comment {
                    comment: comment.clone(),
                };
                self.next();
                s
            }
            TokenKind::Do => {
                self.next();

                self.lbrace()?;
                let body = self.block(&TokenKind::RightBrace)?;
                self.rbrace()?;

                expect!(self, TokenKind::While, "expected while");

                self.lparen()?;
                let condition = self.expression(Precedence::Lowest)?;
                self.rparen()?;
                self.semi()?;

                Statement::DoWhile { condition, body }
            }
            TokenKind::While => {
                self.next();
                self.lparen()?;

                let condition = self.expression(Precedence::Lowest)?;

                self.rparen()?;
                self.lbrace()?;

                let body = self.block(&TokenKind::RightBrace)?;

                self.rbrace()?;

                Statement::While { condition, body }
            }
            TokenKind::Include
            | TokenKind::IncludeOnce
            | TokenKind::Require
            | TokenKind::RequireOnce => {
                let kind: IncludeKind = (&self.current.kind).into();
                self.next();

                let path = self.expression(Precedence::Lowest)?;

                self.semi()?;

                Statement::Include { kind, path }
            }
            TokenKind::For => {
                self.next();

                self.lparen()?;

                let mut init = None;
                if self.current.kind != TokenKind::SemiColon {
                    init = Some(self.expression(Precedence::Lowest)?);
                }
                self.semi()?;

                let mut condition = None;
                if self.current.kind != TokenKind::SemiColon {
                    condition = Some(self.expression(Precedence::Lowest)?);
                }
                self.semi()?;

                let mut r#loop = None;
                if self.current.kind != TokenKind::RightParen {
                    r#loop = Some(self.expression(Precedence::Lowest)?);
                }

                self.rparen()?;
                self.lbrace()?;

                let then = self.block(&TokenKind::RightBrace)?;

                self.rbrace()?;

                Statement::For {
                    init,
                    condition,
                    r#loop,
                    then,
                }
            }
            TokenKind::Foreach => {
                self.next();

                self.lparen()?;

                let expr = self.expression(Precedence::Lowest)?;

                expect!(self, TokenKind::As, "expected 'as'");

                let mut by_ref = self.current.kind == TokenKind::Ampersand;
                if by_ref {
                    self.next();
                }

                let mut key_var = None;
                let mut value_var = self.expression(Precedence::Lowest)?;

                if self.current.kind == TokenKind::DoubleArrow {
                    self.next();

                    key_var = Some(value_var.clone());

                    by_ref = self.current.kind == TokenKind::Ampersand;
                    if by_ref {
                        self.next();
                    }

                    value_var = self.expression(Precedence::Lowest)?;
                }

                self.rparen()?;
                self.lbrace()?;

                let body = self.block(&TokenKind::RightBrace)?;

                self.rbrace()?;

                Statement::Foreach {
                    expr,
                    by_ref,
                    key_var,
                    value_var,
                    body,
                }
            }
            TokenKind::Abstract => {
                self.next();

                match self.class()? {
                    Statement::Class {
                        name,
                        extends,
                        implements,
                        body,
                        ..
                    } => Statement::Class {
                        name,
                        extends,
                        implements,
                        body,
                        flag: Some(ClassFlag::Abstract),
                    },
                    _ => unreachable!(),
                }
            }
            TokenKind::Final => {
                self.next();

                match self.class()? {
                    Statement::Class {
                        name,
                        extends,
                        implements,
                        body,
                        ..
                    } => Statement::Class {
                        name,
                        extends,
                        implements,
                        body,
                        flag: Some(ClassFlag::Final),
                    },
                    _ => unreachable!(),
                }
            }
            TokenKind::Readonly if self.peek.kind == TokenKind::Class => {
                self.next();

                match self.class()? {
                    Statement::Class {
                        name,
                        extends,
                        implements,
                        body,
                        ..
                    } => Statement::Class {
                        name,
                        extends,
                        implements,
                        body,
                        flag: Some(ClassFlag::Readonly),
                    },
                    _ => unreachable!(),
                }
            }
            TokenKind::Trait => {
                self.next();

                let name = self.ident()?;

                self.lbrace()?;

                let mut body = Block::new();
                while self.current.kind != TokenKind::RightBrace {
                    match self.class_statement()? {
                        Statement::ClassConstant { .. } => {
                            return Err(ParseError::TraitCannotContainConstant(self.current.span))
                        }
                        s => {
                            body.push(s);
                        }
                    }
                }

                self.rbrace()?;

                Statement::Trait {
                    name: name.into(),
                    body,
                }
            }
            TokenKind::Interface => {
                self.next();

                let name = self.ident()?;

                let mut extends = vec![];
                if self.current.kind == TokenKind::Extends {
                    self.next();

                    while self.current.kind != TokenKind::LeftBrace {
                        self.optional_comma()?;

                        let e = self.full_name()?;

                        extends.push(e.into());
                    }
                }

                self.lbrace()?;

                let mut body = Block::new();
                self.skip_comments();
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

                            if self.current.kind == TokenKind::Colon
                                || self.config.force_type_strings
                            {
                                expect!(self, TokenKind::Colon, "expected :");

                                return_type = Some(self.type_string()?);
                            }

                            self.semi()?;

                            body.push(Statement::Method {
                                name: name.into(),
                                params,
                                body: vec![],
                                return_type,
                                flags: vec![MethodFlag::Public],
                                by_ref: false,
                            })
                        }
                        TokenKind::Function => {
                            self.next();

                            let name = self.ident()?;

                            self.lparen()?;

                            let params = self.param_list()?;

                            self.rparen()?;

                            let mut return_type = None;

                            if self.current.kind == TokenKind::Colon
                                || self.config.force_type_strings
                            {
                                expect!(self, TokenKind::Colon, "expected :");

                                return_type = Some(self.type_string()?);
                            }

                            self.semi()?;

                            body.push(Statement::Method {
                                name: name.into(),
                                params,
                                body: vec![],
                                return_type,
                                flags: vec![],
                                by_ref: false,
                            })
                        }
                        _ => {
                            return Err(ParseError::UnexpectedToken(
                                self.current.kind.to_string(),
                                self.current.span,
                            ))
                        }
                    };

                    self.skip_comments();
                }

                self.rbrace()?;

                Statement::Interface {
                    name: name.into(),
                    extends,
                    body,
                }
            }
            TokenKind::Enum if matches!(self.peek.kind, TokenKind::Identifier(_)) => {
                self.next();

                let name = self.ident()?;

                let backed_type: Option<BackedEnumType> = if self.current.kind == TokenKind::Colon {
                    expect!(self, TokenKind::Colon, "expected :");

                    match self.current.kind.clone() {
                        TokenKind::Identifier(s) if s == b"string" || s == b"int" => {
                            self.next();

                            Some(match &s[..] {
                                b"string" => BackedEnumType::String,
                                b"int" => BackedEnumType::Int,
                                _ => unreachable!(),
                            })
                        }
                        _ => {
                            return Err(ParseError::UnexpectedToken(
                                self.current.kind.to_string(),
                                self.current.span,
                            ))
                        }
                    }
                } else {
                    None
                };

                let mut implements = Vec::new();
                if self.current.kind == TokenKind::Implements {
                    self.next();

                    while self.current.kind != TokenKind::LeftBrace {
                        implements.push(self.full_name()?.into());

                        self.optional_comma()?;
                    }
                }

                self.lbrace()?;

                let mut body = Block::new();
                while self.current.kind != TokenKind::RightBrace {
                    match self.current.kind {
                        TokenKind::Case => {
                            self.next();

                            let name = self.ident()?;
                            let mut value = None;

                            if self.current.kind == TokenKind::Equals {
                                expect!(self, TokenKind::Equals, "expected =");

                                value = Some(self.expression(Precedence::Lowest)?);
                            }

                            self.semi()?;

                            body.push(Statement::EnumCase {
                                name: name.into(),
                                value,
                            })
                        }
                        _ => {
                            body.push(self.class_statement()?);
                        }
                    }
                }

                self.rbrace()?;

                Statement::Enum {
                    name: name.into(),
                    backed_type,
                    implements,
                    body,
                }
            }
            TokenKind::Use => {
                self.next();

                let kind = match self.current.kind {
                    TokenKind::Function => {
                        self.next();
                        UseKind::Function
                    }
                    TokenKind::Const => {
                        self.next();
                        UseKind::Const
                    }
                    _ => UseKind::Normal,
                };

                let mut uses = Vec::new();
                while !self.is_eof() {
                    let name = self.full_name()?;
                    let mut alias = None;

                    if self.current.kind == TokenKind::As {
                        self.next();
                        alias = Some(self.ident()?.into());
                    }

                    uses.push(Use {
                        name: name.into(),
                        alias,
                    });

                    if self.current.kind == TokenKind::Comma {
                        self.next();
                        continue;
                    }

                    self.semi()?;
                    break;
                }

                Statement::Use { uses, kind }
            }
            TokenKind::Switch => {
                self.next();

                self.lparen()?;

                let condition = self.expression(Precedence::Lowest)?;

                self.rparen()?;

                self.lbrace()?;

                let mut cases = Vec::new();
                loop {
                    if self.current.kind == TokenKind::RightBrace {
                        break;
                    }

                    match self.current.kind {
                        TokenKind::Case => {
                            self.next();

                            let condition = self.expression(Precedence::Lowest)?;

                            expect!(self, TokenKind::Colon | TokenKind::SemiColon, "expected :");

                            let mut body = Block::new();

                            while self.current.kind != TokenKind::Case
                                && self.current.kind != TokenKind::Default
                                && self.current.kind != TokenKind::RightBrace
                            {
                                body.push(self.statement()?);
                            }

                            cases.push(Case {
                                condition: Some(condition),
                                body,
                            });
                        }
                        TokenKind::Default => {
                            self.next();

                            expect!(self, TokenKind::Colon | TokenKind::SemiColon, "expected :");

                            let mut body = Block::new();

                            while self.current.kind != TokenKind::Case
                                && self.current.kind != TokenKind::Default
                                && self.current.kind != TokenKind::RightBrace
                            {
                                body.push(self.statement()?);
                            }

                            cases.push(Case {
                                condition: None,
                                body,
                            });
                        }
                        _ => {
                            return Err(ParseError::UnexpectedToken(
                                self.current.kind.to_string(),
                                self.current.span,
                            ))
                        }
                    }
                }

                self.rbrace()?;

                Statement::Switch { condition, cases }
            }
            TokenKind::Namespace => {
                self.next();

                let mut braced = false;

                let name = if self.current.kind == TokenKind::LeftBrace {
                    braced = true;
                    self.lbrace()?;
                    None
                } else {
                    Some(self.name()?)
                };

                if name.is_some() {
                    if self.current.kind == TokenKind::LeftBrace {
                        braced = true;
                        self.next();
                    } else {
                        self.semi()?;
                    }
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
            }
            TokenKind::If => {
                self.next();

                self.lparen()?;

                let condition = self.expression(Precedence::Lowest)?;

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

                        let condition = self.expression(Precedence::Lowest)?;

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
                    return Ok(Statement::If {
                        condition,
                        then,
                        else_ifs,
                        r#else: None,
                    });
                }

                expect!(self, TokenKind::Else, "expected else");

                self.lbrace()?;

                let r#else = self.block(&TokenKind::RightBrace)?;

                self.rbrace()?;

                Statement::If {
                    condition,
                    then,
                    else_ifs,
                    r#else: Some(r#else),
                }
            }
            TokenKind::Class => self.class()?,
            TokenKind::Echo => {
                self.next();

                let mut values = Vec::new();
                while !self.is_eof() && self.current.kind != TokenKind::SemiColon {
                    values.push(self.expression(Precedence::Lowest)?);

                    self.optional_comma()?;
                }
                self.semi()?;
                Statement::Echo { values }
            }
            TokenKind::Continue => {
                self.next();

                let mut num = None;
                if self.current.kind != TokenKind::SemiColon {
                    num = Some(self.expression(Precedence::Lowest)?);
                }

                self.semi()?;

                Statement::Continue { num }
            }
            TokenKind::Break => {
                self.next();

                let mut num = None;
                if self.current.kind != TokenKind::SemiColon {
                    num = Some(self.expression(Precedence::Lowest)?);
                }

                self.semi()?;

                Statement::Break { num }
            }
            TokenKind::Return => {
                self.next();

                if let Token {
                    kind: TokenKind::SemiColon,
                    ..
                } = self.current
                {
                    let ret = Statement::Return { value: None };
                    self.semi()?;
                    ret
                } else {
                    let ret = Statement::Return {
                        value: self.expression(Precedence::Lowest).ok(),
                    };
                    self.semi()?;
                    ret
                }
            }
            TokenKind::Function
                if matches!(
                    self.peek.kind,
                    TokenKind::Identifier(_) | TokenKind::Ampersand
                ) =>
            {
                // FIXME: This is incredibly hacky but we don't have a way to look at
                // the next N tokens right now. We could probably do with a `peek_buf()`
                // method like the Lexer has.
                if self.peek.kind == TokenKind::Ampersand {
                    let mut cloned = self.iter.clone();
                    for (index, _) in self.iter.clone().enumerate() {
                        if !matches!(
                            cloned.nth(index),
                            Some(Token {
                                kind: TokenKind::Identifier(_),
                                ..
                            })
                        ) {
                            let expr = self.expression(Precedence::Lowest)?;

                            self.semi()?;

                            return Ok(Statement::Expression { expr });
                        }

                        break;
                    }

                    self.function()?
                } else {
                    self.function()?
                }
            }
            TokenKind::SemiColon => {
                self.next();

                Statement::Noop
            }
            TokenKind::Try => {
                let start_span = self.current.span;

                self.next();
                self.lbrace()?;

                let body = self.block(&TokenKind::RightBrace)?;

                self.rbrace()?;

                let mut catches = Vec::new();
                loop {
                    if self.current.kind != TokenKind::Catch {
                        break;
                    }

                    self.next();
                    self.lparen()?;

                    let types = match self.type_string()? {
                        Type::Plain(t) => vec![t.into()],
                        Type::Union(ts) => ts
                            .into_iter()
                            .map(|t| t.into())
                            .collect::<Vec<Identifier>>(),
                        _ => return Err(ParseError::InvalidCatchArgumentType(self.current.span)),
                    };

                    let var = if self.current.kind == TokenKind::RightParen {
                        None
                    } else {
                        Some(self.expression(Precedence::Lowest)?)
                    };

                    self.rparen()?;
                    self.lbrace()?;

                    let body = self.block(&TokenKind::RightBrace)?;

                    self.rbrace()?;

                    catches.push(Catch { types, var, body })
                }

                let mut finally = None;
                if self.current.kind == TokenKind::Finally {
                    self.next();
                    self.lbrace()?;

                    finally = Some(self.block(&TokenKind::RightBrace)?);

                    self.rbrace()?;
                }

                if catches.is_empty() && finally == None {
                    return Err(ParseError::TryWithoutCatchOrFinally(start_span));
                }

                Statement::Try {
                    body,
                    catches,
                    finally,
                }
            }
            TokenKind::LeftBrace => {
                self.next();
                let body = self.block(&TokenKind::RightBrace)?;
                self.rbrace()?;
                Statement::Block { body }
            }
            TokenKind::Const => {
                self.next();

                let mut constants = vec![];

                while self.current.kind != TokenKind::SemiColon {
                    let name = self.ident()?;

                    expect!(self, TokenKind::Equals, "expected =");

                    let value = self.expression(Precedence::Lowest)?;

                    constants.push(Constant {
                        name: name.into(),
                        value,
                    });

                    self.optional_comma()?;
                }

                self.semi()?;

                Statement::Constant { constants }
            }
            _ => {
                let expr = self.expression(Precedence::Lowest)?;

                self.semi()?;

                Statement::Expression { expr }
            }
        };

        self.skip_comments();

        Ok(statement)
    }

    fn function(&mut self) -> ParseResult<Statement> {
        self.next();

        let by_ref = if self.current.kind == TokenKind::Ampersand {
            self.next();
            true
        } else {
            false
        };

        let name = self.ident()?;

        self.lparen()?;

        let params = self.param_list()?;

        self.rparen()?;

        let mut return_type = None;

        if self.current.kind == TokenKind::Colon || self.config.force_type_strings {
            expect!(self, TokenKind::Colon, "expected :");

            return_type = Some(self.type_string()?);
        }

        self.lbrace()?;

        let body = self.block(&TokenKind::RightBrace)?;

        self.rbrace()?;

        Ok(Statement::Function {
            name: name.into(),
            params,
            body,
            return_type,
            by_ref,
        })
    }

    fn class(&mut self) -> ParseResult<Statement> {
        self.next();

        let name = self.ident()?;
        let mut extends: Option<Identifier> = None;

        if self.current.kind == TokenKind::Extends {
            self.next();
            extends = Some(self.full_name()?.into());
        }

        let mut implements = Vec::new();
        if self.current.kind == TokenKind::Implements {
            self.next();

            while self.current.kind != TokenKind::LeftBrace {
                self.optional_comma()?;

                implements.push(self.full_name()?.into());
            }
        }

        self.lbrace()?;

        let mut body = Vec::new();
        while self.current.kind != TokenKind::RightBrace && !self.is_eof() {
            self.gather_comments();

            if self.current.kind == TokenKind::RightBrace {
                self.clear_comments();
                break;
            }

            body.push(self.class_statement()?);
        }
        self.rbrace()?;

        Ok(Statement::Class {
            name: name.into(),
            extends,
            implements,
            body,
            flag: None,
        })
    }

    fn class_statement(&mut self) -> ParseResult<Statement> {
        match self.current.kind {
            TokenKind::Use => {
                self.next();

                let mut traits = Vec::new();

                while self.current.kind != TokenKind::SemiColon {
                    self.optional_comma()?;

                    let t = self.full_name()?;
                    traits.push(t.into());
                }

                self.semi()?;

                Ok(Statement::TraitUse { traits })
            }
            TokenKind::Const => {
                self.next();

                let name = self.ident()?;

                expect!(self, TokenKind::Equals, "expected =");

                let value = self.expression(Precedence::Lowest)?;

                self.semi()?;

                Ok(Statement::ClassConstant {
                    name: name.into(),
                    value,
                    flags: vec![],
                })
            }
            TokenKind::Var => {
                self.next();

                let mut var_type = None;

                if !matches!(self.current.kind, TokenKind::Variable(_))
                    || self.config.force_type_strings
                {
                    var_type = Some(self.type_string()?);
                }

                let var = self.var()?;
                let mut value = None;

                if self.current.kind == TokenKind::Equals {
                    self.next();

                    value = Some(self.expression(Precedence::Lowest)?);
                }

                self.semi()?;

                Ok(Statement::Var {
                    var,
                    value,
                    r#type: var_type,
                })
            }
            TokenKind::Final
            | TokenKind::Abstract
            | TokenKind::Public
            | TokenKind::Private
            | TokenKind::Protected
            | TokenKind::Static
            | TokenKind::Readonly => {
                let mut flags = vec![self.current.kind.clone()];
                self.next();

                while !self.is_eof()
                    && [
                        TokenKind::Final,
                        TokenKind::Abstract,
                        TokenKind::Public,
                        TokenKind::Private,
                        TokenKind::Protected,
                        TokenKind::Static,
                        TokenKind::Readonly,
                    ]
                    .contains(&self.current.kind)
                {
                    if flags.contains(&self.current.kind) {
                        return Err(ParseError::UnexpectedToken(
                            self.current.kind.to_string(),
                            self.current.span,
                        ));
                    }

                    flags.push(self.current.kind.clone());
                    self.next();
                }

                if flags.contains(&TokenKind::Final) && flags.contains(&TokenKind::Abstract) {
                    return Err(ParseError::InvalidAbstractFinalFlagCombination(
                        self.current.span,
                    ));
                }

                match self.current.kind {
                    TokenKind::Const => {
                        if flags.contains(&TokenKind::Static) {
                            return Err(ParseError::ConstantCannotBeStatic(self.current.span));
                        }

                        if flags.contains(&TokenKind::Final) && flags.contains(&TokenKind::Private)
                        {
                            return Err(ParseError::ConstantCannotBePrivateFinal(
                                self.current.span,
                            ));
                        }

                        self.next();

                        let name = self.ident()?;

                        expect!(self, TokenKind::Equals, "expected =");

                        let value = self.expression(Precedence::Lowest)?;

                        self.semi()?;

                        Ok(Statement::ClassConstant {
                            name: name.into(),
                            value,
                            flags: flags.into_iter().map(|f| f.into()).collect(),
                        })
                    }
                    TokenKind::Function => {
                        if flags.contains(&TokenKind::Abstract) {
                            self.next();

                            let by_ref = if self.current.kind == TokenKind::Ampersand {
                                self.next();
                                true
                            } else {
                                false
                            };

                            let name = self.ident()?;

                            self.lparen()?;

                            let params = self.param_list()?;

                            self.rparen()?;

                            let mut return_type = None;

                            if self.current.kind == TokenKind::Colon
                                || self.config.force_type_strings
                            {
                                expect!(self, TokenKind::Colon, "expected :");

                                return_type = Some(self.type_string()?);
                            }

                            self.semi()?;

                            Ok(Statement::Method {
                                name: name.into(),
                                params,
                                body: vec![],
                                return_type,
                                flags: flags.iter().map(|t| t.clone().into()).collect(),
                                by_ref,
                            })
                        } else {
                            match self.function()? {
                                Statement::Function {
                                    name,
                                    params,
                                    body,
                                    return_type,
                                    by_ref,
                                } => Ok(Statement::Method {
                                    name,
                                    params,
                                    body,
                                    flags: flags.iter().map(|t| t.clone().into()).collect(),
                                    return_type,
                                    by_ref,
                                }),
                                _ => unreachable!(),
                            }
                        }
                    }
                    TokenKind::Question
                    | TokenKind::Identifier(_)
                    | TokenKind::QualifiedIdentifier(_)
                    | TokenKind::FullyQualifiedIdentifier(_)
                    | TokenKind::Array => {
                        let prop_type = self.type_string()?;
                        let var = self.var()?;
                        let mut value = None;

                        if self.current.kind == TokenKind::Equals {
                            self.next();
                            value = Some(self.expression(Precedence::Lowest)?);
                        }

                        // TODO: Support comma-separated property declarations.
                        //       nikic/php-parser does this with a single Property statement
                        //       that is capable of holding multiple property declarations.
                        self.semi()?;

                        Ok(Statement::Property {
                            var,
                            value,
                            r#type: Some(prop_type),
                            flags: flags.into_iter().map(|f| f.into()).collect(),
                        })
                    }
                    TokenKind::Variable(_) => {
                        let var = self.var()?;
                        let mut value = None;

                        if self.current.kind == TokenKind::Equals {
                            self.next();
                            value = Some(self.expression(Precedence::Lowest)?);
                        }

                        self.semi()?;

                        Ok(Statement::Property {
                            var,
                            value,
                            r#type: None,
                            flags: flags.into_iter().map(|f| f.into()).collect(),
                        })
                    }
                    _ => Err(ParseError::UnexpectedToken(
                        self.current.kind.to_string(),
                        self.current.span,
                    )),
                }
            }
            TokenKind::Function => match self.function()? {
                Statement::Function {
                    name,
                    params,
                    body,
                    return_type,
                    by_ref,
                } => Ok(Statement::Method {
                    name,
                    params,
                    body,
                    flags: vec![],
                    return_type,
                    by_ref,
                }),
                _ => unreachable!(),
            },
            // TODO: Support use statements.
            _ => Err(ParseError::UnexpectedToken(
                format!("{}", self.current.kind),
                self.current.span,
            )),
        }
    }

    fn expression(&mut self, precedence: Precedence) -> ParseResult<Expression> {
        if self.is_eof() {
            return Err(ParseError::UnexpectedEndOfFile);
        }

        self.skip_comments();

        let mut left = match &self.current.kind {
            TokenKind::Throw => {
                self.next();

                let value = self.expression(Precedence::Lowest)?;

                Expression::Throw {
                    value: Box::new(value),
                }
            }
            TokenKind::Yield => {
                self.next();

                let value = self.expression(Precedence::Lowest)?;

                // FIXME: Check for presence of => here to allow yielding key and value.

                Expression::Yield {
                    value: Box::new(value),
                }
            }
            TokenKind::Clone => {
                self.next();

                let target = self.expression(Precedence::CloneNew)?;

                Expression::Clone {
                    target: Box::new(target),
                }
            }
            TokenKind::Variable(v) => {
                let e = Expression::Variable { name: v.clone() };
                self.next();
                e
            }
            TokenKind::Int(i) => {
                let e = Expression::Int { i: *i };
                self.next();
                e
            }
            TokenKind::Float(f) => {
                let f = Expression::Float { f: *f };
                self.next();
                f
            }
            TokenKind::Identifier(i)
            | TokenKind::QualifiedIdentifier(i)
            | TokenKind::FullyQualifiedIdentifier(i) => {
                let e = Expression::Identifier { name: i.clone() };
                self.next();
                e
            }
            TokenKind::ConstantString(s) => {
                let e = Expression::ConstantString { value: s.clone() };
                self.next();
                e
            }
            TokenKind::True => {
                let e = Expression::Bool { value: true };
                self.next();
                e
            }
            TokenKind::False => {
                let e = Expression::Bool { value: false };
                self.next();
                e
            }
            TokenKind::Null => {
                self.next();
                Expression::Null
            }
            TokenKind::LeftParen => {
                self.next();

                let e = self.expression(Precedence::Lowest)?;

                self.rparen()?;

                e
            }
            TokenKind::Match => {
                self.next();
                self.lparen()?;

                let condition = Box::new(self.expression(Precedence::Lowest)?);

                self.rparen()?;
                self.lbrace()?;

                let mut arms = Vec::new();
                while self.current.kind != TokenKind::RightBrace {
                    let mut conditions = Vec::new();

                    while self.current.kind != TokenKind::DoubleArrow {
                        if self.current.kind == TokenKind::Default {
                            self.next();
                            break;
                        }

                        conditions.push(self.expression(Precedence::Lowest)?);

                        self.optional_comma()?;
                    }

                    expect!(self, TokenKind::DoubleArrow, "expected =>");

                    let body = self.expression(Precedence::Lowest)?;

                    self.optional_comma()?;

                    arms.push(MatchArm {
                        conditions: if conditions.is_empty() {
                            None
                        } else {
                            Some(conditions)
                        },
                        body,
                    })
                }

                self.rbrace()?;

                Expression::Match { condition, arms }
            }
            TokenKind::Array => {
                let mut items = vec![];

                self.next();

                self.lparen()?;

                while self.current.kind != TokenKind::RightParen {
                    let mut key = None;
                    let mut value = self.expression(Precedence::Lowest)?;

                    if self.current.kind == TokenKind::DoubleArrow {
                        self.next();

                        key = Some(value);
                        value = self.expression(Precedence::Lowest)?;
                    }

                    items.push(ArrayItem { key, value });

                    self.optional_comma()?;

                    self.skip_comments();
                }

                self.rparen()?;

                Expression::Array { items }
            }
            TokenKind::LeftBracket => {
                let mut items = Vec::new();
                self.next();

                self.skip_comments();

                while self.current.kind != TokenKind::RightBracket {
                    if self.current.kind == TokenKind::Comma {
                        items.push(ArrayItem {
                            key: None,
                            value: Expression::Empty,
                        });
                        self.next();
                        continue;
                    }

                    let mut key = None;
                    let mut value = self.expression(Precedence::Lowest)?;

                    if self.current.kind == TokenKind::DoubleArrow {
                        self.next();

                        key = Some(value);
                        value = self.expression(Precedence::Lowest)?;
                    }

                    items.push(ArrayItem { key, value });

                    self.optional_comma()?;

                    self.skip_comments();
                }

                self.rbracket()?;

                Expression::Array { items }
            }
            TokenKind::Static if matches!(self.peek.kind, TokenKind::Function | TokenKind::Fn) => {
                self.next();

                match self.expression(Precedence::Lowest)? {
                    Expression::Closure {
                        params,
                        uses,
                        return_type,
                        body,
                        by_ref,
                        ..
                    } => Expression::Closure {
                        params,
                        uses,
                        return_type,
                        body,
                        by_ref,
                        r#static: true,
                    },
                    Expression::ArrowFunction {
                        params,
                        return_type,
                        expr,
                        by_ref,
                        ..
                    } => Expression::ArrowFunction {
                        params,
                        return_type,
                        expr,
                        by_ref,
                        r#static: true,
                    },
                    _ => unreachable!(),
                }
            }
            TokenKind::Function => {
                self.next();

                let by_ref = if self.current.kind == TokenKind::Ampersand {
                    self.next();
                    true
                } else {
                    false
                };

                self.lparen()?;

                let params = self.param_list()?;

                self.rparen()?;

                let mut uses = vec![];
                if self.current.kind == TokenKind::Use {
                    self.next();

                    self.lparen()?;

                    while self.current.kind != TokenKind::RightParen {
                        let var = match self.current.kind {
                            TokenKind::Ampersand => {
                                self.next();

                                match self.expression(Precedence::Lowest)? {
                                    s @ Expression::Variable { .. } => ClosureUse {
                                        var: s,
                                        by_ref: true,
                                    },
                                    _ => {
                                        return Err(ParseError::UnexpectedToken(
                                            "expected variable".into(),
                                            self.current.span,
                                        ))
                                    }
                                }
                            }
                            _ => match self.expression(Precedence::Lowest)? {
                                s @ Expression::Variable { .. } => ClosureUse {
                                    var: s,
                                    by_ref: false,
                                },
                                _ => {
                                    return Err(ParseError::UnexpectedToken(
                                        "expected variable".into(),
                                        self.current.span,
                                    ))
                                }
                            },
                        };

                        uses.push(var);

                        self.optional_comma()?;
                    }

                    self.rparen()?;
                }

                let mut return_type = None;
                if self.current.kind == TokenKind::Colon || self.config.force_type_strings {
                    expect!(self, TokenKind::Colon, "expected :");

                    return_type = Some(self.type_string()?);
                }

                self.lbrace()?;

                let body = self.block(&TokenKind::RightBrace)?;

                self.rbrace()?;

                Expression::Closure {
                    params,
                    uses,
                    return_type,
                    body,
                    r#static: false,
                    by_ref,
                }
            }
            TokenKind::Fn => {
                self.next();

                let by_ref = if self.current.kind == TokenKind::Ampersand {
                    self.next();
                    true
                } else {
                    false
                };

                self.lparen()?;

                let params = self.param_list()?;

                self.rparen()?;

                let mut return_type = None;

                if self.current.kind == TokenKind::Colon || self.config.force_type_strings {
                    expect!(self, TokenKind::Colon, "expected :");

                    return_type = Some(self.type_string()?);
                }

                expect!(self, TokenKind::DoubleArrow, "expected =>");

                let value = self.expression(Precedence::Lowest)?;

                Expression::ArrowFunction {
                    params,
                    return_type,
                    expr: Box::new(value),
                    by_ref,
                    r#static: false,
                }
            }
            TokenKind::New => {
                self.next();

                let mut args = vec![];
                let target = if self.current.kind == TokenKind::Class {
                    self.next();

                    if self.current.kind == TokenKind::LeftParen {
                        self.lparen()?;

                        while self.current.kind != TokenKind::RightParen {
                            let mut name = None;
                            let mut unpack = false;
                            if matches!(self.current.kind, TokenKind::Identifier(_))
                                && self.peek.kind == TokenKind::Colon
                            {
                                name = Some(self.ident_maybe_reserved()?);
                                self.next();
                            } else if self.current.kind == TokenKind::Ellipsis {
                                self.next();
                                unpack = true;
                            }

                            let value = self.expression(Precedence::Lowest)?;

                            args.push(Arg {
                                name,
                                unpack,
                                value,
                            });

                            self.optional_comma()?;
                        }

                        self.rparen()?;
                    }

                    let mut extends: Option<Identifier> = None;

                    if self.current.kind == TokenKind::Extends {
                        self.next();
                        extends = Some(self.full_name()?.into());
                    }

                    let mut implements = Vec::new();
                    if self.current.kind == TokenKind::Implements {
                        self.next();

                        while self.current.kind != TokenKind::LeftBrace {
                            self.optional_comma()?;

                            implements.push(self.full_name()?.into());
                        }
                    }

                    self.lbrace()?;

                    let mut body = Vec::new();
                    while self.current.kind != TokenKind::RightBrace && !self.is_eof() {
                        body.push(self.class_statement()?);
                    }

                    self.rbrace()?;

                    Expression::AnonymousClass {
                        extends,
                        implements,
                        body,
                    }
                } else {
                    self.expression(Precedence::CloneNew)?
                };

                if self.current.kind == TokenKind::LeftParen {
                    self.lparen()?;

                    while self.current.kind != TokenKind::RightParen {
                        let mut name = None;
                        let mut unpack = false;
                        if matches!(self.current.kind, TokenKind::Identifier(_))
                            && self.peek.kind == TokenKind::Colon
                        {
                            name = Some(self.ident_maybe_reserved()?);
                            self.next();
                        } else if self.current.kind == TokenKind::Ellipsis {
                            self.next();
                            unpack = true;
                        }

                        let value = self.expression(Precedence::Lowest)?;

                        args.push(Arg {
                            name,
                            unpack,
                            value,
                        });

                        self.optional_comma()?;
                    }

                    self.rparen()?;
                }

                Expression::New {
                    target: Box::new(target),
                    args,
                }
            }
            TokenKind::DirConstant => {
                self.next();
                Expression::MagicConst {
                    constant: MagicConst::Dir,
                }
            }
            _ if is_prefix(&self.current.kind) => {
                let op = self.current.kind.clone();

                self.next();

                let rpred = Precedence::prefix(&op);
                let rhs = self.expression(rpred)?;

                prefix(&op, rhs)
            }
            _ => todo!(
                "expr lhs: {:?}, line {} col {}",
                self.current.kind,
                self.current.span.0,
                self.current.span.1
            ),
        };

        if self.current.kind == TokenKind::SemiColon {
            return Ok(left);
        }

        self.skip_comments();

        loop {
            self.skip_comments();

            if matches!(self.current.kind, TokenKind::SemiColon | TokenKind::Eof) {
                break;
            }

            let span = self.current.span;
            let kind = self.current.kind.clone();

            if is_postfix(&kind) {
                let lpred = Precedence::postfix(&kind);

                if lpred < precedence {
                    break;
                }

                self.next();

                left = self.postfix(left, &kind)?;
                continue;
            }

            if is_infix(&kind) {
                let rpred = Precedence::infix(&kind);

                if rpred < precedence {
                    break;
                }

                if rpred == precedence && matches!(rpred.associativity(), Some(Associativity::Left))
                {
                    break;
                }

                if rpred == precedence && matches!(rpred.associativity(), Some(Associativity::Non))
                {
                    return Err(ParseError::UnexpectedToken(kind.to_string(), span));
                }

                self.next();

                match kind {
                    TokenKind::Question => {
                        let then = self.expression(Precedence::Lowest)?;
                        expect!(self, TokenKind::Colon, "expected :");
                        let otherwise = self.expression(rpred)?;
                        left = Expression::Ternary {
                            condition: Box::new(left),
                            then: Some(Box::new(then)),
                            r#else: Box::new(otherwise),
                        }
                    }
                    TokenKind::QuestionColon => {
                        let r#else = self.expression(Precedence::Lowest)?;
                        left = Expression::Ternary {
                            condition: Box::new(left),
                            then: None,
                            r#else: Box::new(r#else),
                        }
                    }
                    _ => {
                        let rhs = self.expression(rpred)?;
                        left = infix(left, kind, rhs);
                    }
                }

                continue;
            }

            break;
        }

        self.skip_comments();

        Ok(left)
    }

    fn postfix(&mut self, lhs: Expression, op: &TokenKind) -> Result<Expression, ParseError> {
        Ok(match op {
            TokenKind::Coalesce => {
                let rhs = self.expression(Precedence::NullCoalesce)?;

                Expression::Coalesce {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }
            }
            TokenKind::LeftParen => {
                let mut args = Vec::new();
                while !self.is_eof() && self.current.kind != TokenKind::RightParen {
                    let mut name = None;
                    let mut unpack = false;
                    if matches!(self.current.kind, TokenKind::Identifier(_))
                        && self.peek.kind == TokenKind::Colon
                    {
                        name = Some(self.ident_maybe_reserved()?);
                        self.next();
                    } else if self.current.kind == TokenKind::Ellipsis {
                        self.next();
                        unpack = true;
                    }

                    if unpack && self.current.kind == TokenKind::RightParen {
                        args.push(Arg {
                            name: None,
                            unpack: false,
                            value: Expression::VariadicPlaceholder
                        });

                        break;
                    }

                    let value = self.expression(Precedence::Lowest)?;

                    args.push(Arg {
                        name,
                        unpack,
                        value,
                    });

                    self.optional_comma()?;
                }

                self.rparen()?;

                Expression::Call {
                    target: Box::new(lhs),
                    args,
                }
            }
            TokenKind::LeftBracket => {
                if self.current.kind == TokenKind::RightBracket {
                    self.next();

                    Expression::ArrayIndex {
                        array: Box::new(lhs),
                        index: None,
                    }
                } else {
                    let index = self.expression(Precedence::Lowest)?;

                    expect!(self, TokenKind::RightBracket, "expected ]");

                    Expression::ArrayIndex {
                        array: Box::new(lhs),
                        index: Some(Box::new(index)),
                    }
                }
            }
            TokenKind::DoubleColon => match self.current.kind.clone() {
                TokenKind::Variable(_) => {
                    let var = self.expression(Precedence::Lowest)?;

                    Expression::StaticPropertyFetch {
                        target: Box::new(lhs),
                        property: Box::new(var),
                    }
                }
                _ => {
                    let ident = if self.current.kind == TokenKind::Class {
                        self.next();

                        b"class".into()
                    } else {
                        self.ident_maybe_reserved()?
                    };

                    if self.current.kind == TokenKind::LeftParen {
                        self.lparen()?;

                        let mut args = vec![];
                        while !self.is_eof() && self.current.kind != TokenKind::RightParen {
                            let mut name = None;
                            let mut unpack = false;
                            if matches!(self.current.kind, TokenKind::Identifier(_))
                                && self.peek.kind == TokenKind::Colon
                            {
                                name = Some(self.ident_maybe_reserved()?);
                                self.next();
                            } else if self.current.kind == TokenKind::Ellipsis {
                                self.next();
                                unpack = true;
                            }

                            let value = self.expression(Precedence::Lowest)?;

                            args.push(Arg {
                                name,
                                unpack,
                                value,
                            });

                            self.optional_comma()?;
                        }

                        self.rparen()?;

                        Expression::StaticMethodCall {
                            target: Box::new(lhs),
                            method: ident.into(),
                            args,
                        }
                    } else {
                        Expression::ConstFetch {
                            target: Box::new(lhs),
                            constant: ident.into(),
                        }
                    }
                }
            },
            TokenKind::Arrow | TokenKind::NullsafeArrow => {
                let property = match self.current.kind {
                    TokenKind::LeftBrace => {
                        self.lbrace()?;
                        let expr = self.expression(Precedence::Lowest)?;
                        self.rbrace()?;
                        expr
                    }
                    TokenKind::Variable(ref var) => {
                        let var = Expression::Variable { name: var.clone() };
                        self.next();
                        var
                    }
                    _ => Expression::Identifier {
                        name: self.ident_maybe_reserved()?,
                    },
                };

                if self.current.kind == TokenKind::LeftParen {
                    self.next();

                    let mut args = Vec::new();
                    while !self.is_eof() && self.current.kind != TokenKind::RightParen {
                        let mut name = None;
                        let mut unpack = false;
                        if matches!(self.current.kind, TokenKind::Identifier(_))
                            && self.peek.kind == TokenKind::Colon
                        {
                            name = Some(self.ident_maybe_reserved()?);
                            self.next();
                        } else if self.current.kind == TokenKind::Ellipsis {
                            self.next();
                            unpack = true;
                        }
                        
                        if unpack && self.current.kind == TokenKind::RightParen {
                            args.push(Arg {
                                name: None,
                                unpack: false,
                                value: Expression::VariadicPlaceholder
                            });

                            break;
                        }

                        let value = self.expression(Precedence::Lowest)?;

                        args.push(Arg {
                            name,
                            value,
                            unpack,
                        });

                        self.optional_comma()?;
                    }

                    self.rparen()?;

                    Expression::MethodCall {
                        target: Box::new(lhs),
                        method: Box::new(property),
                        args,
                    }
                } else if op == &TokenKind::NullsafeArrow {
                    Expression::NullsafePropertyFetch {
                        target: Box::new(lhs),
                        property: Box::new(property),
                    }
                } else {
                    Expression::PropertyFetch {
                        target: Box::new(lhs),
                        property: Box::new(property),
                    }
                }
            }
            TokenKind::Increment => Expression::Increment {
                value: Box::new(lhs),
            },
            TokenKind::Decrement => Expression::Decrement {
                value: Box::new(lhs),
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
}

fn is_prefix(op: &TokenKind) -> bool {
    matches!(
        op,
        TokenKind::Bang
            | TokenKind::Minus
            | TokenKind::StringCast
            | TokenKind::ObjectCast
            | TokenKind::BoolCast
            | TokenKind::IntCast
            | TokenKind::DoubleCast
            | TokenKind::At
    )
}

fn prefix(op: &TokenKind, rhs: Expression) -> Expression {
    match op {
        TokenKind::Bang => Expression::BooleanNot {
            value: Box::new(rhs),
        },
        TokenKind::Minus => Expression::Negate {
            value: Box::new(rhs),
        },
        TokenKind::StringCast
        | TokenKind::ObjectCast
        | TokenKind::BoolCast
        | TokenKind::IntCast
        | TokenKind::DoubleCast => Expression::Cast {
            kind: op.into(),
            value: Box::new(rhs),
        },
        TokenKind::At => Expression::ErrorSuppress {
            expr: Box::new(rhs),
        },
        _ => unreachable!(),
    }
}

fn infix(lhs: Expression, op: TokenKind, rhs: Expression) -> Expression {
    Expression::Infix {
        lhs: Box::new(lhs),
        op: op.into(),
        rhs: Box::new(rhs),
    }
}

fn is_infix(t: &TokenKind) -> bool {
    matches!(
        t,
        TokenKind::Pow
            | TokenKind::Instanceof
            | TokenKind::Asterisk
            | TokenKind::Slash
            | TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Dot
            | TokenKind::LessThan
            | TokenKind::GreaterThan
            | TokenKind::LessThanEquals
            | TokenKind::GreaterThanEquals
            | TokenKind::DoubleEquals
            | TokenKind::TripleEquals
            | TokenKind::BangEquals
            | TokenKind::BangDoubleEquals
            | TokenKind::Question
            | TokenKind::QuestionColon
            | TokenKind::BooleanAnd
            | TokenKind::BooleanOr
            | TokenKind::Equals
            | TokenKind::PlusEquals
            | TokenKind::MinusEquals
            | TokenKind::DotEquals
            | TokenKind::CoalesceEqual
            | TokenKind::AsteriskEqual
            | TokenKind::SlashEquals
    )
}

fn is_postfix(t: &TokenKind) -> bool {
    matches!(
        t,
        TokenKind::Increment
            | TokenKind::Decrement
            | TokenKind::LeftParen
            | TokenKind::LeftBracket
            | TokenKind::Arrow
            | TokenKind::NullsafeArrow
            | TokenKind::DoubleColon
            | TokenKind::Coalesce
    )
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
    TryWithoutCatchOrFinally(Span),
    InvalidCatchArgumentType(Span),
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
            Self::TryWithoutCatchOrFinally(span) => write!(f, "Parse error: cannot use try without catch or finally on line {}", span.0),
            Self::InvalidCatchArgumentType(span) => write!(f, "Parse error: catch types must either describe a single type or union of types on line {}", span.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;
    use crate::{
        ast::{
            Arg, ArrayItem, BackedEnumType, Case, ClassFlag, Constant, DeclareItem, ElseIf,
            IncludeKind, InfixOp, MethodFlag, PropertyFlag,
        },
        Catch, Expression, Identifier, Param, Statement, Type,
    };
    use pretty_assertions::assert_eq;
    use trunk_lexer::Lexer;

    macro_rules! function {
        ($name:literal, $params:expr, $body:expr) => {
            Statement::Function {
                name: $name.as_bytes().into(),
                params: $params
                    .to_vec()
                    .into_iter()
                    .map(|p: &str| Param::from(p.as_bytes()))
                    .collect::<Vec<Param>>(),
                body: $body.to_vec(),
                return_type: None,
                by_ref: false,
            }
        };
    }

    macro_rules! class {
        ($name:literal) => {
            Statement::Class {
                name: $name.as_bytes().into(),
                body: vec![],
                extends: None,
                implements: vec![],
                flag: None,
            }
        };
        ($name:literal, $body:expr) => {
            Statement::Class {
                name: $name.as_bytes().into(),
                body: $body.to_vec(),
                extends: None,
                implements: vec![],
                flag: None,
            }
        };
        ($name:literal, $extends:expr, $implements:expr, $body:expr) => {
            Statement::Class {
                name: $name.as_bytes().into(),
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
                name: $name.as_bytes().into(),
                params: $params
                    .to_vec()
                    .into_iter()
                    .map(|p: &str| Param::from(p.as_bytes()))
                    .collect::<Vec<Param>>(),
                flags: $flags.to_vec(),
                body: $body.to_vec(),
                return_type: None,
                by_ref: false,
            }
        };
    }

    macro_rules! expr {
        ($expr:expr) => {
            Statement::Expression { expr: $expr }
        };
    }

    #[test]
    fn include() {
        assert_ast(
            "<?php include 'foo.php';",
            &[Statement::Include {
                path: Expression::ConstantString {
                    value: "foo.php".into(),
                },
                kind: IncludeKind::Include,
            }],
        );

        assert_ast(
            "<?php include_once 'foo.php';",
            &[Statement::Include {
                path: Expression::ConstantString {
                    value: "foo.php".into(),
                },
                kind: IncludeKind::IncludeOnce,
            }],
        );

        assert_ast(
            "<?php require 'foo.php';",
            &[Statement::Include {
                path: Expression::ConstantString {
                    value: "foo.php".into(),
                },
                kind: IncludeKind::Require,
            }],
        );

        assert_ast(
            "<?php require_once 'foo.php';",
            &[Statement::Include {
                path: Expression::ConstantString {
                    value: "foo.php".into(),
                },
                kind: IncludeKind::RequireOnce,
            }],
        );
    }

    #[test]
    fn instanceof() {
        assert_ast(
            "<?php $foo instanceof Foo;",
            &[expr!(Expression::Infix {
                lhs: Box::new(Expression::Variable { name: "foo".into() }),
                op: InfixOp::Instanceof,
                rhs: Box::new(Expression::Identifier { name: "Foo".into() })
            })],
        );
    }

    #[test]
    fn multiple_instances_of() {
        assert_ast(
            "<?php $foo instanceof Foo && $foo instanceof Foo;",
            &[expr!(Expression::Infix {
                lhs: Box::new(Expression::Infix {
                    lhs: Box::new(Expression::Variable { name: "foo".into() }),
                    op: InfixOp::Instanceof,
                    rhs: Box::new(Expression::Identifier { name: "Foo".into() })
                }),
                op: InfixOp::And,
                rhs: Box::new(Expression::Infix {
                    lhs: Box::new(Expression::Variable { name: "foo".into() }),
                    op: InfixOp::Instanceof,
                    rhs: Box::new(Expression::Identifier { name: "Foo".into() })
                })
            })],
        );
    }

    #[test]
    fn pow() {
        assert_ast(
            "<?php 2 ** 2;",
            &[expr!(Expression::Infix {
                lhs: Box::new(Expression::Int { i: 2 }),
                op: InfixOp::Pow,
                rhs: Box::new(Expression::Int { i: 2 }),
            })],
        );
    }

    #[test]
    fn ternary() {
        assert_ast(
            "<?php 1 ? 2 : 3;",
            &[expr!(Expression::Ternary {
                condition: Box::new(Expression::Int { i: 1 }),
                then: Some(Box::new(Expression::Int { i: 2 })),
                r#else: Box::new(Expression::Int { i: 3 }),
            })],
        );

        assert_ast(
            "<?php 1 ? 2 ? 3 : 4 : 5;",
            &[expr!(Expression::Ternary {
                condition: Box::new(Expression::Int { i: 1 }),
                then: Some(Box::new(Expression::Ternary {
                    condition: Box::new(Expression::Int { i: 2 }),
                    then: Some(Box::new(Expression::Int { i: 3 })),
                    r#else: Box::new(Expression::Int { i: 4 }),
                })),
                r#else: Box::new(Expression::Int { i: 5 }),
            })],
        );
    }

    #[test]
    fn coalesce() {
        assert_ast(
            "<?php 1 ?? 2;",
            &[expr!(Expression::Coalesce {
                lhs: Box::new(Expression::Int { i: 1 }),
                rhs: Box::new(Expression::Int { i: 2 })
            })],
        );

        assert_ast(
            "<?php 1 ?? 2 ?? 3;",
            &[expr!(Expression::Coalesce {
                lhs: Box::new(Expression::Int { i: 1 }),
                rhs: Box::new(Expression::Coalesce {
                    lhs: Box::new(Expression::Int { i: 2 }),
                    rhs: Box::new(Expression::Int { i: 3 })
                })
            })],
        );
    }

    #[test]
    fn array_index() {
        assert_ast(
            "<?php $foo['bar'];",
            &[expr!(Expression::ArrayIndex {
                array: Box::new(Expression::Variable { name: "foo".into() }),
                index: Some(Box::new(Expression::ConstantString {
                    value: "bar".into()
                }))
            })],
        );

        assert_ast(
            "<?php $foo['bar']['baz'];",
            &[expr!(Expression::ArrayIndex {
                array: Box::new(Expression::ArrayIndex {
                    array: Box::new(Expression::Variable { name: "foo".into() }),
                    index: Some(Box::new(Expression::ConstantString {
                        value: "bar".into()
                    }))
                }),
                index: Some(Box::new(Expression::ConstantString {
                    value: "baz".into()
                }))
            })],
        );
    }

    #[test]
    fn array_index_assign() {
        assert_ast(
            "<?php $foo['bar'] = 'baz';",
            &[expr!(Expression::Infix {
                lhs: Box::new(Expression::ArrayIndex {
                    array: Box::new(Expression::Variable { name: "foo".into() }),
                    index: Some(Box::new(Expression::ConstantString {
                        value: "bar".into()
                    }))
                }),
                op: InfixOp::Assign,
                rhs: Box::new(Expression::ConstantString {
                    value: "baz".into()
                })
            })],
        );
    }

    #[test]
    fn comparisons() {
        assert_ast(
            "<?php 1 == 1;",
            &[expr!(Expression::Infix {
                lhs: Box::new(Expression::Int { i: 1 }),
                op: InfixOp::Equals,
                rhs: Box::new(Expression::Int { i: 1 })
            })],
        );

        assert_ast(
            "<?php 1 === 1;",
            &[expr!(Expression::Infix {
                lhs: Box::new(Expression::Int { i: 1 }),
                op: InfixOp::Identical,
                rhs: Box::new(Expression::Int { i: 1 })
            })],
        );

        assert_ast(
            "<?php 1 != 1;",
            &[expr!(Expression::Infix {
                lhs: Box::new(Expression::Int { i: 1 }),
                op: InfixOp::NotEquals,
                rhs: Box::new(Expression::Int { i: 1 })
            })],
        );

        assert_ast(
            "<?php 1 !== 1;",
            &[expr!(Expression::Infix {
                lhs: Box::new(Expression::Int { i: 1 }),
                op: InfixOp::NotIdentical,
                rhs: Box::new(Expression::Int { i: 1 })
            })],
        );
    }

    #[test]
    fn paren_expression() {
        assert_ast(
            "<?php (1 + 2);",
            &[Statement::Expression {
                expr: Expression::Infix {
                    lhs: Box::new(Expression::Int { i: 1 }),
                    op: InfixOp::Add,
                    rhs: Box::new(Expression::Int { i: 2 }),
                },
            }],
        );
    }

    #[test]
    fn breaks() {
        assert_ast("<?php break;", &[Statement::Break { num: None }]);

        assert_ast(
            "<?php break 2;",
            &[Statement::Break {
                num: Some(Expression::Int { i: 2 }),
            }],
        );
    }

    #[test]
    fn continues() {
        assert_ast("<?php continue;", &[Statement::Continue { num: None }]);

        assert_ast(
            "<?php continue 2;",
            &[Statement::Continue {
                num: Some(Expression::Int { i: 2 }),
            }],
        );
    }

    #[test]
    fn math_precedence() {
        assert_ast(
            "<?php 1 + 2 * 3 / 4 - 5;",
            &[expr!(Expression::Infix {
                lhs: Box::new(Expression::Infix {
                    lhs: Box::new(Expression::Int { i: 1 }),
                    op: InfixOp::Add,
                    rhs: Box::new(Expression::Infix {
                        lhs: Box::new(Expression::Infix {
                            lhs: Box::new(Expression::Int { i: 2 }),
                            op: InfixOp::Mul,
                            rhs: Box::new(Expression::Int { i: 3 })
                        }),
                        op: InfixOp::Div,
                        rhs: Box::new(Expression::Int { i: 4 })
                    })
                }),
                op: InfixOp::Sub,
                rhs: Box::new(Expression::Int { i: 5 })
            })],
        );
    }

    #[test]
    fn property_fetch() {
        assert_ast(
            "<?php $foo->bar; $foo->bar->baz;",
            &[
                expr!(Expression::PropertyFetch {
                    target: Box::new(Expression::Variable { name: "foo".into() }),
                    property: Box::new(Expression::Identifier { name: "bar".into() })
                }),
                expr!(Expression::PropertyFetch {
                    target: Box::new(Expression::PropertyFetch {
                        target: Box::new(Expression::Variable { name: "foo".into() }),
                        property: Box::new(Expression::Identifier { name: "bar".into() })
                    }),
                    property: Box::new(Expression::Identifier { name: "baz".into() })
                }),
            ],
        );
    }

    #[test]
    fn method_calls() {
        assert_ast(
            "<?php $foo->bar();",
            &[expr!(Expression::MethodCall {
                target: Box::new(Expression::Variable { name: "foo".into() }),
                method: Box::new(Expression::Identifier { name: "bar".into() }),
                args: vec![]
            })],
        );

        assert_ast(
            "<?php $foo->bar()->baz();",
            &[expr!(Expression::MethodCall {
                target: Box::new(Expression::MethodCall {
                    target: Box::new(Expression::Variable { name: "foo".into() }),
                    method: Box::new(Expression::Identifier { name: "bar".into() }),
                    args: vec![]
                }),
                method: Box::new(Expression::Identifier { name: "baz".into() }),
                args: vec![]
            })],
        );

        assert_ast(
            "<?php $foo->bar()();",
            &[expr!(Expression::Call {
                target: Box::new(Expression::MethodCall {
                    target: Box::new(Expression::Variable { name: "foo".into() }),
                    method: Box::new(Expression::Identifier { name: "bar".into() }),
                    args: vec![]
                }),
                args: vec![]
            })],
        );
    }

    #[test]
    fn concat() {
        assert_ast(
            "<?php 'foo' . 'bar' . 'baz';",
            &[expr!(Expression::Infix {
                lhs: Box::new(Expression::Infix {
                    lhs: Box::new(Expression::ConstantString {
                        value: "foo".into()
                    }),
                    op: InfixOp::Concat,
                    rhs: Box::new(Expression::ConstantString {
                        value: "bar".into()
                    }),
                }),
                op: InfixOp::Concat,
                rhs: Box::new(Expression::ConstantString {
                    value: "baz".into()
                })
            })],
        );
    }

    #[test]
    fn empty_fn() {
        assert_ast("<?php function foo() {}", &[function!("foo", &[], &[])]);
    }

    #[test]
    fn empty_fn_with_params() {
        assert_ast(
            "<?php function foo($n) {}",
            &[function!("foo", &["n"], &[])],
        );

        assert_ast(
            "<?php function foo($n, $m) {}",
            &[function!("foo", &["n", "m"], &[])],
        );
    }

    #[test]
    fn fib() {
        assert_ast(
            "\
        <?php

        function fib($n) {
            if ($n < 2) {
                return $n;
            }

            return fib($n - 1) + fib($n - 2);
        }",
            &[function!(
                "fib",
                &["n"],
                &[
                    Statement::If {
                        condition: Expression::Infix {
                            lhs: Box::new(Expression::Variable { name: "n".into() }),
                            op: InfixOp::LessThan,
                            rhs: Box::new(Expression::Int { i: 2 }),
                        },
                        then: vec![Statement::Return {
                            value: Some(Expression::Variable { name: "n".into() })
                        }],
                        else_ifs: vec![],
                        r#else: None
                    },
                    Statement::Return {
                        value: Some(Expression::Infix {
                            lhs: Box::new(Expression::Call {
                                target: Box::new(Expression::Identifier { name: "fib".into() }),
                                args: vec![Arg {
                                    name: None,
                                    value: Expression::Infix {
                                        lhs: Box::new(Expression::Variable { name: "n".into() }),
                                        op: InfixOp::Sub,
                                        rhs: Box::new(Expression::Int { i: 1 }),
                                    },
                                    unpack: false,
                                }]
                            }),
                            op: InfixOp::Add,
                            rhs: Box::new(Expression::Call {
                                target: Box::new(Expression::Identifier { name: "fib".into() }),
                                args: vec![Arg {
                                    name: None,
                                    value: Expression::Infix {
                                        lhs: Box::new(Expression::Variable { name: "n".into() }),
                                        op: InfixOp::Sub,
                                        rhs: Box::new(Expression::Int { i: 2 }),
                                    },
                                    unpack: false,
                                }]
                            }),
                        })
                    }
                ]
            )],
        );
    }

    #[test]
    fn one_liner_if_statement() {
        assert_ast(
            "<?php if($foo) return $foo;",
            &[Statement::If {
                condition: Expression::Variable { name: "foo".into() },
                then: vec![Statement::Return {
                    value: Some(Expression::Variable { name: "foo".into() }),
                }],
                else_ifs: vec![],
                r#else: None,
            }],
        );
    }

    #[test]
    fn if_else_statement() {
        assert_ast(
            "<?php if($foo) { return $foo; } else { return $foo; }",
            &[Statement::If {
                condition: Expression::Variable { name: "foo".into() },
                then: vec![Statement::Return {
                    value: Some(Expression::Variable { name: "foo".into() }),
                }],
                else_ifs: vec![],
                r#else: Some(vec![Statement::Return {
                    value: Some(Expression::Variable { name: "foo".into() }),
                }]),
            }],
        );
    }

    #[test]
    fn if_elseif_else_statement() {
        assert_ast(
            "<?php if($foo) { return $foo; } elseif($foo) { return $foo; } else { return $foo; }",
            &[Statement::If {
                condition: Expression::Variable { name: "foo".into() },
                then: vec![Statement::Return {
                    value: Some(Expression::Variable { name: "foo".into() }),
                }],
                else_ifs: vec![ElseIf {
                    condition: Expression::Variable { name: "foo".into() },
                    body: vec![Statement::Return {
                        value: Some(Expression::Variable { name: "foo".into() }),
                    }],
                }],
                r#else: Some(vec![Statement::Return {
                    value: Some(Expression::Variable { name: "foo".into() }),
                }]),
            }],
        );
    }

    #[test]
    fn echo() {
        assert_ast(
            "<?php echo 1;",
            &[Statement::Echo {
                values: vec![Expression::Int { i: 1 }],
            }],
        );
    }

    #[test]
    fn empty_class() {
        assert_ast("<?php class Foo {}", &[class!("Foo")]);
    }

    #[test]
    fn class_with_basic_method() {
        assert_ast(
            "\
        <?php

        class Foo {
            function bar() {
                echo 1;
            }
        }
        ",
            &[class!(
                "Foo",
                &[method!(
                    "bar",
                    &[],
                    &[],
                    &[Statement::Echo {
                        values: vec![Expression::Int { i: 1 },]
                    }]
                )]
            )],
        );
    }

    #[test]
    fn class_with_extends() {
        assert_ast(
            "\
        <?php

        class Foo extends Bar {}
        ",
            &[class!("Foo", Some("Bar".as_bytes().into()), &[], &[])],
        );
    }

    #[test]
    fn class_with_implements() {
        assert_ast(
            "\
        <?php

        class Foo implements Bar, Baz {}
        ",
            &[class!(
                "Foo",
                None,
                &["Bar".as_bytes().into(), "Baz".as_bytes().into()],
                &[]
            )],
        );
    }

    #[test]
    fn plain_typestrings_test() {
        assert_ast(
            "<?php function foo(string $b) {}",
            &[Statement::Function {
                name: "foo".as_bytes().into(),
                params: vec![Param {
                    name: Expression::Variable { name: "b".into() },
                    r#type: Some(Type::Plain("string".into())),
                    variadic: false,
                    default: None,
                    flag: None,
                    by_ref: false,
                }],
                body: vec![],
                return_type: None,
                by_ref: false,
            }],
        );
    }

    #[test]
    fn variadic_params() {
        assert_ast(
            "<?php function foo(...$bar) {}",
            &[Statement::Function {
                name: "foo".as_bytes().into(),
                params: vec![Param {
                    name: Expression::Variable { name: "bar".into() },
                    r#type: None,
                    variadic: true,
                    default: None,
                    flag: None,
                    by_ref: false,
                }],
                body: vec![],
                return_type: None,
                by_ref: false,
            }],
        );

        assert_ast(
            "<?php function foo(string ...$bar) {}",
            &[Statement::Function {
                name: "foo".as_bytes().into(),
                params: vec![Param {
                    name: Expression::Variable { name: "bar".into() },
                    r#type: Some(Type::Plain("string".into())),
                    variadic: true,
                    default: None,
                    flag: None,
                    by_ref: false,
                }],
                body: vec![],
                return_type: None,
                by_ref: false,
            }],
        );

        assert_ast(
            "<?php function foo($bar, $baz, ...$car) {}",
            &[Statement::Function {
                name: "foo".as_bytes().into(),
                params: vec![
                    Param {
                        name: Expression::Variable { name: "bar".into() },
                        r#type: None,
                        variadic: false,
                        default: None,
                        flag: None,
                        by_ref: false,
                    },
                    Param {
                        name: Expression::Variable { name: "baz".into() },
                        r#type: None,
                        variadic: false,
                        default: None,
                        flag: None,
                        by_ref: false,
                    },
                    Param {
                        name: Expression::Variable { name: "car".into() },
                        r#type: None,
                        variadic: true,
                        default: None,
                        flag: None,
                        by_ref: false,
                    },
                ],
                body: vec![],
                return_type: None,
                by_ref: false,
            }],
        );
    }

    #[test]
    fn nullable_typestrings_test() {
        assert_ast(
            "<?php function foo(?string $b) {}",
            &[Statement::Function {
                name: "foo".as_bytes().into(),
                params: vec![Param {
                    name: Expression::Variable { name: "b".into() },
                    r#type: Some(Type::Nullable("string".into())),
                    variadic: false,
                    default: None,
                    flag: None,
                    by_ref: false,
                }],
                body: vec![],
                return_type: None,
                by_ref: false,
            }],
        );
    }

    #[test]
    fn union_typestrings_test() {
        assert_ast(
            "<?php function foo(int|float $b) {}",
            &[Statement::Function {
                name: "foo".as_bytes().into(),
                params: vec![Param {
                    name: Expression::Variable { name: "b".into() },
                    r#type: Some(Type::Union(vec!["int".into(), "float".into()])),
                    variadic: false,
                    default: None,
                    flag: None,
                    by_ref: false,
                }],
                body: vec![],
                return_type: None,
                by_ref: false,
            }],
        );

        assert_ast(
            "<?php function foo(string|int|float $b) {}",
            &[Statement::Function {
                name: "foo".as_bytes().into(),
                params: vec![Param {
                    name: Expression::Variable { name: "b".into() },
                    r#type: Some(Type::Union(vec![
                        "string".into(),
                        "int".into(),
                        "float".into(),
                    ])),
                    variadic: false,
                    default: None,
                    flag: None,
                    by_ref: false,
                }],
                body: vec![],
                return_type: None,
                by_ref: false,
            }],
        );
    }

    #[test]
    fn intersection_typestrings_test() {
        assert_ast(
            "<?php function foo(Foo&Bar $b) {}",
            &[Statement::Function {
                name: "foo".as_bytes().into(),
                params: vec![Param {
                    name: Expression::Variable { name: "b".into() },
                    r#type: Some(Type::Intersection(vec!["Foo".into(), "Bar".into()])),
                    variadic: false,
                    default: None,
                    flag: None,
                    by_ref: false,
                }],
                body: vec![],
                return_type: None,
                by_ref: false,
            }],
        );

        assert_ast(
            "<?php function foo(Foo&Bar&Baz $b) {}",
            &[Statement::Function {
                name: "foo".as_bytes().into(),
                params: vec![Param {
                    name: Expression::Variable { name: "b".into() },
                    r#type: Some(Type::Intersection(vec![
                        "Foo".into(),
                        "Bar".into(),
                        "Baz".into(),
                    ])),
                    variadic: false,
                    default: None,
                    flag: None,
                    by_ref: false,
                }],
                body: vec![],
                return_type: None,
                by_ref: false,
            }],
        );
    }

    #[test]
    fn function_return_types() {
        assert_ast(
            "<?php function foo(): string {}",
            &[Statement::Function {
                name: "foo".as_bytes().into(),
                params: vec![],
                body: vec![],
                return_type: Some(Type::Plain("string".into())),
                by_ref: false,
            }],
        );

        assert_ast(
            "<?php function foo(): void {}",
            &[Statement::Function {
                name: "foo".as_bytes().into(),
                params: vec![],
                body: vec![],
                return_type: Some(Type::Void),
                by_ref: false,
            }],
        );
    }

    #[test]
    fn new_anon_class() {
        assert_ast(
            "<?php new class{};",
            &[expr!(Expression::New {
                target: Box::new(Expression::AnonymousClass {
                    extends: None,
                    implements: vec![],
                    body: vec![]
                }),
                args: vec![],
            })],
        );

        assert_ast(
            "<?php new class(1, 2) {};",
            &[expr!(Expression::New {
                target: Box::new(Expression::AnonymousClass {
                    extends: None,
                    implements: vec![],
                    body: vec![]
                }),
                args: vec![
                    Arg {
                        name: None,
                        value: Expression::Int { i: 1 },
                        unpack: false,
                    },
                    Arg {
                        name: None,
                        value: Expression::Int { i: 2 },
                        unpack: false,
                    },
                ],
            })],
        );

        assert_ast(
            "<?php new class extends Foo {};",
            &[expr!(Expression::New {
                target: Box::new(Expression::AnonymousClass {
                    extends: Some(Identifier::from("Foo".as_bytes())),
                    implements: vec![],
                    body: vec![]
                }),
                args: vec![]
            })],
        );

        assert_ast(
            "<?php new class implements Foo, Bar {};",
            &[expr!(Expression::New {
                target: Box::new(Expression::AnonymousClass {
                    extends: None,
                    implements: vec![
                        Identifier::from("Foo".as_bytes()),
                        Identifier::from("Bar".as_bytes()),
                    ],
                    body: vec![]
                }),
                args: vec![]
            })],
        );

        assert_ast(
            "<?php new class {
            public function foo() {}
        };",
            &[expr!(Expression::New {
                target: Box::new(Expression::AnonymousClass {
                    extends: None,
                    implements: vec![],
                    body: vec![Statement::Method {
                        name: "foo".as_bytes().into(),
                        params: vec![],
                        body: vec![],
                        return_type: None,
                        flags: vec![MethodFlag::Public,],
                        by_ref: false,
                    }]
                }),
                args: vec![]
            })],
        );
    }

    #[test]
    fn foreach() {
        assert_ast(
            "<?php foreach ($foo as $bar) {}",
            &[Statement::Foreach {
                expr: Expression::Variable { name: "foo".into() },
                by_ref: false,
                key_var: None,
                value_var: Expression::Variable { name: "bar".into() },
                body: vec![],
            }],
        );

        assert_ast(
            "<?php foreach ($foo as $bar => $baz) {}",
            &[Statement::Foreach {
                expr: Expression::Variable { name: "foo".into() },
                by_ref: false,
                key_var: Some(Expression::Variable { name: "bar".into() }),
                value_var: Expression::Variable { name: "baz".into() },
                body: vec![],
            }],
        );

        assert_ast(
            "<?php foreach ($foo as [$baz, $car]) {}",
            &[Statement::Foreach {
                expr: Expression::Variable { name: "foo".into() },
                by_ref: false,
                key_var: None,
                value_var: Expression::Array {
                    items: vec![
                        ArrayItem {
                            key: None,
                            value: Expression::Variable { name: "baz".into() },
                        },
                        ArrayItem {
                            key: None,
                            value: Expression::Variable { name: "car".into() },
                        },
                    ],
                },
                body: vec![],
            }],
        );
    }

    #[test]
    fn block() {
        assert_ast("<?php {}", &[Statement::Block { body: vec![] }]);
        assert_ast(
            "<?php { $a; }",
            &[Statement::Block {
                body: vec![Statement::Expression {
                    expr: Expression::Variable { name: "a".into() },
                }],
            }],
        );
    }

    #[test]
    fn noop() {
        assert_ast("<?php ;", &[Statement::Noop]);
    }

    #[test]
    fn comment_at_end_of_class() {
        assert_ast(
            "<?php
        class MyClass {
            protected $a;
            // my comment
        }",
            &[Statement::Class {
                name: "MyClass".as_bytes().into(),
                extends: None,
                implements: vec![],
                body: vec![Statement::Property {
                    var: "a".into(),
                    value: None,
                    r#type: None,
                    flags: vec![PropertyFlag::Protected],
                }],
                flag: None,
            }],
        );
    }

    #[test]
    fn do_while() {
        assert_ast(
            "<?php do { } while ($a);",
            &[Statement::DoWhile {
                condition: Expression::Variable { name: "a".into() },
                body: vec![],
            }],
        );

        assert_ast(
            "<?php
        do {
            echo 'Hi!';
        } while (true);
        ",
            &[Statement::DoWhile {
                condition: Expression::Bool { value: true },
                body: vec![Statement::Echo {
                    values: vec![Expression::ConstantString {
                        value: "Hi!".into(),
                    }],
                }],
            }],
        )
    }

    #[test]
    fn close_tag_followed_by_content() {
        assert_ast(
            "<?php ?> <html>",
            &[Statement::InlineHtml(" <html>".into())],
        );
    }

    #[test]
    fn error_suppress() {
        assert_ast(
            "<?php @hello();",
            &[expr!(Expression::ErrorSuppress {
                expr: Box::new(Expression::Call {
                    target: Box::new(Expression::Identifier {
                        name: "hello".into()
                    }),
                    args: vec![],
                }),
            })],
        );
    }

    #[test]
    fn nullsafe_operator() {
        assert_ast(
            "<?php $a?->b;",
            &[expr!(Expression::NullsafePropertyFetch {
                target: Box::new(Expression::Variable { name: "a".into() }),
                property: Box::new(Expression::Identifier { name: "b".into() })
            })],
        );
    }

    #[test]
    fn try_catch() {
        assert_ast(
            "<?php try {} catch (Exception $e) {}",
            &[Statement::Try {
                body: vec![],
                catches: vec![Catch {
                    types: vec!["Exception".as_bytes().into()],
                    var: Some(Expression::Variable { name: "e".into() }),
                    body: vec![],
                }],
                finally: None,
            }],
        );
    }

    #[test]
    fn try_catch_no_variable() {
        assert_ast(
            "<?php try {} catch (Exception) {}",
            &[Statement::Try {
                body: vec![],
                catches: vec![Catch {
                    types: vec!["Exception".as_bytes().into()],
                    var: None,
                    body: vec![],
                }],
                finally: None,
            }],
        );
    }

    #[test]
    fn try_catch_multiple_catches() {
        assert_ast(
            "<?php try {} catch (Exception $e) {} catch (CustomException $e) {}",
            &[Statement::Try {
                body: vec![],
                catches: vec![
                    Catch {
                        types: vec!["Exception".as_bytes().into()],
                        var: Some(Expression::Variable { name: "e".into() }),
                        body: vec![],
                    },
                    Catch {
                        types: vec!["CustomException".as_bytes().into()],
                        var: Some(Expression::Variable { name: "e".into() }),
                        body: vec![],
                    },
                ],
                finally: None,
            }],
        );
    }

    #[test]
    fn try_catch_finally() {
        assert_ast(
            "<?php try {} catch (Exception $e) {} finally {}",
            &[Statement::Try {
                body: vec![],
                catches: vec![Catch {
                    types: vec!["Exception".as_bytes().into()],
                    var: Some(Expression::Variable { name: "e".into() }),
                    body: vec![],
                }],
                finally: Some(vec![]),
            }],
        );
    }

    #[test]
    fn try_finally_no_catch() {
        assert_ast(
            "<?php try {} finally {}",
            &[Statement::Try {
                body: vec![],
                catches: vec![],
                finally: Some(vec![]),
            }],
        );
    }

    #[test]
    fn top_level_constant() {
        assert_ast(
            "<?php const FOO = 1;",
            &[Statement::Constant {
                constants: vec![Constant {
                    name: "FOO".as_bytes().into(),
                    value: Expression::Int { i: 1 },
                }],
            }],
        );
    }

    #[test]
    fn top_level_constant_multiple() {
        assert_ast(
            "<?php const FOO = 1, BAR = 2;",
            &[Statement::Constant {
                constants: vec![
                    Constant {
                        name: "FOO".as_bytes().into(),
                        value: Expression::Int { i: 1 },
                    },
                    Constant {
                        name: "BAR".as_bytes().into(),
                        value: Expression::Int { i: 2 },
                    },
                ],
            }],
        );
    }

    #[test]
    fn global_statements() {
        assert_ast(
            "<?php global $a;",
            &[Statement::Global {
                vars: vec!["a".as_bytes().into()],
            }],
        );
    }

    #[test]
    fn multiple_global_vars_in_statement() {
        assert_ast(
            "<?php global $a, $b;",
            &[Statement::Global {
                vars: vec!["a".as_bytes().into(), "b".as_bytes().into()],
            }],
        );
    }

    #[test]
    fn basic_declare() {
        assert_ast(
            "<?php declare(A='B');",
            &[Statement::Declare {
                declares: vec![DeclareItem {
                    key: "A".as_bytes().into(),
                    value: Expression::ConstantString { value: "B".into() },
                }],
                body: vec![],
            }],
        );
    }

    #[test]
    fn multiple_declares_in_single_statement() {
        assert_ast(
            "<?php declare(A='B', C='D');",
            &[Statement::Declare {
                declares: vec![
                    DeclareItem {
                        key: "A".as_bytes().into(),
                        value: Expression::ConstantString { value: "B".into() },
                    },
                    DeclareItem {
                        key: "C".as_bytes().into(),
                        value: Expression::ConstantString { value: "D".into() },
                    },
                ],
                body: vec![],
            }],
        );
    }

    #[test]
    fn declare_block() {
        assert_ast(
            "<?php declare(A='B') { echo 'Hello, world!'; }",
            &[Statement::Declare {
                declares: vec![DeclareItem {
                    key: "A".as_bytes().into(),
                    value: Expression::ConstantString { value: "B".into() },
                }],
                body: vec![Statement::Echo {
                    values: vec![Expression::ConstantString {
                        value: "Hello, world!".into(),
                    }],
                }],
            }],
        );
    }

    #[test]
    fn array_empty_items() {
        assert_ast(
            "<?php [1, 2, , 4];",
            &[expr!(Expression::Array {
                items: vec![
                    ArrayItem {
                        key: None,
                        value: Expression::Int { i: 1 },
                    },
                    ArrayItem {
                        key: None,
                        value: Expression::Int { i: 2 },
                    },
                    ArrayItem {
                        key: None,
                        value: Expression::Empty,
                    },
                    ArrayItem {
                        key: None,
                        value: Expression::Int { i: 4 },
                    },
                ]
            })],
        )
    }

    #[test]
    fn switch() {
        assert_ast(
            "<?php
        switch ($a) {
            case 0:
                break;
            case 1;
            default:
        }
        ",
            &[Statement::Switch {
                condition: Expression::Variable { name: "a".into() },
                cases: vec![
                    Case {
                        condition: Some(Expression::Int { i: 0 }),
                        body: vec![Statement::Break { num: None }],
                    },
                    Case {
                        condition: Some(Expression::Int { i: 1 }),
                        body: vec![],
                    },
                    Case {
                        condition: None,
                        body: vec![],
                    },
                ],
            }],
        )
    }

    #[test]
    fn readonly_classes() {
        assert_ast(
            "<?php readonly class Foo {}",
            &[Statement::Class {
                name: "Foo".as_bytes().into(),
                extends: None,
                implements: vec![],
                body: vec![],
                flag: Some(ClassFlag::Readonly),
            }],
        );
    }

    #[test]
    fn readonly_class_props() {
        assert_ast(
            "<?php class Foo { public readonly $bar; }",
            &[Statement::Class {
                name: "Foo".as_bytes().into(),
                extends: None,
                implements: vec![],
                body: vec![Statement::Property {
                    var: "bar".as_bytes().into(),
                    value: None,
                    r#type: None,
                    flags: vec![PropertyFlag::Public, PropertyFlag::Readonly],
                }],
                flag: None,
            }],
        );
    }

    #[test]
    fn backed_enum_without_values() {
        assert_ast(
            "<?php enum Foo: string {
                case Bar;
                case Baz = 'Baz';
            }",
            &[Statement::Enum {
                name: "Foo".as_bytes().into(),
                implements: vec![],
                backed_type: Some(BackedEnumType::String),
                body: vec![
                    Statement::EnumCase {
                        name: "Bar".as_bytes().into(),
                        value: None,
                    },
                    Statement::EnumCase {
                        name: "Baz".as_bytes().into(),
                        value: Some(Expression::ConstantString {
                            value: "Baz".into(),
                        }),
                    },
                ],
            }],
        );
    }

    #[test]
    fn basic_namespace() {
        assert_ast(
            "<?php namespace Foo;",
            &[Statement::Namespace {
                name: Some("Foo".as_bytes().into()),
                body: vec![],
            }],
        );
    }

    #[test]
    fn basic_braced_namespace() {
        assert_ast(
            "<?php namespace Foo {}",
            &[Statement::Namespace {
                name: Some("Foo".as_bytes().into()),
                body: vec![],
            }],
        );
    }

    #[test]
    fn braced_global_namespace() {
        assert_ast(
            "<?php
        namespace {
            function globalFunc() {}
        }
        ",
            &[Statement::Namespace {
                name: None,
                body: vec![Statement::Function {
                    name: "globalFunc".as_bytes().into(),
                    params: vec![],
                    body: vec![],
                    return_type: None,
                    by_ref: false,
                }],
            }],
        );
    }

    #[test]
    fn basic_closures() {
        assert_ast(
            "<?php function () {};",
            &[expr!(Expression::Closure {
                params: vec![],
                uses: vec![],
                return_type: None,
                body: vec![],
                r#static: false,
                by_ref: false,
            })],
        );
    }

    #[test]
    fn arrow_functions() {
        assert_ast(
            "<?php fn () => null;",
            &[expr!(Expression::ArrowFunction {
                params: vec![],
                return_type: None,
                expr: Box::new(Expression::Null),
                by_ref: false,
                r#static: false,
            })],
        );
    }

    #[test]
    fn static_closures() {
        assert_ast(
            "<?php static function () {};",
            &[expr!(Expression::Closure {
                params: vec![],
                uses: vec![],
                return_type: None,
                body: vec![],
                r#static: true,
                by_ref: false,
            })],
        );
    }

    #[test]
    fn simple_foreach_reference() {
        assert_ast(
            "<?php foreach ($a as &$b) {}",
            &[Statement::Foreach {
                expr: Expression::Variable { name: "a".into() },
                by_ref: true,
                key_var: None,
                value_var: Expression::Variable { name: "b".into() },
                body: vec![],
            }],
        );
    }

    #[test]
    fn key_value_foreach_reference() {
        assert_ast(
            "<?php foreach ($a as $b => &$c) {}",
            &[Statement::Foreach {
                expr: Expression::Variable { name: "a".into() },
                by_ref: true,
                key_var: Some(Expression::Variable { name: "b".into() }),
                value_var: Expression::Variable { name: "c".into() },
                body: vec![],
            }],
        );
    }

    #[test]
    fn function_with_ref_param() {
        assert_ast(
            "<?php function a(&$b) {}",
            &[Statement::Function {
                name: "a".into(),
                params: vec![Param {
                    name: Expression::Variable { name: "b".into() },
                    r#type: None,
                    variadic: false,
                    flag: None,
                    default: None,
                    by_ref: true,
                }],
                body: vec![],
                return_type: None,
                by_ref: false,
            }],
        );
    }

    #[test]
    fn arrow_function_with_ref_param() {
        assert_ast(
            "<?php fn (&$b) => null;",
            &[expr!(Expression::ArrowFunction {
                params: vec![Param {
                    name: Expression::Variable { name: "b".into() },
                    r#type: None,
                    variadic: false,
                    flag: None,
                    default: None,
                    by_ref: true,
                }],
                return_type: None,
                expr: Box::new(Expression::Null),
                by_ref: false,
                r#static: false,
            })],
        );
    }

    #[test]
    fn function_returning_ref() {
        assert_ast(
            "<?php function &a($b) {}",
            &[Statement::Function {
                name: "a".into(),
                params: vec![Param {
                    name: Expression::Variable { name: "b".into() },
                    r#type: None,
                    variadic: false,
                    flag: None,
                    default: None,
                    by_ref: false,
                }],
                body: vec![],
                return_type: None,
                by_ref: true,
            }],
        );
    }

    #[test]
    fn closure_returning_ref() {
        assert_ast(
            "<?php function &() {};",
            &[expr!(Expression::Closure {
                params: vec![],
                body: vec![],
                return_type: None,
                r#static: false,
                uses: vec![],
                by_ref: true,
            })],
        );
    }

    #[test]
    fn static_closures_returning_by_ref() {
        assert_ast(
            "<?php static function &() {};",
            &[expr!(Expression::Closure {
                params: vec![],
                body: vec![],
                return_type: None,
                r#static: true,
                uses: vec![],
                by_ref: true,
            })],
        );
    }

    #[test]
    fn arrow_functions_returning_by_ref() {
        assert_ast(
            "<?php fn &() => null;",
            &[expr!(Expression::ArrowFunction {
                params: vec![],
                expr: Box::new(Expression::Null),
                return_type: None,
                by_ref: true,
                r#static: false,
            })],
        );
    }

    #[test]
    fn static_arrow_functions() {
        assert_ast(
            "<?php static fn () => null;",
            &[expr!(Expression::ArrowFunction {
                params: vec![],
                expr: Box::new(Expression::Null),
                return_type: None,
                by_ref: false,
                r#static: true,
            })],
        );
    }

    #[test]
    fn static_arrow_functions_returning_by_ref() {
        assert_ast(
            "<?php static fn &() => null;",
            &[expr!(Expression::ArrowFunction {
                params: vec![],
                expr: Box::new(Expression::Null),
                return_type: None,
                by_ref: true,
                r#static: true,
            })],
        );
    }

    #[test]
    fn first_class_callables() {
        assert_ast("<?php foo(...);", &[
            expr!(Expression::Call { target: Box::new(Expression::Identifier { name: "foo".into() }), args: vec![
                Arg {
                    name: None,
                    unpack: false,
                    value: Expression::VariadicPlaceholder
                }
            ] })
        ]);
    }

    #[test]
    fn first_class_callable_method() {
        assert_ast("<?php $this->foo(...);", &[
            expr!(Expression::MethodCall { target: Box::new(Expression::Variable { name: "this".into() }), method: Box::new(Expression::Identifier { name: "foo".into() }), args: vec![
                Arg {
                    name: None,
                    unpack: false,
                    value: Expression::VariadicPlaceholder
                }
            ] })
        ]);
    }

    fn assert_ast(source: &str, expected: &[Statement]) {
        let mut lexer = Lexer::new(None);
        let tokens = lexer.tokenize(source).unwrap();

        let mut parser = Parser::new(None);
        let ast = parser.parse(tokens);

        if ast.is_err() {
            panic!("{}", ast.err().unwrap());
        } else {
            assert_eq!(ast.unwrap(), expected);
        }
    }
}
