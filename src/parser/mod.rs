use crate::expect_literal;
use crate::expect_token;
use crate::expected_token_err;
use crate::lexer::token::Token;
use crate::lexer::token::TokenKind;
use crate::lexer::DocStringKind;
use crate::parser::ast::comments::Comment;
use crate::parser::ast::comments::CommentFormat;
use crate::parser::ast::identifiers::Identifier;
use crate::parser::ast::variables::Variable;
use crate::parser::ast::DefaultMatchArm;
use crate::parser::ast::{
    ArrayItem, Block, Case, Constant, DeclareItem, ElseIf, Expression, IncludeKind, MagicConst,
    MatchArm, Program, Statement, StaticVar, StringPart, Use, UseKind,
};
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::internal::identifiers::is_reserved_ident;
use crate::parser::internal::precedences::{Associativity, Precedence};
use crate::parser::state::State;

use self::ast::ListItem;

pub mod ast;
pub mod error;

mod internal;
mod macros;
mod state;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Parser;

impl Parser {
    pub const fn new() -> Self {
        Self {}
    }

    pub fn parse(&self, tokens: Vec<Token>) -> ParseResult<Program> {
        let mut state = State::new(tokens);

        let mut ast = Program::new();

        while state.current.kind != TokenKind::Eof {
            if matches!(
                state.current.kind,
                TokenKind::OpenTag(_) | TokenKind::CloseTag
            ) {
                state.next();
                continue;
            }

            state.gather_comments();

            if state.is_eof() {
                break;
            }

            ast.push(self.top_level_statement(&mut state)?);

            state.clear_comments();
        }

        Ok(ast.to_vec())
    }

    fn top_level_statement(&self, state: &mut State) -> ParseResult<Statement> {
        state.skip_comments();

        let statement = match &state.current.kind {
            TokenKind::Namespace => self.namespace(state)?,
            TokenKind::Use => {
                state.next();

                let kind = match state.current.kind {
                    TokenKind::Function => {
                        state.next();
                        UseKind::Function
                    }
                    TokenKind::Const => {
                        state.next();
                        UseKind::Const
                    }
                    _ => UseKind::Normal,
                };

                if state.peek.kind == TokenKind::LeftBrace {
                    let prefix = self.full_name(state)?;
                    state.next();

                    let mut uses = Vec::new();
                    while state.current.kind != TokenKind::RightBrace {
                        let name = self.full_name(state)?;
                        let mut alias = None;

                        if state.current.kind == TokenKind::As {
                            state.next();
                            alias = Some(self.ident(state)?);
                        }

                        uses.push(Use { name, alias });

                        if state.current.kind == TokenKind::Comma {
                            state.next();
                            continue;
                        }
                    }

                    self.right_brace(state)?;
                    self.semicolon(state)?;

                    Statement::GroupUse { prefix, kind, uses }
                } else {
                    let mut uses = Vec::new();
                    while !state.is_eof() {
                        let name = self.full_name(state)?;
                        let mut alias = None;

                        if state.current.kind == TokenKind::As {
                            state.next();
                            alias = Some(self.ident(state)?);
                        }

                        uses.push(Use { name, alias });

                        if state.current.kind == TokenKind::Comma {
                            state.next();
                            continue;
                        }

                        self.semicolon(state)?;
                        break;
                    }

                    Statement::Use { uses, kind }
                }
            }
            TokenKind::Const => {
                state.next();

                let mut constants = vec![];

                loop {
                    let name = self.ident(state)?;

                    self.skip(state, TokenKind::Equals)?;

                    let value = self.expression(state, Precedence::Lowest)?;

                    constants.push(Constant { name, value });

                    if state.current.kind == TokenKind::Comma {
                        state.next();
                    } else {
                        break;
                    }
                }

                self.semicolon(state)?;

                Statement::Constant { constants }
            }
            TokenKind::HaltCompiler => {
                state.next();

                let content = if let TokenKind::InlineHtml(content) = state.current.kind.clone() {
                    state.next();
                    Some(content)
                } else {
                    None
                };

                Statement::HaltCompiler { content }
            }
            _ => self.statement(state)?,
        };

        state.clear_comments();

        Ok(statement)
    }

    fn statement(&self, state: &mut State) -> ParseResult<Statement> {
        let has_attributes = self.gather_attributes(state)?;

        let statement = if has_attributes {
            match &state.current.kind {
                TokenKind::Abstract => self.class_definition(state)?,
                TokenKind::Readonly => self.class_definition(state)?,
                TokenKind::Final => self.class_definition(state)?,
                TokenKind::Class => self.class_definition(state)?,
                TokenKind::Interface => self.interface_definition(state)?,
                TokenKind::Trait => self.trait_definition(state)?,
                TokenKind::Enum => self.enum_definition(state)?,
                TokenKind::Function
                    if matches!(
                        state.peek.kind,
                        TokenKind::Identifier(_) | TokenKind::Null | TokenKind::Ampersand
                    ) =>
                {
                    // FIXME: This is incredibly hacky but we don't have a way to look at
                    // the next N tokens right now. We could probably do with a `peek_buf()`
                    // method like the Lexer has.
                    if state.peek.kind == TokenKind::Ampersand {
                        let mut cloned = state.iter.clone();
                        if let Some((index, _)) = state.iter.clone().enumerate().next() {
                            if !matches!(
                                cloned.nth(index),
                                Some(Token {
                                    kind: TokenKind::Identifier(_),
                                    ..
                                })
                            ) {
                                let expr = self.expression(state, Precedence::Lowest)?;

                                self.semicolon(state)?;

                                return Ok(Statement::Expression { expr });
                            }
                        }

                        self.function(state)?
                    } else {
                        self.function(state)?
                    }
                }
                _ => {
                    // Note, we can get attributes and know their span, maybe use that in the
                    // error in the future?
                    return Err(ParseError::ExpectedItemDefinitionAfterAttributes(
                        state.current.span,
                    ));
                }
            }
        } else {
            match &state.current.kind {
                TokenKind::Abstract => self.class_definition(state)?,
                TokenKind::Readonly => self.class_definition(state)?,
                TokenKind::Final => self.class_definition(state)?,
                TokenKind::Class => self.class_definition(state)?,
                TokenKind::Interface => self.interface_definition(state)?,
                TokenKind::Trait => self.trait_definition(state)?,
                TokenKind::Enum => self.enum_definition(state)?,
                TokenKind::Function
                    if matches!(
                        state.peek.kind,
                        TokenKind::Identifier(_) | TokenKind::Null | TokenKind::Ampersand
                    ) =>
                {
                    // FIXME: This is incredibly hacky but we don't have a way to look at
                    // the next N tokens right now. We could probably do with a `peek_buf()`
                    // method like the Lexer has.
                    if state.peek.kind == TokenKind::Ampersand {
                        let mut cloned = state.iter.clone();
                        if let Some((index, _)) = state.iter.clone().enumerate().next() {
                            if !matches!(
                                cloned.nth(index),
                                Some(Token {
                                    kind: TokenKind::Identifier(_),
                                    ..
                                })
                            ) {
                                let expr = self.expression(state, Precedence::Lowest)?;

                                self.semicolon(state)?;

                                return Ok(Statement::Expression { expr });
                            }
                        }

                        self.function(state)?
                    } else {
                        self.function(state)?
                    }
                }
                TokenKind::Goto => {
                    state.next();

                    let label = self.ident(state)?;

                    self.semicolon(state)?;

                    Statement::Goto { label }
                }
                TokenKind::Identifier(_) if state.peek.kind == TokenKind::Colon => {
                    let label = self.ident(state)?;

                    self.colon(state)?;

                    Statement::Label { label }
                }
                TokenKind::Declare => {
                    state.next();
                    self.left_parenthesis(state)?;

                    let mut declares = Vec::new();
                    loop {
                        let key = self.ident(state)?;

                        self.skip(state, TokenKind::Equals)?;

                        let value = expect_literal!(state);

                        declares.push(DeclareItem { key, value });

                        if state.current.kind == TokenKind::Comma {
                            state.next();
                        } else {
                            break;
                        }
                    }

                    self.right_parenthesis(state)?;

                    let body = if state.current.kind == TokenKind::LeftBrace {
                        state.next();
                        let b = self.block(state, &TokenKind::RightBrace)?;
                        self.right_brace(state)?;
                        b
                    } else if state.current.kind == TokenKind::Colon {
                        self.colon(state)?;
                        let b = self.block(state, &TokenKind::EndDeclare)?;
                        self.skip(state, TokenKind::EndDeclare)?;
                        self.semicolon(state)?;
                        b
                    } else {
                        self.semicolon(state)?;
                        vec![]
                    };

                    Statement::Declare { declares, body }
                }
                TokenKind::Global => {
                    state.next();

                    let mut vars = vec![];
                    // `loop` instead of `while` as we don't allow for extra commas.
                    loop {
                        vars.push(self.var(state)?);

                        if state.current.kind == TokenKind::Comma {
                            state.next();
                        } else {
                            break;
                        }
                    }

                    self.semicolon(state)?;
                    Statement::Global { vars }
                }
                TokenKind::Static if matches!(state.peek.kind, TokenKind::Variable(_)) => {
                    state.next();

                    let mut vars = vec![];

                    // `loop` instead of `while` as we don't allow for extra commas.
                    loop {
                        let var = self.var(state)?;
                        let mut default = None;

                        if state.current.kind == TokenKind::Equals {
                            state.next();

                            default = Some(self.expression(state, Precedence::Lowest)?);
                        }

                        // TODO: group static vars.
                        vars.push(StaticVar { var, default });

                        if state.current.kind == TokenKind::Comma {
                            state.next();
                        } else {
                            break;
                        }
                    }

                    self.semicolon(state)?;

                    Statement::Static { vars }
                }
                TokenKind::InlineHtml(html) => {
                    let s = Statement::InlineHtml(html.clone());
                    state.next();
                    s
                }
                TokenKind::SingleLineComment(comment) => {
                    let start = state.current.span;
                    let content = comment.clone();
                    state.next();
                    let end = state.current.span;
                    let format = CommentFormat::SingleLine;

                    Statement::Comment(Comment {
                        start,
                        end,
                        format,
                        content,
                    })
                }
                TokenKind::MultiLineComment(comment) => {
                    let start = state.current.span;
                    let content = comment.clone();
                    state.next();
                    let end = state.current.span;
                    let format = CommentFormat::MultiLine;

                    Statement::Comment(Comment {
                        start,
                        end,
                        format,
                        content,
                    })
                }
                TokenKind::HashMarkComment(comment) => {
                    let start = state.current.span;
                    let content = comment.clone();
                    state.next();
                    let end = state.current.span;
                    let format = CommentFormat::HashMark;

                    Statement::Comment(Comment {
                        start,
                        end,
                        format,
                        content,
                    })
                }
                TokenKind::DocumentComment(comment) => {
                    let start = state.current.span;
                    let content = comment.clone();
                    state.next();
                    let end = state.current.span;
                    let format = CommentFormat::Document;

                    Statement::Comment(Comment {
                        start,
                        end,
                        format,
                        content,
                    })
                }
                TokenKind::Do => {
                    state.next();

                    self.left_brace(state)?;
                    let body = self.block(state, &TokenKind::RightBrace)?;
                    self.right_brace(state)?;

                    self.skip(state, TokenKind::While)?;

                    self.left_parenthesis(state)?;
                    let condition = self.expression(state, Precedence::Lowest)?;
                    self.right_parenthesis(state)?;
                    self.semicolon(state)?;

                    Statement::DoWhile { condition, body }
                }
                TokenKind::While => {
                    state.next();
                    self.left_parenthesis(state)?;

                    let condition = self.expression(state, Precedence::Lowest)?;

                    self.right_parenthesis(state)?;

                    let end_token = if state.current.kind == TokenKind::Colon {
                        self.colon(state)?;
                        TokenKind::EndWhile
                    } else {
                        self.left_brace(state)?;
                        TokenKind::RightBrace
                    };

                    let body = self.block(state, &end_token)?;

                    if end_token == TokenKind::RightBrace {
                        self.right_brace(state)?;
                    } else {
                        self.skip(state, TokenKind::EndWhile)?;
                        self.semicolon(state)?;
                    }

                    Statement::While { condition, body }
                }
                TokenKind::For => {
                    state.next();

                    self.left_parenthesis(state)?;

                    let mut init = Vec::new();
                    loop {
                        if state.current.kind == TokenKind::SemiColon {
                            break;
                        }

                        init.push(self.expression(state, Precedence::Lowest)?);

                        if state.current.kind == TokenKind::Comma {
                            state.next();
                        } else {
                            break;
                        }
                    }

                    self.semicolon(state)?;

                    let mut condition = Vec::new();
                    loop {
                        if state.current.kind == TokenKind::SemiColon {
                            break;
                        }

                        condition.push(self.expression(state, Precedence::Lowest)?);

                        if state.current.kind == TokenKind::Comma {
                            state.next();
                        } else {
                            break;
                        }
                    }
                    self.semicolon(state)?;

                    let mut r#loop = Vec::new();
                    loop {
                        if state.current.kind == TokenKind::RightParen {
                            break;
                        }

                        r#loop.push(self.expression(state, Precedence::Lowest)?);

                        if state.current.kind == TokenKind::Comma {
                            state.next();
                        } else {
                            break;
                        }
                    }

                    self.right_parenthesis(state)?;

                    let end_token = if state.current.kind == TokenKind::Colon {
                        self.colon(state)?;
                        TokenKind::EndFor
                    } else {
                        self.left_brace(state)?;
                        TokenKind::RightBrace
                    };

                    let then = self.block(state, &end_token)?;

                    if end_token == TokenKind::EndFor {
                        self.skip(state, TokenKind::EndFor)?;
                        self.semicolon(state)?;
                    } else {
                        self.right_brace(state)?;
                    };

                    Statement::For {
                        init,
                        condition,
                        r#loop,
                        then,
                    }
                }
                TokenKind::Foreach => {
                    state.next();

                    self.left_parenthesis(state)?;

                    let expr = self.expression(state, Precedence::Lowest)?;

                    self.skip(state, TokenKind::As)?;

                    let mut by_ref = state.current.kind == TokenKind::Ampersand;
                    if by_ref {
                        state.next();
                    }

                    let mut key_var = None;
                    let mut value_var = self.expression(state, Precedence::Lowest)?;

                    if state.current.kind == TokenKind::DoubleArrow {
                        state.next();

                        key_var = Some(value_var.clone());

                        by_ref = state.current.kind == TokenKind::Ampersand;
                        if by_ref {
                            state.next();
                        }

                        value_var = self.expression(state, Precedence::Lowest)?;
                    }

                    self.right_parenthesis(state)?;

                    let end_token = if state.current.kind == TokenKind::Colon {
                        self.colon(state)?;
                        TokenKind::EndForeach
                    } else {
                        self.left_brace(state)?;
                        TokenKind::RightBrace
                    };

                    let body = self.block(state, &end_token)?;

                    if end_token == TokenKind::EndForeach {
                        self.skip(state, TokenKind::EndForeach)?;
                        self.semicolon(state)?;
                    } else {
                        self.right_brace(state)?;
                    }

                    Statement::Foreach {
                        expr,
                        by_ref,
                        key_var,
                        value_var,
                        body,
                    }
                }
                TokenKind::Switch => {
                    state.next();

                    self.left_parenthesis(state)?;

                    let condition = self.expression(state, Precedence::Lowest)?;

                    self.right_parenthesis(state)?;

                    let end_token = if state.current.kind == TokenKind::Colon {
                        self.colon(state)?;
                        TokenKind::EndSwitch
                    } else {
                        self.left_brace(state)?;
                        TokenKind::RightBrace
                    };

                    let mut cases = Vec::new();
                    while state.current.kind != end_token {
                        match state.current.kind {
                            TokenKind::Case => {
                                state.next();

                                let condition = self.expression(state, Precedence::Lowest)?;

                                self.skip_any_of(state, &[TokenKind::Colon, TokenKind::SemiColon])?;

                                let mut body = Block::new();

                                while state.current.kind != TokenKind::Case
                                    && state.current.kind != TokenKind::Default
                                    && state.current.kind != TokenKind::RightBrace
                                {
                                    body.push(self.statement(state)?);
                                    state.skip_comments();
                                }

                                cases.push(Case {
                                    condition: Some(condition),
                                    body,
                                });
                            }
                            TokenKind::Default => {
                                state.next();

                                self.skip_any_of(state, &[TokenKind::Colon, TokenKind::SemiColon])?;

                                let mut body = Block::new();

                                while state.current.kind != TokenKind::Case
                                    && state.current.kind != TokenKind::Default
                                    && state.current.kind != TokenKind::RightBrace
                                {
                                    body.push(self.statement(state)?);
                                }

                                cases.push(Case {
                                    condition: None,
                                    body,
                                });
                            }
                            _ => {
                                return expected_token_err!(["`case`", "`default`"], state);
                            }
                        }
                    }

                    if end_token == TokenKind::EndSwitch {
                        self.skip(state, TokenKind::EndSwitch)?;
                        self.semicolon(state)?;
                    } else {
                        self.right_brace(state)?;
                    }

                    Statement::Switch { condition, cases }
                }
                TokenKind::If => {
                    state.next();

                    self.left_parenthesis(state)?;

                    let condition = self.expression(state, Precedence::Lowest)?;

                    self.right_parenthesis(state)?;

                    // FIXME: Tidy up duplication and make the intent a bit clearer.
                    match state.current.kind {
                        TokenKind::Colon => {
                            state.next();

                            let mut then = vec![];
                            while !matches!(
                                state.current.kind,
                                TokenKind::ElseIf | TokenKind::Else | TokenKind::EndIf
                            ) {
                                then.push(self.statement(state)?);
                            }

                            let mut else_ifs = vec![];
                            loop {
                                if state.current.kind != TokenKind::ElseIf {
                                    break;
                                }

                                state.next();

                                self.left_parenthesis(state)?;
                                let condition = self.expression(state, Precedence::Lowest)?;
                                self.right_parenthesis(state)?;

                                self.colon(state)?;

                                let mut body = vec![];
                                while !matches!(
                                    state.current.kind,
                                    TokenKind::ElseIf | TokenKind::Else | TokenKind::EndIf
                                ) {
                                    body.push(self.statement(state)?);
                                }

                                else_ifs.push(ElseIf { condition, body });
                            }

                            let mut r#else = None;
                            if state.current.kind == TokenKind::Else {
                                state.next();
                                self.colon(state)?;

                                let mut body = vec![];
                                while state.current.kind != TokenKind::EndIf {
                                    body.push(self.statement(state)?);
                                }
                                r#else = Some(body);
                            }

                            self.skip(state, TokenKind::EndIf)?;

                            self.semicolon(state)?;

                            Statement::If {
                                condition,
                                then,
                                else_ifs,
                                r#else,
                            }
                        }
                        _ => {
                            let body_end_token = if state.current.kind == TokenKind::LeftBrace {
                                state.next();

                                TokenKind::RightBrace
                            } else {
                                TokenKind::SemiColon
                            };

                            let then = self.block(state, &body_end_token)?;

                            if body_end_token == TokenKind::RightBrace {
                                self.right_brace(state)?;
                            }

                            let mut else_ifs: Vec<ElseIf> = Vec::new();
                            loop {
                                if state.current.kind == TokenKind::ElseIf {
                                    state.next();

                                    self.left_parenthesis(state)?;

                                    let condition = self.expression(state, Precedence::Lowest)?;

                                    self.right_parenthesis(state)?;

                                    self.left_brace(state)?;

                                    let body = self.block(state, &TokenKind::RightBrace)?;

                                    self.right_brace(state)?;

                                    else_ifs.push(ElseIf { condition, body });
                                } else {
                                    break;
                                }
                            }

                            if state.current.kind != TokenKind::Else {
                                return Ok(Statement::If {
                                    condition,
                                    then,
                                    else_ifs,
                                    r#else: None,
                                });
                            }

                            self.skip(state, TokenKind::Else)?;

                            self.left_brace(state)?;

                            let r#else = self.block(state, &TokenKind::RightBrace)?;

                            self.right_brace(state)?;

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
                    state.next();

                    let mut values = Vec::new();
                    loop {
                        values.push(self.expression(state, Precedence::Lowest)?);

                        if state.current.kind == TokenKind::Comma {
                            state.next();
                        } else {
                            break;
                        }
                    }

                    self.semicolon(state)?;
                    Statement::Echo { values }
                }
                TokenKind::Continue => {
                    state.next();

                    let mut num = None;
                    if state.current.kind != TokenKind::SemiColon {
                        num = Some(self.expression(state, Precedence::Lowest)?);
                    }

                    self.semicolon(state)?;

                    Statement::Continue { num }
                }
                TokenKind::Break => {
                    state.next();

                    let mut num = None;
                    if state.current.kind != TokenKind::SemiColon {
                        num = Some(self.expression(state, Precedence::Lowest)?);
                    }

                    self.semicolon(state)?;

                    Statement::Break { num }
                }
                TokenKind::Return => {
                    state.next();

                    if let Token {
                        kind: TokenKind::SemiColon,
                        ..
                    } = state.current
                    {
                        let ret = Statement::Return { value: None };
                        self.semicolon(state)?;
                        ret
                    } else {
                        let ret = Statement::Return {
                            value: self.expression(state, Precedence::Lowest).ok(),
                        };
                        self.semicolon(state)?;
                        ret
                    }
                }
                TokenKind::SemiColon => {
                    state.next();

                    Statement::Noop
                }
                TokenKind::Try => self.try_block(state)?,
                TokenKind::LeftBrace => {
                    state.next();
                    let body = self.block(state, &TokenKind::RightBrace)?;
                    self.right_brace(state)?;
                    Statement::Block { body }
                }
                _ => {
                    let expr = self.expression(state, Precedence::Lowest)?;

                    self.semicolon(state)?;

                    Statement::Expression { expr }
                }
            }
        };

        state.skip_comments();

        Ok(statement)
    }

    fn expression(&self, state: &mut State, precedence: Precedence) -> ParseResult<Expression> {
        if state.is_eof() {
            return Err(ParseError::UnexpectedEndOfFile);
        }

        let has_attributes = self.gather_attributes(state)?;

        let mut left = if has_attributes {
            match &state.current.kind {
                TokenKind::Static if state.peek.kind == TokenKind::Function => {
                    self.anonymous_function(state)?
                }
                TokenKind::Static if state.peek.kind == TokenKind::Fn => {
                    self.arrow_function(state)?
                }
                TokenKind::Function => self.anonymous_function(state)?,
                TokenKind::Fn => self.arrow_function(state)?,
                _ => {
                    // Note, we can get attributes and know their span, maybe use that in the
                    // error in the future?
                    return Err(ParseError::ExpectedItemDefinitionAfterAttributes(
                        state.current.span,
                    ));
                }
            }
        } else {
            match &state.current.kind {
                TokenKind::List => {
                    state.next();
                    self.left_parenthesis(state)?;

                    let mut items = Vec::new();
                    let mut has_atleast_one_key = false;

                    while state.current.kind != TokenKind::RightParen {
                        if state.current.kind == TokenKind::Comma {
                            items.push(ListItem {
                                key: None,
                                value: Expression::Empty,
                            });
                            state.next();
                            continue;
                        }

                        let mut key = None;

                        if state.current.kind == TokenKind::Ellipsis {
                            return Err(ParseError::IllegalSpreadOperator(state.current.span));
                        }

                        if state.current.kind == TokenKind::Ampersand {
                            return Err(ParseError::CannotAssignReferenceToNonReferencableValue(
                                state.current.span,
                            ));
                        }

                        let mut value = self.expression(state, Precedence::Lowest)?;

                        if state.current.kind == TokenKind::DoubleArrow {
                            if !has_atleast_one_key && !items.is_empty() {
                                return Err(ParseError::CannotMixKeyedAndUnkeyedEntries(
                                    state.current.span,
                                ));
                            }

                            state.next();

                            key = Some(value);

                            if state.current.kind == TokenKind::Ellipsis {
                                return Err(ParseError::IllegalSpreadOperator(state.current.span));
                            }

                            if state.current.kind == TokenKind::Ampersand {
                                return Err(
                                    ParseError::CannotAssignReferenceToNonReferencableValue(
                                        state.current.span,
                                    ),
                                );
                            }

                            has_atleast_one_key = true;
                            value = self.expression(state, Precedence::Lowest)?;
                        } else if has_atleast_one_key {
                            return Err(ParseError::CannotMixKeyedAndUnkeyedEntries(
                                state.current.span,
                            ));
                        }

                        items.push(ListItem { key, value });

                        state.skip_comments();
                        if state.current.kind == TokenKind::Comma {
                            state.next();
                            state.skip_comments();
                        } else {
                            break;
                        }
                    }

                    self.right_parenthesis(state)?;

                    Expression::List { items }
                }
                TokenKind::Static if state.peek.kind == TokenKind::Function => {
                    self.anonymous_function(state)?
                }
                TokenKind::Static if state.peek.kind == TokenKind::Fn => {
                    self.arrow_function(state)?
                }
                TokenKind::Function => self.anonymous_function(state)?,
                TokenKind::Fn => self.arrow_function(state)?,
                TokenKind::New
                    if state.peek.kind == TokenKind::Class
                        || state.peek.kind == TokenKind::Attribute =>
                {
                    self.anonymous_class_definition(state)?
                }
                TokenKind::Throw => {
                    state.next();

                    let value = self.expression(state, Precedence::Lowest)?;

                    Expression::Throw {
                        value: Box::new(value),
                    }
                }
                TokenKind::Yield => {
                    state.next();

                    if state.current.kind == TokenKind::SemiColon {
                        Expression::Yield {
                            key: None,
                            value: None,
                        }
                    } else {
                        let mut from = false;

                        if state.current.kind == TokenKind::From {
                            state.next();
                            from = true;
                        }

                        let mut key = None;
                        let mut value = Box::new(self.expression(
                            state,
                            if from {
                                Precedence::YieldFrom
                            } else {
                                Precedence::Yield
                            },
                        )?);

                        if state.current.kind == TokenKind::DoubleArrow && !from {
                            state.next();
                            key = Some(value.clone());
                            value = Box::new(self.expression(state, Precedence::Yield)?);
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
                    state.next();

                    let target = self.expression(state, Precedence::CloneOrNew)?;

                    Expression::Clone {
                        target: Box::new(target),
                    }
                }
                TokenKind::Variable(_) => Expression::Variable(self.var(state)?),
                TokenKind::LiteralInteger(i) => {
                    let e = Expression::LiteralInteger { i: *i };
                    state.next();
                    e
                }
                TokenKind::LiteralFloat(f) => {
                    let f = Expression::LiteralFloat { f: *f };
                    state.next();
                    f
                }
                TokenKind::Identifier(_)
                | TokenKind::QualifiedIdentifier(_)
                | TokenKind::FullyQualifiedIdentifier(_) => {
                    Expression::Identifier(self.full_name(state)?)
                }
                TokenKind::Static => {
                    state.next();
                    Expression::Static
                }
                TokenKind::LiteralString(s) => {
                    let e = Expression::LiteralString { value: s.clone() };
                    state.next();
                    e
                }
                TokenKind::StringPart(_) => self.interpolated_string(state)?,
                TokenKind::StartDocString(_, kind) => {
                    let kind = *kind;

                    self.doc_string(state, kind)?
                }
                TokenKind::Backtick => self.shell_exec(state)?,
                TokenKind::True => {
                    let e = Expression::Bool { value: true };
                    state.next();
                    e
                }
                TokenKind::False => {
                    let e = Expression::Bool { value: false };
                    state.next();
                    e
                }
                TokenKind::Null => {
                    state.next();
                    Expression::Null
                }
                TokenKind::LeftParen => {
                    state.next();

                    let e = self.expression(state, Precedence::Lowest)?;

                    self.right_parenthesis(state)?;

                    e
                }
                TokenKind::Match => {
                    state.next();
                    self.left_parenthesis(state)?;

                    let condition = Box::new(self.expression(state, Precedence::Lowest)?);

                    self.right_parenthesis(state)?;
                    self.left_brace(state)?;

                    let mut default = None;
                    let mut arms = Vec::new();
                    while state.current.kind != TokenKind::RightBrace {
                        state.skip_comments();

                        if state.current.kind == TokenKind::Default {
                            if default.is_some() {
                                return Err(ParseError::MatchExpressionWithMultipleDefaultArms(
                                    state.current.span,
                                ));
                            }

                            state.next();

                            // match conditions can have an extra comma at the end, including `default`.
                            if state.current.kind == TokenKind::Comma {
                                state.next();
                            }

                            self.double_arrow(state)?;

                            let body = self.expression(state, Precedence::Lowest)?;

                            default = Some(Box::new(DefaultMatchArm { body }));
                        } else {
                            let mut conditions = Vec::new();
                            while state.current.kind != TokenKind::DoubleArrow {
                                conditions.push(self.expression(state, Precedence::Lowest)?);

                                if state.current.kind == TokenKind::Comma {
                                    state.next();
                                } else {
                                    break;
                                }
                            }

                            if !conditions.is_empty() {
                                self.double_arrow(state)?;
                            } else {
                                break;
                            }

                            let body = self.expression(state, Precedence::Lowest)?;

                            arms.push(MatchArm { conditions, body });
                        }

                        if state.current.kind == TokenKind::Comma {
                            state.next();
                        } else {
                            break;
                        }
                    }

                    self.right_brace(state)?;

                    Expression::Match {
                        condition,
                        default,
                        arms,
                    }
                }
                TokenKind::Array => {
                    let mut items = vec![];

                    state.next();

                    self.left_parenthesis(state)?;

                    while state.current.kind != TokenKind::RightParen {
                        let mut key = None;
                        let unpack = if state.current.kind == TokenKind::Ellipsis {
                            state.next();
                            true
                        } else {
                            false
                        };

                        let (mut by_ref, amper_span) = if state.current.kind == TokenKind::Ampersand
                        {
                            let span = state.current.span;
                            state.next();
                            (true, span)
                        } else {
                            (false, (0, 0))
                        };

                        let mut value = self.expression(state, Precedence::Lowest)?;

                        // TODO: return error for `[...$a => $b]`.
                        if state.current.kind == TokenKind::DoubleArrow {
                            state.next();

                            if by_ref {
                                return Err(ParseError::UnexpectedToken(
                                    TokenKind::Ampersand.to_string(),
                                    amper_span,
                                ));
                            }

                            key = Some(value);

                            by_ref = if state.current.kind == TokenKind::Ampersand {
                                state.next();
                                true
                            } else {
                                false
                            };

                            value = self.expression(state, Precedence::Lowest)?;
                        }

                        items.push(ArrayItem {
                            key,
                            value,
                            unpack,
                            by_ref,
                        });

                        if state.current.kind == TokenKind::Comma {
                            state.next();
                        } else {
                            break;
                        }

                        state.skip_comments();
                    }

                    self.right_parenthesis(state)?;

                    Expression::Array { items }
                }
                TokenKind::LeftBracket => {
                    let mut items = Vec::new();
                    state.next();

                    state.skip_comments();

                    while state.current.kind != TokenKind::RightBracket {
                        // TODO: return an error here instead of
                        // an empty array element
                        // see: https://3v4l.org/uLTVA
                        if state.current.kind == TokenKind::Comma {
                            items.push(ArrayItem {
                                key: None,
                                value: Expression::Empty,
                                unpack: false,
                                by_ref: false,
                            });
                            state.next();
                            continue;
                        }

                        let mut key = None;
                        let unpack = if state.current.kind == TokenKind::Ellipsis {
                            state.next();
                            true
                        } else {
                            false
                        };

                        let (mut by_ref, amper_span) = if state.current.kind == TokenKind::Ampersand
                        {
                            let span = state.current.span;
                            state.next();
                            (true, span)
                        } else {
                            (false, (0, 0))
                        };

                        let mut value = self.expression(state, Precedence::Lowest)?;
                        if state.current.kind == TokenKind::DoubleArrow {
                            state.next();

                            if by_ref {
                                return Err(ParseError::UnexpectedToken(
                                    TokenKind::Ampersand.to_string(),
                                    amper_span,
                                ));
                            }

                            key = Some(value);
                            by_ref = if state.current.kind == TokenKind::Ampersand {
                                state.next();
                                true
                            } else {
                                false
                            };
                            value = self.expression(state, Precedence::Lowest)?;
                        }

                        items.push(ArrayItem {
                            key,
                            value,
                            unpack,
                            by_ref,
                        });

                        state.skip_comments();
                        if state.current.kind == TokenKind::Comma {
                            state.next();
                            state.skip_comments();
                        } else {
                            break;
                        }
                    }

                    state.skip_comments();

                    self.right_bracket(state)?;

                    Expression::Array { items }
                }
                TokenKind::New => {
                    state.next();

                    let mut args = vec![];
                    let target = self.expression(state, Precedence::CloneOrNew)?;

                    if state.current.kind == TokenKind::LeftParen {
                        args = self.args_list(state)?;
                    }

                    Expression::New {
                        target: Box::new(target),
                        args,
                    }
                }
                TokenKind::DirConstant => {
                    state.next();
                    Expression::MagicConst {
                        constant: MagicConst::Dir,
                    }
                }
                TokenKind::Include
                | TokenKind::IncludeOnce
                | TokenKind::Require
                | TokenKind::RequireOnce => {
                    let kind: IncludeKind = (&state.current.kind).into();
                    state.next();

                    let path = self.expression(state, Precedence::Lowest)?;

                    Expression::Include {
                        kind,
                        path: Box::new(path),
                    }
                }
                _ if is_prefix(&state.current.kind) => {
                    let op = state.current.kind.clone();

                    state.next();

                    let rpred = Precedence::prefix(&op);
                    let rhs = self.expression(state, rpred)?;

                    prefix(&op, rhs)
                }
                TokenKind::Dollar => self.dynamic_variable(state)?,
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        state.current.kind.to_string(),
                        state.current.span,
                    ))
                }
            }
        };

        if state.current.kind == TokenKind::SemiColon {
            return Ok(left);
        }

        state.skip_comments();

        loop {
            state.skip_comments();

            if matches!(state.current.kind, TokenKind::SemiColon | TokenKind::Eof) {
                break;
            }

            let span = state.current.span;
            let kind = state.current.kind.clone();

            if is_postfix(&kind) {
                let lpred = Precedence::postfix(&kind);

                if lpred < precedence {
                    break;
                }

                left = self.postfix(state, left, &kind)?;
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

                state.next();

                match kind {
                    TokenKind::Question => {
                        let then = self.expression(state, Precedence::Lowest)?;
                        self.colon(state)?;
                        let otherwise = self.expression(state, rpred)?;
                        left = Expression::Ternary {
                            condition: Box::new(left),
                            then: Some(Box::new(then)),
                            r#else: Box::new(otherwise),
                        }
                    }
                    TokenKind::QuestionColon => {
                        let r#else = self.expression(state, Precedence::Lowest)?;
                        left = Expression::Ternary {
                            condition: Box::new(left),
                            then: None,
                            r#else: Box::new(r#else),
                        }
                    }
                    _ => {
                        // FIXME: Hacky, should probably be refactored.
                        let by_ref =
                            kind == TokenKind::Equals && state.current.kind == TokenKind::Ampersand;
                        if by_ref {
                            state.next();
                        }

                        let rhs = self.expression(state, rpred)?;

                        left = infix(left, kind, rhs, by_ref);
                    }
                }

                continue;
            }

            break;
        }

        state.skip_comments();

        Ok(left)
    }

    fn postfix(
        &self,
        state: &mut State,
        lhs: Expression,
        op: &TokenKind,
    ) -> Result<Expression, ParseError> {
        Ok(match op {
            TokenKind::Coalesce => {
                state.next();

                let rhs = self.expression(state, Precedence::NullCoalesce)?;

                Expression::Coalesce {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }
            }
            TokenKind::LeftParen => {
                let args = self.args_list(state)?;

                Expression::Call {
                    target: Box::new(lhs),
                    args,
                }
            }
            TokenKind::LeftBracket => {
                state.next();

                if state.current.kind == TokenKind::RightBracket {
                    state.next();

                    Expression::ArrayIndex {
                        array: Box::new(lhs),
                        index: None,
                    }
                } else {
                    let index = self.expression(state, Precedence::Lowest)?;

                    self.right_bracket(state)?;

                    Expression::ArrayIndex {
                        array: Box::new(lhs),
                        index: Some(Box::new(index)),
                    }
                }
            }
            TokenKind::DoubleColon => {
                state.next();

                let mut must_be_method_call = false;

                let property = match state.current.kind.clone() {
                    TokenKind::Dollar => self.dynamic_variable(state)?,
                    TokenKind::Variable(_) => Expression::Variable(self.var(state)?),
                    TokenKind::Identifier(_) => Expression::Identifier(self.ident(state)?),
                    TokenKind::LeftBrace => {
                        must_be_method_call = true;
                        state.next();

                        let name = self.expression(state, Precedence::Lowest)?;

                        self.right_brace(state)?;

                        Expression::DynamicVariable {
                            name: Box::new(name),
                        }
                    }
                    TokenKind::Class => {
                        let start = state.current.span;
                        state.next();
                        let end = state.current.span;

                        Expression::Identifier(Identifier {
                            start,
                            name: "class".into(),
                            end,
                        })
                    }
                    _ if is_reserved_ident(&state.current.kind) => {
                        Expression::Identifier(self.ident_maybe_reserved(state)?)
                    }
                    _ => {
                        return expected_token_err!(["`{`", "`$`", "an identifier"], state);
                    }
                };

                let lhs = Box::new(lhs);

                match property {
                    // 1. If we have an identifier and the current token is not a left paren,
                    //    the resulting expression must be a constant fetch.
                    Expression::Identifier(identifier)
                        if state.current.kind != TokenKind::LeftParen =>
                    {
                        Expression::ConstFetch {
                            target: lhs,
                            constant: identifier,
                        }
                    }
                    // 2. If the current token is a left paren, or if we know the property expression
                    //    is only valid a method call context, we can assume we're parsing a static
                    //    method call.
                    _ if state.current.kind == TokenKind::LeftParen || must_be_method_call => {
                        let args = self.args_list(state)?;

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
                state.next();

                let property = match state.current.kind {
                    TokenKind::LeftBrace => {
                        self.left_brace(state)?;
                        let expr = self.expression(state, Precedence::Lowest)?;
                        self.right_brace(state)?;
                        expr
                    }
                    TokenKind::Variable(_) => Expression::Variable(self.var(state)?),
                    TokenKind::Dollar => self.dynamic_variable(state)?,
                    _ => Expression::Identifier(self.ident_maybe_reserved(state)?),
                };

                if state.current.kind == TokenKind::LeftParen {
                    let args = self.args_list(state)?;

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
            TokenKind::Increment => {
                state.next();
                Expression::Increment {
                    value: Box::new(lhs),
                }
            }
            TokenKind::Decrement => {
                state.next();

                Expression::Decrement {
                    value: Box::new(lhs),
                }
            }
            _ => todo!("postfix: {:?}", op),
        })
    }

    fn interpolated_string(&self, state: &mut State) -> ParseResult<Expression> {
        let mut parts = Vec::new();

        while state.current.kind != TokenKind::DoubleQuote {
            if let Some(part) = self.interpolated_string_part(state)? {
                parts.push(part);
            }
        }

        state.next();

        Ok(Expression::InterpolatedString { parts })
    }

    fn shell_exec(&self, state: &mut State) -> ParseResult<Expression> {
        state.next();

        let mut parts = Vec::new();

        while state.current.kind != TokenKind::Backtick {
            if let Some(part) = self.interpolated_string_part(state)? {
                parts.push(part);
            }
        }

        state.next();

        Ok(Expression::ShellExec { parts })
    }

    fn doc_string(&self, state: &mut State, kind: DocStringKind) -> ParseResult<Expression> {
        state.next();

        Ok(match kind {
            DocStringKind::Heredoc => {
                let mut parts = Vec::new();

                while !matches!(state.current.kind, TokenKind::EndDocString(_, _, _)) {
                    if let Some(part) = self.interpolated_string_part(state)? {
                        parts.push(part);
                    }
                }

                let (indentation_type, indentation_amount) = match state.current.kind {
                    TokenKind::EndDocString(_, indentation_type, indentation_amount) => {
                        (indentation_type, indentation_amount)
                    }
                    _ => unreachable!(),
                };

                state.next();

                // FIXME: Can we move this logic above into the loop, by peeking ahead in
                //        the token stream for the EndHeredoc? Might be more performant.
                if let Some(indentation_type) = indentation_type {
                    let search_char: u8 = indentation_type.into();

                    for part in parts.iter_mut() {
                        match part {
                            StringPart::Const(bytes) => {
                                for _ in 0..indentation_amount {
                                    if bytes.starts_with(&[search_char]) {
                                        bytes.remove(0);
                                    }
                                }
                            }
                            _ => continue,
                        }
                    }
                }

                Expression::Heredoc { parts }
            }
            DocStringKind::Nowdoc => {
                // FIXME: This feels hacky. We should probably produce different tokens from the lexer
                //        but since I already had the logic in place for parsing heredocs, this was
                //        the fastest way to get nowdocs working too.
                let mut s = expect_token!([
                    TokenKind::StringPart(s) => s
                ], state, "constant string");

                let (indentation_type, indentation_amount) = expect_token!([
                    TokenKind::EndDocString(_, indentation_type, indentation_amount) => (indentation_type, indentation_amount)
                ], state, "label");

                // FIXME: Hacky code, but it's late and I want to get this done.
                if let Some(indentation_type) = indentation_type {
                    let search_char: u8 = indentation_type.into();
                    let mut lines = s
                        .split(|b| *b == b'\n')
                        .map(|s| s.to_vec())
                        .collect::<Vec<Vec<u8>>>();
                    for line in lines.iter_mut() {
                        for _ in 0..indentation_amount {
                            if line.starts_with(&[search_char]) {
                                line.remove(0);
                            }
                        }
                    }
                    let mut bytes = Vec::new();
                    for (i, line) in lines.iter().enumerate() {
                        bytes.extend(line);
                        if i < lines.len() - 1 {
                            bytes.push(b'\n');
                        }
                    }
                    s = bytes.into();
                }

                Expression::Nowdoc { value: s }
            }
        })
    }

    fn interpolated_string_part(&self, state: &mut State) -> ParseResult<Option<StringPart>> {
        Ok(match &state.current.kind {
            TokenKind::StringPart(s) => {
                let part = if s.len() > 0 {
                    Some(StringPart::Const(s.clone()))
                } else {
                    None
                };

                state.next();
                part
            }
            TokenKind::DollarLeftBrace => {
                state.next();
                let e = match (state.current.kind.clone(), state.peek.kind.clone()) {
                    (TokenKind::Identifier(name), TokenKind::RightBrace) => {
                        let start = state.current.span;
                        let end = state.peek.span;

                        state.next();
                        state.next();
                        // "${var}"
                        // TODO: we should use a different node for this.
                        Expression::Variable(Variable { start, name, end })
                    }
                    (TokenKind::Identifier(name), TokenKind::LeftBracket) => {
                        let start = state.current.span;
                        let end = state.peek.span;
                        state.next();
                        state.next();
                        let var = Expression::Variable(Variable { start, name, end });

                        let e = self.expression(state, Precedence::Lowest)?;
                        self.right_bracket(state)?;
                        self.right_brace(state)?;

                        // TODO: we should use a different node for this.
                        Expression::ArrayIndex {
                            array: Box::new(var),
                            index: Some(Box::new(e)),
                        }
                    }
                    _ => {
                        // Arbitrary expressions are allowed, but are treated as variable variables.
                        let e = self.expression(state, Precedence::Lowest)?;
                        self.right_brace(state)?;

                        Expression::DynamicVariable { name: Box::new(e) }
                    }
                };
                Some(StringPart::Expr(Box::new(e)))
            }
            TokenKind::LeftBrace => {
                // "{$expr}"
                state.next();
                let e = self.expression(state, Precedence::Lowest)?;
                self.right_brace(state)?;
                Some(StringPart::Expr(Box::new(e)))
            }
            TokenKind::Variable(_) => {
                // "$expr", "$expr[0]", "$expr[name]", "$expr->a"
                let var = Expression::Variable(self.var(state)?);
                let e = match state.current.kind {
                    TokenKind::LeftBracket => {
                        state.next();
                        // Full expression syntax is not allowed here,
                        // so we can't call self.expression.
                        let index = match &state.current.kind {
                            &TokenKind::LiteralInteger(i) => {
                                state.next();
                                Expression::LiteralInteger { i }
                            }
                            TokenKind::Minus => {
                                state.next();
                                if let TokenKind::LiteralInteger(i) = state.current.kind {
                                    state.next();
                                    Expression::Negate {
                                        value: Box::new(Expression::LiteralInteger { i }),
                                    }
                                } else {
                                    return expected_token_err!("an integer", state);
                                }
                            }
                            TokenKind::Identifier(ident) => {
                                let e = Expression::LiteralString {
                                    value: ident.clone(),
                                };
                                state.next();
                                e
                            }
                            TokenKind::Variable(_) => {
                                let v = self.var(state)?;
                                Expression::Variable(v)
                            }
                            _ => {
                                return expected_token_err!(
                                    ["`-`", "an integer", "an identifier", "a variable"],
                                    state
                                );
                            }
                        };

                        self.right_bracket(state)?;

                        Expression::ArrayIndex {
                            array: Box::new(var),
                            index: Some(Box::new(index)),
                        }
                    }
                    TokenKind::Arrow => {
                        state.next();
                        Expression::PropertyFetch {
                            target: Box::new(var),
                            property: Box::new(Expression::Identifier(
                                self.ident_maybe_reserved(state)?,
                            )),
                        }
                    }
                    TokenKind::NullsafeArrow => {
                        state.next();
                        Expression::NullsafePropertyFetch {
                            target: Box::new(var),
                            property: Box::new(Expression::Identifier(
                                self.ident_maybe_reserved(state)?,
                            )),
                        }
                    }
                    _ => var,
                };
                Some(StringPart::Expr(Box::new(e)))
            }
            _ => {
                return expected_token_err!(["`${`", "`{$", "`\"`", "a variable"], state);
            }
        })
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

fn infix(lhs: Expression, op: TokenKind, rhs: Expression, by_ref: bool) -> Expression {
    Expression::Infix {
        lhs: Box::new(lhs),
        op: match (&op, by_ref) {
            (TokenKind::Equals, true) => ast::InfixOp::AssignRef,
            _ => op.into(),
        },
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
