use crate::expect_literal;
use crate::lexer::token::Token;
use crate::lexer::token::TokenKind;
use crate::parser::ast::comments::Comment;
use crate::parser::ast::comments::CommentFormat;
use crate::parser::ast::variables::Variable;
use crate::parser::ast::{DeclareItem, Expression, Program, Statement, StaticVar};
use crate::parser::error::ParseResult;
use crate::parser::internal::attributes;
use crate::parser::internal::blocks;
use crate::parser::internal::classes;
use crate::parser::internal::constants;
use crate::parser::internal::control_flow;
use crate::parser::internal::enums;
use crate::parser::internal::functions;
use crate::parser::internal::goto;
use crate::parser::internal::identifiers;
use crate::parser::internal::interfaces;
use crate::parser::internal::loops;
use crate::parser::internal::namespaces;
use crate::parser::internal::traits;
use crate::parser::internal::try_block;
use crate::parser::internal::uses;
use crate::parser::internal::utils;
use crate::parser::internal::variables;
use crate::parser::state::State;

pub mod ast;
pub mod error;

mod expressions;
mod internal;
mod macros;
mod state;

pub fn parse(tokens: Vec<Token>) -> ParseResult<Program> {
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

        if state.current.kind == TokenKind::CloseTag {
            state.next();
            continue;
        }

        ast.push(top_level_statement(&mut state)?);

        state.clear_comments();
    }

    Ok(ast.to_vec())
}

fn top_level_statement(state: &mut State) -> ParseResult<Statement> {
    state.skip_comments();

    let statement = match &state.current.kind {
        TokenKind::Namespace => namespaces::namespace(state)?,
        TokenKind::Use => uses::use_statement(state)?,
        TokenKind::Const => Statement::Constant(constants::parse(state)?),
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
        _ => statement(state)?,
    };

    state.clear_comments();

    // A closing PHP tag is valid after the end of any top-level statement.
    if state.current.kind == TokenKind::CloseTag {
        state.next();
    }

    Ok(statement)
}

fn statement(state: &mut State) -> ParseResult<Statement> {
    let has_attributes = attributes::gather_attributes(state)?;

    // FIXME: There's a better place to put this but night-time brain doesn't know where.
    utils::skip_open_tag(state)?;

    let statement = if has_attributes {
        match &state.current.kind {
            TokenKind::Abstract => classes::parse(state)?,
            TokenKind::Readonly if state.peek.kind != TokenKind::LeftParen => {
                classes::parse(state)?
            }
            TokenKind::Final => classes::parse(state)?,
            TokenKind::Class => classes::parse(state)?,
            TokenKind::Interface => interfaces::parse(state)?,
            TokenKind::Trait => traits::parse(state)?,
            TokenKind::Enum
                if state.peek.kind != TokenKind::LeftParen
                    && state.peek.kind != TokenKind::DoubleColon
                    && state.peek.kind != TokenKind::Colon =>
            {
                enums::parse(state)?
            }
            TokenKind::Function
                if identifiers::is_identifier_maybe_soft_reserved(&state.peek.kind)
                    || state.peek.kind == TokenKind::Ampersand =>
            {
                // FIXME: This is incredibly hacky but we don't have a way to look at
                // the next N tokens right now. We could probably do with a `peek_buf()`
                // method like the Lexer has.
                if state.peek.kind == TokenKind::Ampersand {
                    if let Some((_, token)) = state.iter.clone().enumerate().next() {
                        if !matches!(
                            token,
                            Token {
                                kind: TokenKind::Identifier(_),
                                ..
                            }
                        ) {
                            let expr = expressions::lowest_precedence(state)?;

                            utils::skip_semicolon(state)?;

                            return Ok(Statement::Expression { expr });
                        }
                    }

                    functions::function(state)?
                } else {
                    functions::function(state)?
                }
            }
            _ => Statement::Expression {
                expr: expressions::attributes(state)?,
            },
        }
    } else {
        match &state.current.kind {
            TokenKind::Abstract => classes::parse(state)?,
            TokenKind::Readonly if state.peek.kind != TokenKind::LeftParen => {
                classes::parse(state)?
            }
            TokenKind::Final => classes::parse(state)?,
            TokenKind::Class => classes::parse(state)?,
            TokenKind::Interface => interfaces::parse(state)?,
            TokenKind::Trait => traits::parse(state)?,
            TokenKind::Enum
                if state.peek.kind != TokenKind::LeftParen
                    && state.peek.kind != TokenKind::DoubleColon
                    && state.peek.kind != TokenKind::Colon =>
            {
                enums::parse(state)?
            }
            TokenKind::Function
                if identifiers::is_identifier_maybe_soft_reserved(&state.peek.kind)
                    || state.peek.kind == TokenKind::Ampersand =>
            {
                // FIXME: This is incredibly hacky but we don't have a way to look at
                // the next N tokens right now. We could probably do with a `peek_buf()`
                // method like the Lexer has.
                if state.peek.kind == TokenKind::Ampersand {
                    if let Some((_, token)) = state.iter.clone().enumerate().next() {
                        if !matches!(
                            token,
                            Token {
                                kind: TokenKind::Identifier(_),
                                ..
                            }
                        ) {
                            let expr = expressions::lowest_precedence(state)?;

                            utils::skip_semicolon(state)?;

                            return Ok(Statement::Expression { expr });
                        }
                    }

                    functions::function(state)?
                } else {
                    functions::function(state)?
                }
            }
            TokenKind::Goto => goto::goto_statement(state)?,
            token
                if identifiers::is_identifier_maybe_reserved(token)
                    && state.peek.kind == TokenKind::Colon =>
            {
                goto::label_statement(state)?
            }
            TokenKind::Declare => {
                state.next();
                utils::skip_left_parenthesis(state)?;

                let mut declares = Vec::new();
                loop {
                    let key = identifiers::identifier(state)?;

                    utils::skip(state, TokenKind::Equals)?;

                    let value = expect_literal!(state);

                    declares.push(DeclareItem { key, value });

                    if state.current.kind == TokenKind::Comma {
                        state.next();
                    } else {
                        break;
                    }
                }

                utils::skip_right_parenthesis(state)?;

                let body = if state.current.kind == TokenKind::LeftBrace {
                    state.next();
                    let b = blocks::body(state, &TokenKind::RightBrace)?;
                    utils::skip_right_brace(state)?;
                    b
                } else if state.current.kind == TokenKind::Colon {
                    utils::skip_colon(state)?;
                    let b = blocks::body(state, &TokenKind::EndDeclare)?;
                    utils::skip(state, TokenKind::EndDeclare)?;
                    utils::skip_semicolon(state)?;
                    b
                } else if state.current.kind == TokenKind::SemiColon {
                    utils::skip_semicolon(state)?;
                    vec![]
                } else {
                    vec![statement(state)?]
                };

                Statement::Declare { declares, body }
            }
            TokenKind::Global => {
                let span = state.current.span;
                state.next();

                let mut variables = vec![];
                // `loop` instead of `while` as we don't allow for extra commas.
                loop {
                    variables.push(variables::dynamic_variable(state)?);

                    if state.current.kind == TokenKind::Comma {
                        state.next();
                    } else {
                        break;
                    }
                }

                utils::skip_semicolon(state)?;
                Statement::Global { span, variables }
            }
            TokenKind::Static if matches!(state.peek.kind, TokenKind::Variable(_)) => {
                state.next();

                let mut vars = vec![];

                // `loop` instead of `while` as we don't allow for extra commas.
                loop {
                    let var = variables::simple_variable(state)?;
                    let mut default = None;

                    if state.current.kind == TokenKind::Equals {
                        state.next();

                        default = Some(expressions::lowest_precedence(state)?);
                    }

                    // TODO: group static vars.
                    vars.push(StaticVar {
                        var: Variable::SimpleVariable(var),
                        default,
                    });

                    if state.current.kind == TokenKind::Comma {
                        state.next();
                    } else {
                        break;
                    }
                }

                utils::skip_semicolon(state)?;

                Statement::Static { vars }
            }
            TokenKind::InlineHtml(html) => {
                let s = Statement::InlineHtml(html.clone());
                state.next();
                utils::skip_open_tag(state)?;
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
            TokenKind::Do => loops::do_loop(state)?,
            TokenKind::While => loops::while_loop(state)?,
            TokenKind::For => loops::for_loop(state)?,
            TokenKind::Foreach => loops::foreach_loop(state)?,
            TokenKind::Continue => loops::continue_statement(state)?,
            TokenKind::Break => loops::break_statement(state)?,
            TokenKind::Switch => control_flow::switch_statement(state)?,
            TokenKind::If => control_flow::if_statement(state)?,
            TokenKind::Echo => {
                state.next();

                let mut values = Vec::new();
                loop {
                    values.push(expressions::lowest_precedence(state)?);

                    if state.current.kind == TokenKind::Comma {
                        state.next();
                    } else {
                        break;
                    }
                }

                utils::skip_semicolon(state)?;
                Statement::Echo { values }
            }
            TokenKind::Return => {
                state.next();

                if TokenKind::SemiColon == state.current.kind {
                    let ret = Statement::Return { value: None };
                    utils::skip_semicolon(state)?;
                    ret
                } else {
                    let ret = Statement::Return {
                        value: Some(expressions::lowest_precedence(state)?),
                    };
                    utils::skip_semicolon(state)?;
                    ret
                }
            }
            TokenKind::SemiColon => {
                let start = state.current.span;

                state.next();

                Statement::Noop(start)
            }
            TokenKind::Try => try_block::try_block(state)?,
            TokenKind::LeftBrace => blocks::block_statement(state)?,
            _ => {
                let expr = expressions::lowest_precedence(state)?;

                utils::skip_semicolon(state)?;

                Statement::Expression { expr }
            }
        }
    };

    state.skip_comments();

    // A closing PHP tag is valid after the end of any top-level statement.
    if state.current.kind == TokenKind::CloseTag {
        state.next();
    }

    Ok(statement)
}
