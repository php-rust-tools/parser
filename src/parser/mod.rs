use std::vec::IntoIter;

use crate::expect_literal;
use crate::expect_token;
use crate::expected_token_err;
use crate::lexer::byte_string::ByteString;
use crate::lexer::token::Token;
use crate::lexer::token::TokenKind;
use crate::parser::ast::{
    ArrayItem, Block, Case, Catch, ClosureUse, Constant, DeclareItem, ElseIf, Expression,
    IncludeKind, MagicConst, MatchArm, Program, Statement, StaticVar, StringPart,
    TryBlockCaughtType, Type, Use, UseKind,
};
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::ident::is_reserved_ident;
use crate::parser::params::ParamPosition;
use crate::parser::precedence::{Associativity, Precedence};

pub mod ast;
pub mod error;

mod block;
mod classish;
mod classish_statement;
mod comments;
mod flags;
mod functions;
mod ident;
mod macros;
mod params;
mod precedence;
mod punc;
mod vars;

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

    pub fn parse(&mut self, tokens: Vec<Token>) -> ParseResult<Program> {
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

            ast.push(self.top_level_statement()?);

            self.clear_comments();
        }

        Ok(ast.to_vec())
    }

    fn try_block_caught_type_string(&mut self) -> ParseResult<TryBlockCaughtType> {
        let id = self.full_name()?;

        if self.current.kind == TokenKind::Pipe {
            self.next();

            let mut types = vec![id.into()];

            while !self.is_eof() {
                let id = self.full_name()?;
                types.push(id.into());

                if self.current.kind != TokenKind::Pipe {
                    break;
                }

                self.next();
            }

            return Ok(TryBlockCaughtType::Union(types));
        }

        Ok(TryBlockCaughtType::Identifier(id.into()))
    }

    fn type_string(&mut self) -> ParseResult<Type> {
        if self.current.kind == TokenKind::Question {
            self.next();
            let t = self.type_with_static()?;
            return Ok(Type::Nullable(Box::new(parse_simple_type(t))));
        }

        let id = self.type_with_static()?;

        if self.current.kind == TokenKind::Pipe {
            self.next();

            let r#type = parse_simple_type(id);
            if r#type.standalone() {
                return Err(ParseError::StandaloneTypeUsedInCombination(
                    r#type,
                    self.current.span,
                ));
            }

            let mut types = vec![r#type];

            while !self.is_eof() {
                let id = self.type_with_static()?;
                let r#type = parse_simple_type(id);
                if r#type.standalone() {
                    return Err(ParseError::StandaloneTypeUsedInCombination(
                        r#type,
                        self.current.span,
                    ));
                }

                types.push(r#type);

                if self.current.kind != TokenKind::Pipe {
                    break;
                } else {
                    self.next();
                }
            }

            return Ok(Type::Union(types));
        }

        if self.current.kind == TokenKind::Ampersand
            && !matches!(self.peek.kind, TokenKind::Variable(_))
        {
            self.next();

            let r#type = parse_simple_type(id);
            if r#type.standalone() {
                return Err(ParseError::StandaloneTypeUsedInCombination(
                    r#type,
                    self.current.span,
                ));
            }

            let mut types = vec![r#type];

            while !self.is_eof() {
                let id = self.type_with_static()?;
                let r#type = parse_simple_type(id);
                if r#type.standalone() {
                    return Err(ParseError::StandaloneTypeUsedInCombination(
                        r#type,
                        self.current.span,
                    ));
                }

                types.push(r#type);

                if self.current.kind != TokenKind::Ampersand {
                    break;
                } else {
                    self.next();
                }
            }

            return Ok(Type::Intersection(types));
        }

        Ok(parse_simple_type(id))
    }

    fn top_level_statement(&mut self) -> ParseResult<Statement> {
        self.skip_comments();

        let statement = match &self.current.kind {
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
                    let mut body = Block::new();
                    while !self.is_eof() {
                        body.push(self.top_level_statement()?);
                    }
                    body
                };

                if braced {
                    self.rbrace()?;
                }

                Statement::Namespace { name, body }
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

                if self.peek.kind == TokenKind::LeftBrace {
                    let prefix = self.full_name()?;
                    self.next();

                    let mut uses = Vec::new();
                    while self.current.kind != TokenKind::RightBrace {
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
                    }

                    self.rbrace()?;
                    self.semi()?;

                    Statement::GroupUse {
                        prefix: prefix.into(),
                        kind,
                        uses,
                    }
                } else {
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
            }
            TokenKind::Const => {
                self.next();

                let mut constants = vec![];

                while self.current.kind != TokenKind::SemiColon {
                    let name = self.ident()?;

                    expect_token!([TokenKind::Equals], self, "`=`");

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
            _ => self.statement()?,
        };

        self.clear_comments();

        Ok(statement)
    }

    fn statement(&mut self) -> ParseResult<Statement> {
        self.skip_comments();

        let statement = match &self.current.kind {
            TokenKind::Goto => {
                self.next();

                let label = self.ident()?.into();

                self.semi()?;

                Statement::Goto { label }
            }
            TokenKind::Identifier(_) if self.peek.kind == TokenKind::Colon => {
                let label = self.ident()?.into();

                self.colon()?;

                Statement::Label { label }
            }
            TokenKind::Declare => {
                self.next();
                self.lparen()?;

                let mut declares = Vec::new();
                while self.current.kind != TokenKind::RightParen {
                    let key = self.ident()?;

                    expect_token!([TokenKind::Equals], self, "`=`");

                    let value = expect_literal!(self);

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
                } else if self.current.kind == TokenKind::Colon {
                    self.colon()?;
                    let b = self.block(&TokenKind::EndDeclare)?;
                    expect_token!([TokenKind::EndDeclare], self, "`enddeclare`");
                    self.semi()?;
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
                        expect_token!([TokenKind::Equals], self, "`=`");
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

                expect_token!([TokenKind::While], self, "`while`");

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

                let end_token = if self.current.kind == TokenKind::Colon {
                    self.colon()?;
                    TokenKind::EndWhile
                } else {
                    self.lbrace()?;
                    TokenKind::RightBrace
                };

                let body = self.block(&end_token)?;

                if end_token == TokenKind::RightBrace {
                    self.rbrace()?;
                } else {
                    expect_token!([TokenKind::EndWhile], self, "`endwhile`");
                    self.semi()?;
                }

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

                let end_token = if self.current.kind == TokenKind::Colon {
                    self.colon()?;
                    TokenKind::EndFor
                } else {
                    self.lbrace()?;
                    TokenKind::RightBrace
                };

                let then = self.block(&end_token)?;

                if end_token == TokenKind::EndFor {
                    expect_token!([TokenKind::EndFor], self, "`endfor`");
                    self.semi()?;
                } else {
                    self.rbrace()?;
                };

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

                expect_token!([TokenKind::As], self, ["`as`"]);

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

                let end_token = if self.current.kind == TokenKind::Colon {
                    self.colon()?;
                    TokenKind::EndForeach
                } else {
                    self.lbrace()?;
                    TokenKind::RightBrace
                };

                let body = self.block(&end_token)?;

                if end_token == TokenKind::EndForeach {
                    expect_token!([TokenKind::EndForeach], self, "`endforeach`");
                    self.semi()?;
                } else {
                    self.rbrace()?;
                }

                Statement::Foreach {
                    expr,
                    by_ref,
                    key_var,
                    value_var,
                    body,
                }
            }
            TokenKind::Abstract => self.class_definition()?,
            TokenKind::Readonly => self.class_definition()?,
            TokenKind::Final => self.class_definition()?,
            TokenKind::Class => self.class_definition()?,
            TokenKind::Interface => self.interface_definition()?,
            TokenKind::Trait => self.trait_definition()?,
            TokenKind::Enum => self.enum_definition()?,
            TokenKind::Switch => {
                self.next();

                self.lparen()?;

                let condition = self.expression(Precedence::Lowest)?;

                self.rparen()?;

                let end_token = if self.current.kind == TokenKind::Colon {
                    self.colon()?;
                    TokenKind::EndSwitch
                } else {
                    self.lbrace()?;
                    TokenKind::RightBrace
                };

                let mut cases = Vec::new();
                loop {
                    if self.current.kind == end_token {
                        break;
                    }

                    match self.current.kind {
                        TokenKind::Case => {
                            self.next();

                            let condition = self.expression(Precedence::Lowest)?;

                            expect_token!(
                                [TokenKind::Colon, TokenKind::SemiColon],
                                self,
                                ["`:`", "`;`"]
                            );
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

                            expect_token!(
                                [TokenKind::Colon, TokenKind::SemiColon],
                                self,
                                ["`:`", "`;`"]
                            );

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
                            return expected_token_err!(["`case`", "`default`"], self);
                        }
                    }
                }

                if end_token == TokenKind::EndSwitch {
                    expect_token!([TokenKind::EndSwitch], self, ["`endswitch`"]);
                    self.semi()?;
                } else {
                    self.rbrace()?;
                }

                Statement::Switch { condition, cases }
            }
            TokenKind::If => {
                self.next();

                self.lparen()?;

                let condition = self.expression(Precedence::Lowest)?;

                self.rparen()?;

                // FIXME: Tidy up duplication and make the intent a bit clearer.
                match self.current.kind {
                    TokenKind::Colon => {
                        self.next();

                        let mut then = vec![];
                        while !matches!(
                            self.current.kind,
                            TokenKind::ElseIf | TokenKind::Else | TokenKind::EndIf
                        ) {
                            then.push(self.statement()?);
                        }

                        let mut else_ifs = vec![];
                        loop {
                            if self.current.kind != TokenKind::ElseIf {
                                break;
                            }

                            self.next();

                            self.lparen()?;
                            let condition = self.expression(Precedence::Lowest)?;
                            self.rparen()?;

                            self.colon()?;

                            let mut body = vec![];
                            while !matches!(
                                self.current.kind,
                                TokenKind::ElseIf | TokenKind::Else | TokenKind::EndIf
                            ) {
                                body.push(self.statement()?);
                            }

                            else_ifs.push(ElseIf { condition, body });
                        }

                        let mut r#else = None;
                        if self.current.kind == TokenKind::Else {
                            self.next();
                            self.colon()?;

                            let mut body = vec![];
                            while self.current.kind != TokenKind::EndIf {
                                body.push(self.statement()?);
                            }
                            r#else = Some(body);
                        }

                        expect_token!([TokenKind::EndIf], self, ["`endif`"]);
                        self.semi()?;

                        Statement::If {
                            condition,
                            then,
                            else_ifs,
                            r#else,
                        }
                    }
                    _ => {
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

                        expect_token!([TokenKind::Else], self, ["`else`"]);

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
                }
            }
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
                    if let Some((index, _)) = self.iter.clone().enumerate().next() {
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

                    let types = self.try_block_caught_type_string()?;
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

                if catches.is_empty() && finally.is_none() {
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
            _ => {
                let expr = self.expression(Precedence::Lowest)?;

                self.semi()?;

                Statement::Expression { expr }
            }
        };

        self.skip_comments();

        Ok(statement)
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

                if self.current.kind == TokenKind::SemiColon {
                    Expression::Yield {
                        key: None,
                        value: None,
                    }
                } else {
                    let mut from = false;

                    if self.current.kind == TokenKind::From {
                        self.next();
                        from = true;
                    }

                    let mut key = None;
                    let mut value = Box::new(self.expression(if from {
                        Precedence::YieldFrom
                    } else {
                        Precedence::Yield
                    })?);

                    if self.current.kind == TokenKind::DoubleArrow && !from {
                        self.next();
                        key = Some(value.clone());
                        value = Box::new(self.expression(Precedence::Yield)?);
                    }

                    if from {
                        Expression::YieldFrom { value }
                    } else {
                        Expression::Yield {
                            key,
                            value: Some(value),
                        }
                    }
                }
            }
            TokenKind::Clone => {
                self.next();

                let target = self.expression(Precedence::CloneOrNew)?;

                Expression::Clone {
                    target: Box::new(target),
                }
            }
            TokenKind::Variable(v) => {
                let e = Expression::Variable { name: v.clone() };
                self.next();
                e
            }
            TokenKind::LiteralInteger(i) => {
                let e = Expression::LiteralInteger { i: *i };
                self.next();
                e
            }
            TokenKind::LiteralFloat(f) => {
                let f = Expression::LiteralFloat { f: *f };
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
            TokenKind::Static if matches!(self.peek.kind, TokenKind::DoubleColon) => {
                self.next();
                Expression::Static
            }
            TokenKind::LiteralString(s) => {
                let e = Expression::LiteralString { value: s.clone() };
                self.next();
                e
            }
            TokenKind::StringPart(_) => self.interpolated_string()?,
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

                    expect_token!([TokenKind::DoubleArrow], self, "`=>`");

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
                    let unpack = if self.current.kind == TokenKind::Ellipsis {
                        self.next();
                        true
                    } else {
                        false
                    };

                    let mut value = self.expression(Precedence::Lowest)?;

                    if self.current.kind == TokenKind::DoubleArrow {
                        self.next();

                        key = Some(value);
                        value = self.expression(Precedence::Lowest)?;
                    }

                    items.push(ArrayItem { key, value, unpack });

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
                            unpack: false,
                        });
                        self.next();
                        continue;
                    }

                    let mut key = None;
                    let unpack = if self.current.kind == TokenKind::Ellipsis {
                        self.next();
                        true
                    } else {
                        false
                    };
                    let mut value = self.expression(Precedence::Lowest)?;

                    if self.current.kind == TokenKind::DoubleArrow {
                        self.next();

                        key = Some(value);
                        value = self.expression(Precedence::Lowest)?;
                    }

                    items.push(ArrayItem { key, value, unpack });

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

                let params = self.param_list(ParamPosition::Function)?;

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
                    self.colon()?;

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

                let params = self.param_list(ParamPosition::Function)?;

                self.rparen()?;

                let mut return_type = None;

                if self.current.kind == TokenKind::Colon || self.config.force_type_strings {
                    self.colon()?;

                    return_type = Some(self.type_string()?);
                }

                expect_token!([TokenKind::DoubleArrow], self, ["`=>`"]);

                let value = self.expression(Precedence::Lowest)?;

                Expression::ArrowFunction {
                    params,
                    return_type,
                    expr: Box::new(value),
                    by_ref,
                    r#static: false,
                }
            }
            TokenKind::New if self.peek.kind == TokenKind::Class => {
                self.anonymous_class_definition()?
            }
            TokenKind::New => {
                self.next();

                let mut args = vec![];
                let target = self.expression(Precedence::CloneOrNew)?;

                if self.current.kind == TokenKind::LeftParen {
                    self.lparen()?;

                    args = self.args_list()?;

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
            TokenKind::Dollar => self.dynamic_variable()?,
            _ => {
                return Err(ParseError::UnexpectedToken(
                    self.current.kind.to_string(),
                    self.current.span,
                ))
            }
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
                        self.colon()?;
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
                let args = self.args_list()?;

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

                    expect_token!([TokenKind::RightBracket], self, ["`]`"]);

                    Expression::ArrayIndex {
                        array: Box::new(lhs),
                        index: Some(Box::new(index)),
                    }
                }
            }
            TokenKind::DoubleColon => {
                let mut must_be_method_call = false;

                let property = match self.current.kind.clone() {
                    TokenKind::Dollar => self.dynamic_variable()?,
                    TokenKind::Variable(var) => {
                        self.next();
                        Expression::Variable { name: var }
                    }
                    TokenKind::LeftBrace => {
                        must_be_method_call = true;
                        self.next();

                        let name = self.expression(Precedence::Lowest)?;

                        self.rbrace()?;

                        Expression::DynamicVariable {
                            name: Box::new(name),
                        }
                    }
                    TokenKind::Identifier(ident) => {
                        self.next();
                        Expression::Identifier { name: ident }
                    }
                    TokenKind::Class => {
                        self.next();
                        // FIXME: Can this be represented in a nicer way? Kind of hacky.
                        Expression::Identifier {
                            name: "class".into(),
                        }
                    }
                    _ if is_reserved_ident(&self.current.kind) => Expression::Identifier {
                        name: self.ident_maybe_reserved()?,
                    },
                    _ => {
                        return expected_token_err!(["`{`", "`$`", "an identifier"], self);
                    }
                };

                let lhs = Box::new(lhs);

                match property {
                    // 1. If we have an identifier and the current token is not a left paren,
                    //    the resulting expression must be a constant fetch.
                    Expression::Identifier { name }
                        if self.current.kind != TokenKind::LeftParen =>
                    {
                        Expression::ConstFetch {
                            target: lhs,
                            constant: name.into(),
                        }
                    }
                    // 2. If the current token is a left paren, or if we know the property expression
                    //    is only valid a method call context, we can assume we're parsing a static
                    //    method call.
                    _ if self.current.kind == TokenKind::LeftParen || must_be_method_call => {
                        self.lparen()?;

                        let args = self.args_list()?;

                        self.rparen()?;

                        Expression::StaticMethodCall {
                            target: lhs,
                            method: Box::new(property),
                            args,
                        }
                    }
                    // 3. If we haven't met any of the previous conditions, we can assume
                    //    that we're parsing a static property fetch.
                    _ => Expression::StaticPropertyFetch {
                        target: lhs,
                        property: Box::new(property),
                    },
                }
            }
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
                    TokenKind::Dollar => self.dynamic_variable()?,
                    _ => Expression::Identifier {
                        name: self.ident_maybe_reserved()?,
                    },
                };

                if self.current.kind == TokenKind::LeftParen {
                    self.next();

                    let args = self.args_list()?;

                    self.rparen()?;

                    if op == &TokenKind::NullsafeArrow {
                        Expression::NullsafeMethodCall {
                            target: Box::new(lhs),
                            method: Box::new(property),
                            args,
                        }
                    } else {
                        Expression::MethodCall {
                            target: Box::new(lhs),
                            method: Box::new(property),
                            args,
                        }
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

    fn interpolated_string(&mut self) -> ParseResult<Expression> {
        let mut parts = Vec::new();

        while self.current.kind != TokenKind::DoubleQuote {
            match &self.current.kind {
                TokenKind::StringPart(s) => {
                    if s.len() > 0 {
                        parts.push(StringPart::Const(s.clone()));
                    }
                    self.next();
                }
                TokenKind::DollarLeftBrace => {
                    self.next();
                    let e = match (&self.current.kind, &self.peek.kind) {
                        (TokenKind::Identifier(var), TokenKind::RightBrace) => {
                            // "${var}"
                            let e = Expression::Variable { name: var.clone() };
                            self.next();
                            self.next();
                            e
                        }
                        (TokenKind::Identifier(var), TokenKind::LeftBracket) => {
                            // "${var[e]}"
                            let var = Expression::Variable { name: var.clone() };
                            self.next();
                            self.next();
                            let e = self.expression(Precedence::Lowest)?;
                            expect_token!([TokenKind::RightBracket], self, "`]`");
                            expect_token!([TokenKind::RightBrace], self, "`}`");
                            Expression::ArrayIndex {
                                array: Box::new(var),
                                index: Some(Box::new(e)),
                            }
                        }
                        _ => {
                            // Arbitrary expressions are allowed, but are treated as variable variables.
                            let e = self.expression(Precedence::Lowest)?;
                            expect_token!([TokenKind::RightBrace], self, "`}`");

                            Expression::DynamicVariable { name: Box::new(e) }
                        }
                    };
                    parts.push(StringPart::Expr(Box::new(e)));
                }
                TokenKind::LeftBrace => {
                    // "{$expr}"
                    self.next();
                    let e = self.expression(Precedence::Lowest)?;
                    expect_token!([TokenKind::RightBrace], self, "`}`");
                    parts.push(StringPart::Expr(Box::new(e)));
                }
                TokenKind::Variable(var) => {
                    // "$expr", "$expr[0]", "$expr[name]", "$expr->a"
                    let var = Expression::Variable { name: var.clone() };
                    self.next();
                    let e = match self.current.kind {
                        TokenKind::LeftBracket => {
                            self.next();
                            // Full expression syntax is not allowed here,
                            // so we can't call self.expression.
                            let index = match &self.current.kind {
                                &TokenKind::LiteralInteger(i) => {
                                    self.next();
                                    Expression::LiteralInteger { i }
                                }
                                TokenKind::Minus => {
                                    self.next();
                                    if let TokenKind::LiteralInteger(i) = self.current.kind {
                                        self.next();
                                        Expression::Negate {
                                            value: Box::new(Expression::LiteralInteger { i }),
                                        }
                                    } else {
                                        return expected_token_err!("an integer", self);
                                    }
                                }
                                TokenKind::Identifier(ident) => {
                                    let e = Expression::LiteralString {
                                        value: ident.clone(),
                                    };
                                    self.next();
                                    e
                                }
                                TokenKind::Variable(var) => {
                                    let e = Expression::Variable { name: var.clone() };
                                    self.next();
                                    e
                                }
                                _ => {
                                    return expected_token_err!(
                                        ["`-`", "an integer", "an identifier", "a variable"],
                                        self
                                    );
                                }
                            };

                            expect_token!([TokenKind::RightBracket], self, "`]`");
                            Expression::ArrayIndex {
                                array: Box::new(var),
                                index: Some(Box::new(index)),
                            }
                        }
                        TokenKind::Arrow => {
                            self.next();
                            Expression::PropertyFetch {
                                target: Box::new(var),
                                property: Box::new(Expression::Identifier {
                                    name: self.ident_maybe_reserved()?,
                                }),
                            }
                        }
                        TokenKind::NullsafeArrow => {
                            self.next();
                            Expression::NullsafePropertyFetch {
                                target: Box::new(var),
                                property: Box::new(Expression::Identifier {
                                    name: self.ident_maybe_reserved()?,
                                }),
                            }
                        }
                        _ => var,
                    };
                    parts.push(StringPart::Expr(Box::new(e)));
                }
                _ => {
                    return expected_token_err!(["`${`", "`{$", "`\"`", "a variable"], self);
                }
            }
        }
        self.next();

        Ok(Expression::InterpolatedString { parts })
    }

    fn is_eof(&self) -> bool {
        self.current.kind == TokenKind::Eof
    }

    pub fn next(&mut self) {
        self.current = self.peek.clone();
        self.peek = self.iter.next().unwrap_or_default()
    }
}

fn parse_simple_type(id: ByteString) -> Type {
    let name = &id[..];
    let lowered_name = name.to_ascii_lowercase();
    match lowered_name.as_slice() {
        b"void" => Type::Void,
        b"never" => Type::Never,
        b"null" => Type::Null,
        b"true" => Type::True,
        b"false" => Type::False,
        b"float" => Type::Float,
        b"bool" => Type::Boolean,
        b"int" => Type::Integer,
        b"string" => Type::String,
        b"array" => Type::Array,
        b"object" => Type::Object,
        b"mixed" => Type::Mixed,
        b"iterable" => Type::Iterable,
        b"callable" => Type::Callable,
        _ => Type::Identifier(id.into()),
    }
}

fn is_prefix(op: &TokenKind) -> bool {
    matches!(
        op,
        TokenKind::Bang
            | TokenKind::Print
            | TokenKind::BitwiseNot
            | TokenKind::Decrement
            | TokenKind::Increment
            | TokenKind::Minus
            | TokenKind::Plus
            | TokenKind::StringCast
            | TokenKind::BinaryCast
            | TokenKind::ObjectCast
            | TokenKind::BoolCast
            | TokenKind::BooleanCast
            | TokenKind::IntCast
            | TokenKind::IntegerCast
            | TokenKind::FloatCast
            | TokenKind::DoubleCast
            | TokenKind::RealCast
            | TokenKind::UnsetCast
            | TokenKind::ArrayCast
            | TokenKind::At
    )
}

fn prefix(op: &TokenKind, rhs: Expression) -> Expression {
    match op {
        TokenKind::Print => Expression::Print {
            value: Box::new(rhs),
        },
        TokenKind::Bang => Expression::BooleanNot {
            value: Box::new(rhs),
        },
        TokenKind::Minus => Expression::Negate {
            value: Box::new(rhs),
        },
        TokenKind::Plus => Expression::UnaryPlus {
            value: Box::new(rhs),
        },
        TokenKind::BitwiseNot => Expression::BitwiseNot {
            value: Box::new(rhs),
        },
        TokenKind::Decrement => Expression::PreDecrement {
            value: Box::new(rhs),
        },
        TokenKind::Increment => Expression::PreIncrement {
            value: Box::new(rhs),
        },
        TokenKind::StringCast
        | TokenKind::BinaryCast
        | TokenKind::ObjectCast
        | TokenKind::BoolCast
        | TokenKind::BooleanCast
        | TokenKind::IntCast
        | TokenKind::IntegerCast
        | TokenKind::FloatCast
        | TokenKind::DoubleCast
        | TokenKind::RealCast
        | TokenKind::UnsetCast
        | TokenKind::ArrayCast => Expression::Cast {
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
            | TokenKind::RightShiftEquals
            | TokenKind::LeftShiftEquals
            | TokenKind::CaretEquals
            | TokenKind::AmpersandEquals
            | TokenKind::PipeEquals
            | TokenKind::PercentEquals
            | TokenKind::PowEquals
            | TokenKind::LogicalAnd
            | TokenKind::LogicalOr
            | TokenKind::LogicalXor
            | TokenKind::Spaceship
            | TokenKind::LeftShift
            | TokenKind::RightShift
            | TokenKind::Ampersand
            | TokenKind::Pipe
            | TokenKind::Caret
            | TokenKind::Percent
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
            | TokenKind::AngledLeftRight
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
