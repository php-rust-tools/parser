use crate::{
    ast::{
        Arg, ArrayItem, BackedEnumType, ClassFlag, ClosureUse, ElseIf, IncludeKind, MagicConst,
        MethodFlag, StaticVar, Use, UseKind,
    },
    Block, Case, Catch, Expression, Identifier, MatchArm, Program, Statement, Type,
};
use core::panic;
use std::{fmt::Display, vec::IntoIter};
use trunk_lexer::{Span, Token, TokenKind};

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

        Ok(match id.as_str() {
            "void" => Type::Void,
            _ => Type::Plain(id),
        })
    }

    fn statement(&mut self) -> ParseResult<Statement> {
        self.skip_comments();

        let statement = match &self.current.kind {
            TokenKind::Global => {
                self.next();

                let mut vars = vec![];
                while self.current.kind != TokenKind::SemiColon {
                    vars.push(self.var()?.into());

                    self.optional_comma()?;
                }

                self.semi()?;
                Statement::Global { vars }
            },
            TokenKind::Static if matches!(self.peek.kind, TokenKind::Variable(_)) => {
                self.next();

                let mut vars = vec![];

                while self.current.kind != TokenKind::SemiColon {
                    let var = Expression::Variable { name: self.var()? };
                    let mut default = None;

                    if self.current.kind == TokenKind::Equals {
                        expect!(self, TokenKind::Equals, "expected =");
                        default = Some(self.expression(0)?);
                    }

                    self.optional_comma()?;

                    vars.push(StaticVar { var, default })
                }

                self.semi()?;

                Statement::Static { vars }
            }
            TokenKind::InlineHtml(html) => {
                let s = Statement::InlineHtml(html.to_string());
                self.next();
                s
            }
            TokenKind::Comment(comment) => {
                let s = Statement::Comment {
                    comment: comment.to_string(),
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
                let condition = self.expression(0)?;
                self.rparen()?;
                self.semi()?;

                Statement::DoWhile { condition, body }
            }
            TokenKind::While => {
                self.next();
                self.lparen()?;

                let condition = self.expression(0)?;

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

                let path = self.expression(0)?;

                self.semi()?;

                Statement::Include { kind, path }
            }
            TokenKind::For => {
                self.next();

                self.lparen()?;

                let mut init = None;
                if self.current.kind != TokenKind::SemiColon {
                    init = Some(self.expression(0)?);
                }
                self.semi()?;

                let mut condition = None;
                if self.current.kind != TokenKind::SemiColon {
                    condition = Some(self.expression(0)?);
                }
                self.semi()?;

                let mut r#loop = None;
                if self.current.kind != TokenKind::RightParen {
                    r#loop = Some(self.expression(0)?);
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

                let expr = self.expression(0)?;

                expect!(self, TokenKind::As, "expected 'as'");

                let mut by_ref = self.current.kind == TokenKind::Ampersand;
                if by_ref {
                    self.next();
                }

                let mut key_var = None;
                let mut value_var = self.expression(0)?;

                if self.current.kind == TokenKind::DoubleArrow {
                    self.next();

                    key_var = Some(value_var.clone());

                    by_ref = self.current.kind == TokenKind::Ampersand;
                    if by_ref {
                        self.next();
                    }

                    value_var = self.expression(0)?;
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
            TokenKind::Trait => {
                self.next();

                let name = self.ident()?;

                self.lbrace()?;

                let mut body = Block::new();
                while self.current.kind != TokenKind::RightBrace {
                    match self.class_statement()? {
                        Statement::Constant { .. } => {
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

                let mut is_backed = false;
                let backed_type: Option<BackedEnumType> = if self.current.kind == TokenKind::Colon {
                    expect!(self, TokenKind::Colon, "expected :");

                    match self.current.kind.clone() {
                        TokenKind::Identifier(s) if s == *"string" || s == *"int" => {
                            self.next();

                            is_backed = true;

                            Some(match s.as_str() {
                                "string" => BackedEnumType::String,
                                "int" => BackedEnumType::Int,
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

                            if is_backed {
                                expect!(self, TokenKind::Equals, "expected =");

                                value = Some(self.expression(0)?);
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

                let condition = self.expression(0)?;

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

                            let condition = self.expression(0)?;

                            expect!(self, TokenKind::Colon, "expected :");

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

                            expect!(self, TokenKind::Colon, "expected :");

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

                let name = self.name()?;

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
            }
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
                    values.push(self.expression(0)?);

                    self.optional_comma()?;
                }
                self.semi()?;
                Statement::Echo { values }
            }
            TokenKind::Continue => {
                self.next();

                let mut num = None;
                if self.current.kind != TokenKind::SemiColon {
                    num = Some(self.expression(0)?);
                }

                self.semi()?;

                Statement::Continue { num }
            }
            TokenKind::Break => {
                self.next();

                let mut num = None;
                if self.current.kind != TokenKind::SemiColon {
                    num = Some(self.expression(0)?);
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
                        value: self.expression(0).ok(),
                    };
                    self.semi()?;
                    ret
                }
            }
            TokenKind::Function if matches!(self.peek.kind, TokenKind::Identifier(_)) => {
                self.function()?
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
                        Some(self.expression(0)?)
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

                // TODO: Support defining multiple constants in one go.
                let name = self.ident()?;

                expect!(self, TokenKind::Equals, "expected =");

                let value = self.expression(0)?;

                self.semi()?;

                Statement::Constant {
                    name: name.into(),
                    value,
                    flags: vec![],
                }
            }
            _ => {
                let expr = self.expression(0)?;

                self.semi()?;

                Statement::Expression { expr }
            }
        };

        self.skip_comments();

        Ok(statement)
    }

    fn function(&mut self) -> ParseResult<Statement> {
        self.next();

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

                let value = self.expression(0)?;

                self.semi()?;

                Ok(Statement::Constant {
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

                    value = Some(self.expression(0)?);
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
            | TokenKind::Static => {
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

                        let value = self.expression(0)?;

                        self.semi()?;

                        Ok(Statement::Constant {
                            name: name.into(),
                            value,
                            flags: flags.into_iter().map(|f| f.into()).collect(),
                        })
                    }
                    TokenKind::Function => {
                        if flags.contains(&TokenKind::Abstract) {
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

                            Ok(Statement::Method {
                                name: name.into(),
                                params,
                                body: vec![],
                                return_type,
                                flags: flags.iter().map(|t| t.clone().into()).collect(),
                            })
                        } else {
                            match self.function()? {
                                Statement::Function {
                                    name,
                                    params,
                                    body,
                                    return_type,
                                } => Ok(Statement::Method {
                                    name,
                                    params,
                                    body,
                                    flags: flags.iter().map(|t| t.clone().into()).collect(),
                                    return_type,
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
                            value = Some(self.expression(0)?);
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
                            value = Some(self.expression(0)?);
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
                } => Ok(Statement::Method {
                    name,
                    params,
                    body,
                    flags: vec![],
                    return_type,
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

    fn expression(&mut self, bp: u8) -> Result<Expression, ParseError> {
        if self.is_eof() {
            return Err(ParseError::UnexpectedEndOfFile);
        }

        self.skip_comments();

        let mut lhs = match &self.current.kind {
            TokenKind::Throw => {
                self.next();

                let value = self.expression(0)?;

                Expression::Throw {
                    value: Box::new(value),
                }
            }
            TokenKind::Yield => {
                self.next();

                let value = self.expression(0)?;

                // FIXME: Check for presence of => here to allow yielding key and value.

                Expression::Yield {
                    value: Box::new(value),
                }
            }
            TokenKind::Clone => {
                self.next();

                let target = self.expression(0)?;

                Expression::Clone {
                    target: Box::new(target),
                }
            }
            TokenKind::Variable(v) => {
                let e = Expression::Variable {
                    name: v.to_string(),
                };
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
                let e = Expression::Identifier {
                    name: i.to_string(),
                };
                self.next();
                e
            }
            TokenKind::Static => {
                self.next();
                Expression::Static
            }
            TokenKind::ConstantString(s) => {
                let e = Expression::ConstantString {
                    value: s.to_string(),
                };
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

                let e = self.expression(0)?;

                self.rparen()?;

                e
            }
            TokenKind::Match => {
                self.next();
                self.lparen()?;

                let condition = Box::new(self.expression(0)?);

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

                        conditions.push(self.expression(0)?);

                        self.optional_comma()?;
                    }

                    expect!(self, TokenKind::DoubleArrow, "expected =>");

                    let body = self.expression(0)?;

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
                    let mut value = self.expression(0)?;

                    if self.current.kind == TokenKind::DoubleArrow {
                        self.next();

                        key = Some(value);
                        value = self.expression(0)?;
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
                    let mut key = None;
                    let mut value = self.expression(0)?;

                    if self.current.kind == TokenKind::DoubleArrow {
                        self.next();

                        key = Some(value);
                        value = self.expression(0)?;
                    }

                    items.push(ArrayItem { key, value });

                    self.optional_comma()?;

                    self.skip_comments();
                }

                self.rbracket()?;

                Expression::Array { items }
            }
            TokenKind::Function => {
                self.next();

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

                                match self.expression(0)? {
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
                            _ => match self.expression(0)? {
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
                }
            }
            TokenKind::Fn => {
                self.next();

                self.lparen()?;

                let params = self.param_list()?;

                self.rparen()?;

                let mut return_type = None;

                if self.current.kind == TokenKind::Colon || self.config.force_type_strings {
                    expect!(self, TokenKind::Colon, "expected :");

                    return_type = Some(self.type_string()?);
                }

                expect!(self, TokenKind::DoubleArrow, "expected =>");

                let value = self.expression(0)?;

                Expression::ArrowFunction {
                    params,
                    return_type,
                    expr: Box::new(value),
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

                            let value = self.expression(0)?;

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
                    self.expression(20)?
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

                        let value = self.expression(0)?;

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

                let rbp = prefix_binding_power(&op);
                let rhs = self.expression(rbp)?;

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
            return Ok(lhs);
        }

        self.skip_comments();

        loop {
            self.skip_comments();

            let kind = match &self.current {
                Token {
                    kind: TokenKind::SemiColon | TokenKind::Eof,
                    ..
                } => break,
                Token { kind, .. } => kind.clone(),
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
                match op {
                    TokenKind::Question => {
                        let then = self.expression(0)?;
                        expect!(self, TokenKind::Colon, "expected :");
                        let otherwise = self.expression(rbp)?;
                        lhs = Expression::Ternary {
                            condition: Box::new(lhs),
                            then: Some(Box::new(then)),
                            r#else: Box::new(otherwise),
                        }
                    }
                    TokenKind::QuestionColon => {
                        let r#else = self.expression(0)?;
                        lhs = Expression::Ternary {
                            condition: Box::new(lhs),
                            then: None,
                            r#else: Box::new(r#else),
                        }
                    }
                    _ => {
                        let rhs = self.expression(rbp)?;
                        lhs = infix(lhs, op, rhs);
                    }
                }

                continue;
            }

            break;
        }

        self.skip_comments();

        Ok(lhs)
    }

    fn postfix(&mut self, lhs: Expression, op: &TokenKind) -> Result<Expression, ParseError> {
        Ok(match op {
            TokenKind::Coalesce => {
                let rhs = self.expression(11)?;

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

                    let value = self.expression(0)?;

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
                    let index = self.expression(0)?;

                    expect!(self, TokenKind::RightBracket, "expected ]");

                    Expression::ArrayIndex {
                        array: Box::new(lhs),
                        index: Some(Box::new(index)),
                    }
                }
            }
            TokenKind::DoubleColon => match self.current.kind.clone() {
                TokenKind::Variable(_) => {
                    let var = self.expression(0)?;

                    Expression::StaticPropertyFetch {
                        target: Box::new(lhs),
                        property: Box::new(var),
                    }
                }
                _ => {
                    let ident = if self.current.kind == TokenKind::Class {
                        self.next();

                        String::from("class")
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

                            let value = self.expression(0)?;

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
                        let expr = self.expression(0)?;
                        self.rbrace()?;
                        expr
                    }
                    TokenKind::Variable(ref var) => {
                        let var = Expression::Variable {
                            name: var.to_string(),
                        };
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

                        let value = self.expression(0)?;

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
                } else {
                    if op == &TokenKind::NullsafeArrow {
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

fn prefix_binding_power(op: &TokenKind) -> u8 {
    match op {
        TokenKind::StringCast
        | TokenKind::ObjectCast
        | TokenKind::BoolCast
        | TokenKind::IntCast
        | TokenKind::DoubleCast => 101,
        TokenKind::At => 16,
        TokenKind::Minus => 99,
        TokenKind::Bang => 98,
        _ => unreachable!(),
    }
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

fn infix_binding_power(t: &TokenKind) -> Option<(u8, u8)> {
    Some(match t {
        TokenKind::Pow => (18, 19),
        TokenKind::Instanceof => (16, 17),
        TokenKind::Asterisk | TokenKind::Slash => (14, 15),
        TokenKind::Plus | TokenKind::Minus => (12, 13),
        TokenKind::Dot => (12, 12),
        TokenKind::LessThan
        | TokenKind::GreaterThan
        | TokenKind::LessThanEquals
        | TokenKind::GreaterThanEquals => (10, 11),
        TokenKind::DoubleEquals
        | TokenKind::TripleEquals
        | TokenKind::BangEquals
        | TokenKind::BangDoubleEquals => (8, 9),
        TokenKind::Question | TokenKind::QuestionColon => (6, 7),
        TokenKind::BooleanAnd => (4, 5),
        TokenKind::BooleanOr => (2, 3),
        TokenKind::Equals
        | TokenKind::PlusEquals
        | TokenKind::MinusEquals
        | TokenKind::DotEquals
        | TokenKind::CoalesceEqual
        | TokenKind::AsteriskEqual
        | TokenKind::SlashEquals => (0, 1),
        _ => return None,
    })
}

fn postfix_binding_power(t: &TokenKind) -> Option<u8> {
    Some(match t {
        TokenKind::Increment | TokenKind::Decrement => 77,
        TokenKind::LeftParen | TokenKind::LeftBracket => 19,
        TokenKind::Arrow | TokenKind::NullsafeArrow | TokenKind::DoubleColon => 18,
        TokenKind::Coalesce => 11,
        _ => return None,
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
        ast::{Arg, ArrayItem, ElseIf, IncludeKind, InfixOp, MethodFlag, PropertyFlag},
        Catch, Expression, Identifier, Param, Statement, Type,
    };
    use trunk_lexer::Lexer;

    macro_rules! function {
        ($name:literal, $params:expr, $body:expr) => {
            Statement::Function {
                name: $name.to_string().into(),
                params: $params
                    .to_vec()
                    .into_iter()
                    .map(|p: &str| Param::from(p))
                    .collect::<Vec<Param>>(),
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
                params: $params
                    .to_vec()
                    .into_iter()
                    .map(|p: &str| Param::from(p))
                    .collect::<Vec<Param>>(),
                flags: $flags.to_vec(),
                body: $body.to_vec(),
                return_type: None,
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
                lhs: Box::new(Expression::ConstantString {
                    value: "foo".into()
                }),
                op: InfixOp::Concat,
                rhs: Box::new(Expression::Infix {
                    lhs: Box::new(Expression::ConstantString {
                        value: "bar".into()
                    }),
                    op: InfixOp::Concat,
                    rhs: Box::new(Expression::ConstantString {
                        value: "baz".into()
                    }),
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
            &[class!("Foo", Some("Bar".to_string().into()), &[], &[])],
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
                &["Bar".to_string().into(), "Baz".to_string().into()],
                &[]
            )],
        );
    }

    #[test]
    fn plain_typestrings_test() {
        assert_ast(
            "<?php function foo(string $b) {}",
            &[Statement::Function {
                name: "foo".to_string().into(),
                params: vec![Param {
                    name: Expression::Variable { name: "b".into() },
                    r#type: Some(Type::Plain("string".into())),
                    variadic: false,
                    default: None,
                    flag: None,
                }],
                body: vec![],
                return_type: None,
            }],
        );
    }

    #[test]
    fn variadic_params() {
        assert_ast(
            "<?php function foo(...$bar) {}",
            &[Statement::Function {
                name: "foo".to_string().into(),
                params: vec![Param {
                    name: Expression::Variable { name: "bar".into() },
                    r#type: None,
                    variadic: true,
                    default: None,
                    flag: None,
                }],
                body: vec![],
                return_type: None,
            }],
        );

        assert_ast(
            "<?php function foo(string ...$bar) {}",
            &[Statement::Function {
                name: "foo".to_string().into(),
                params: vec![Param {
                    name: Expression::Variable { name: "bar".into() },
                    r#type: Some(Type::Plain("string".into())),
                    variadic: true,
                    default: None,
                    flag: None,
                }],
                body: vec![],
                return_type: None,
            }],
        );

        assert_ast(
            "<?php function foo($bar, $baz, ...$car) {}",
            &[Statement::Function {
                name: "foo".to_string().into(),
                params: vec![
                    Param {
                        name: Expression::Variable { name: "bar".into() },
                        r#type: None,
                        variadic: false,
                        default: None,
                        flag: None,
                    },
                    Param {
                        name: Expression::Variable { name: "baz".into() },
                        r#type: None,
                        variadic: false,
                        default: None,
                        flag: None,
                    },
                    Param {
                        name: Expression::Variable { name: "car".into() },
                        r#type: None,
                        variadic: true,
                        default: None,
                        flag: None,
                    },
                ],
                body: vec![],
                return_type: None,
            }],
        );
    }

    #[test]
    fn nullable_typestrings_test() {
        assert_ast(
            "<?php function foo(?string $b) {}",
            &[Statement::Function {
                name: "foo".to_string().into(),
                params: vec![Param {
                    name: Expression::Variable { name: "b".into() },
                    r#type: Some(Type::Nullable("string".into())),
                    variadic: false,
                    default: None,
                    flag: None,
                }],
                body: vec![],
                return_type: None,
            }],
        );
    }

    #[test]
    fn union_typestrings_test() {
        assert_ast(
            "<?php function foo(int|float $b) {}",
            &[Statement::Function {
                name: "foo".to_string().into(),
                params: vec![Param {
                    name: Expression::Variable { name: "b".into() },
                    r#type: Some(Type::Union(vec!["int".into(), "float".into()])),
                    variadic: false,
                    default: None,
                    flag: None,
                }],
                body: vec![],
                return_type: None,
            }],
        );

        assert_ast(
            "<?php function foo(string|int|float $b) {}",
            &[Statement::Function {
                name: "foo".to_string().into(),
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
                }],
                body: vec![],
                return_type: None,
            }],
        );
    }

    #[test]
    fn intersection_typestrings_test() {
        assert_ast(
            "<?php function foo(Foo&Bar $b) {}",
            &[Statement::Function {
                name: "foo".to_string().into(),
                params: vec![Param {
                    name: Expression::Variable { name: "b".into() },
                    r#type: Some(Type::Intersection(vec!["Foo".into(), "Bar".into()])),
                    variadic: false,
                    default: None,
                    flag: None,
                }],
                body: vec![],
                return_type: None,
            }],
        );

        assert_ast(
            "<?php function foo(Foo&Bar&Baz $b) {}",
            &[Statement::Function {
                name: "foo".to_string().into(),
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
                }],
                body: vec![],
                return_type: None,
            }],
        );
    }

    #[test]
    fn function_return_types() {
        assert_ast(
            "<?php function foo(): string {}",
            &[Statement::Function {
                name: "foo".to_string().into(),
                params: vec![],
                body: vec![],
                return_type: Some(Type::Plain("string".into())),
            }],
        );

        assert_ast(
            "<?php function foo(): void {}",
            &[Statement::Function {
                name: "foo".to_string().into(),
                params: vec![],
                body: vec![],
                return_type: Some(Type::Void),
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
                    extends: Some(Identifier::from("Foo")),
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
                    implements: vec![Identifier::from("Foo"), Identifier::from("Bar"),],
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
                        name: "foo".into(),
                        params: vec![],
                        body: vec![],
                        return_type: None,
                        flags: vec![MethodFlag::Public,]
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
                name: "MyClass".into(),
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
                    types: vec!["Exception".into()],
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
                    types: vec!["Exception".into()],
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
                        types: vec!["Exception".into()],
                        var: Some(Expression::Variable { name: "e".into() }),
                        body: vec![],
                    },
                    Catch {
                        types: vec!["CustomException".into()],
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
                    types: vec!["Exception".into()],
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
                name: "FOO".into(),
                value: Expression::Int { i: 1 },
                flags: vec![],
            }],
        );
    }

    #[test]
    fn global_statements() {
        assert_ast("<?php global $a;", &[
            Statement::Global { vars: vec!["a".into()] }
        ]);
    }

    #[test]
    fn multiple_global_vars_in_statement() {
        assert_ast("<?php global $a, $b;", &[
            Statement::Global { vars: vec!["a".into(), "b".into()] }
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
