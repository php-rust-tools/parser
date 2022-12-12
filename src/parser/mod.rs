use crate::expect_literal;
use crate::lexer::token::OpenTagKind;
use crate::lexer::token::Token;
use crate::lexer::token::TokenKind;
use crate::parser::ast::comments::Comment;
use crate::parser::ast::comments::CommentFormat;
use crate::parser::ast::declares::Declare;
use crate::parser::ast::declares::DeclareBody;
use crate::parser::ast::declares::DeclareEntry;
use crate::parser::ast::declares::DeclareEntryGroup;
use crate::parser::ast::variables::Variable;
use crate::parser::ast::{Expression, Program, Statement, StaticVar};
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
use crate::parser::stream::TokenStream;

pub mod ast;
pub mod error;

mod expressions;
mod internal;
mod macros;
mod state;
mod stream;

pub fn parse(tokens: &[Token]) -> ParseResult<Program> {
    let mut stream = TokenStream::new(tokens);
    let mut state = State::new(&mut stream);

    let mut ast = Program::new();

    while !state.stream.is_eof() {
        if matches!(
            state.stream.current().kind,
            TokenKind::OpenTag(OpenTagKind::Full) | TokenKind::CloseTag
        ) {
            state.stream.next();
            continue;
        }

        if state.stream.is_eof() {
            break;
        }

        if state.stream.current().kind == TokenKind::CloseTag {
            state.stream.next();
            continue;
        }

        ast.push(top_level_statement(&mut state)?);
    }

    Ok(ast.to_vec())
}

fn top_level_statement(state: &mut State) -> ParseResult<Statement> {
    let statement = match &state.stream.current().kind {
        TokenKind::Namespace => namespaces::namespace(state)?,
        TokenKind::Use => uses::use_statement(state)?,
        TokenKind::Const => Statement::Constant(constants::parse(state)?),
        TokenKind::HaltCompiler => {
            state.stream.next();

            let content =
                if let TokenKind::InlineHtml(content) = state.stream.current().kind.clone() {
                    state.stream.next();
                    Some(content)
                } else {
                    None
                };

            Statement::HaltCompiler { content }
        }
        _ => statement(state)?,
    };

    // A closing PHP tag is valid after the end of any top-level statement.
    if state.stream.current().kind == TokenKind::CloseTag {
        state.stream.next();
    }

    Ok(statement)
}

fn statement(state: &mut State) -> ParseResult<Statement> {
    let has_attributes = attributes::gather_attributes(state)?;

    // FIXME: There's a better place to put this but night-time brain doesn't know where.
    utils::skip_open_tag(state)?;

    let statement = if has_attributes {
        match &state.stream.current().kind {
            TokenKind::Abstract => classes::parse(state)?,
            TokenKind::Readonly if state.stream.peek().kind != TokenKind::LeftParen => {
                classes::parse(state)?
            }
            TokenKind::Final => classes::parse(state)?,
            TokenKind::Class => classes::parse(state)?,
            TokenKind::Interface => interfaces::parse(state)?,
            TokenKind::Trait => traits::parse(state)?,
            TokenKind::Enum
                if state.stream.peek().kind != TokenKind::LeftParen
                    && state.stream.peek().kind != TokenKind::DoubleColon
                    && state.stream.peek().kind != TokenKind::Colon =>
            {
                enums::parse(state)?
            }
            TokenKind::Function
                if identifiers::is_identifier_maybe_soft_reserved(&state.stream.peek().kind)
                    || state.stream.peek().kind == TokenKind::Ampersand =>
            {
                if state.stream.peek().kind == TokenKind::Ampersand {
                    if !matches!(state.stream.lookahead(1).kind, TokenKind::Identifier(_),) {
                        let expression = expressions::lowest_precedence(state)?;
                        let end = utils::skip_semicolon(state)?;

                        return Ok(Statement::Expression { expression, end });
                    }

                    functions::function(state)?
                } else {
                    functions::function(state)?
                }
            }
            _ => {
                let expression = expressions::attributes(state)?;
                let end = utils::skip_semicolon(state)?;

                Statement::Expression { expression, end }
            }
        }
    } else {
        match &state.stream.current().kind {
            TokenKind::OpenTag(OpenTagKind::Echo) => {
                let span = state.stream.current().span;
                state.stream.next();

                let mut values = Vec::new();
                loop {
                    values.push(expressions::lowest_precedence(state)?);

                    if state.stream.current().kind == TokenKind::Comma {
                        state.stream.next();
                    } else {
                        break;
                    }
                }

                utils::skip_semicolon(state)?;

                Statement::ShortEcho { span, values }
            }
            TokenKind::Abstract => classes::parse(state)?,
            TokenKind::Readonly if state.stream.peek().kind != TokenKind::LeftParen => {
                classes::parse(state)?
            }
            TokenKind::Final => classes::parse(state)?,
            TokenKind::Class => classes::parse(state)?,
            TokenKind::Interface => interfaces::parse(state)?,
            TokenKind::Trait => traits::parse(state)?,
            TokenKind::Enum
                if state.stream.peek().kind != TokenKind::LeftParen
                    && state.stream.peek().kind != TokenKind::DoubleColon
                    && state.stream.peek().kind != TokenKind::Colon =>
            {
                enums::parse(state)?
            }
            TokenKind::Function
                if identifiers::is_identifier_maybe_soft_reserved(&state.stream.peek().kind)
                    || state.stream.peek().kind == TokenKind::Ampersand =>
            {
                if state.stream.peek().kind == TokenKind::Ampersand {
                    if !matches!(state.stream.lookahead(1).kind, TokenKind::Identifier(_),) {
                        let expression = expressions::lowest_precedence(state)?;
                        let end = utils::skip_semicolon(state)?;

                        return Ok(Statement::Expression { expression, end });
                    }

                    functions::function(state)?
                } else {
                    functions::function(state)?
                }
            }
            TokenKind::Goto => goto::goto_statement(state)?,
            token
                if identifiers::is_identifier_maybe_reserved(token)
                    && state.stream.peek().kind == TokenKind::Colon =>
            {
                goto::label_statement(state)?
            }
            TokenKind::Declare => {
                let span = utils::skip(state, TokenKind::Declare)?;

                let entries = {
                    let start = utils::skip_left_parenthesis(state)?;
                    let mut entries = Vec::new();
                    loop {
                        let key = identifiers::identifier(state)?;
                        let span = utils::skip(state, TokenKind::Equals)?;
                        let value = expect_literal!(state);

                        entries.push(DeclareEntry { key, span, value });

                        if state.stream.current().kind == TokenKind::Comma {
                            state.stream.next();
                        } else {
                            break;
                        }
                    }
                    let end = utils::skip_right_parenthesis(state)?;

                    DeclareEntryGroup {
                        start,
                        entries,
                        end,
                    }
                };

                let body = match state.stream.current().kind.clone() {
                    TokenKind::SemiColon => {
                        let span = utils::skip_semicolon(state)?;

                        DeclareBody::Noop { span }
                    }
                    TokenKind::LeftBrace => {
                        let start = utils::skip_left_brace(state)?;
                        let statements = blocks::body(state, &TokenKind::RightBrace)?;
                        let end = utils::skip_right_brace(state)?;

                        DeclareBody::Braced {
                            start,
                            statements,
                            end,
                        }
                    }
                    TokenKind::Colon => {
                        let start = utils::skip_colon(state)?;
                        let statements = blocks::body(state, &TokenKind::EndDeclare)?;
                        let end = (
                            utils::skip(state, TokenKind::EndDeclare)?,
                            utils::skip_semicolon(state)?,
                        );

                        DeclareBody::Block {
                            start,
                            statements,
                            end,
                        }
                    }
                    _ => {
                        let expression = expressions::lowest_precedence(state)?;
                        let end = utils::skip_semicolon(state)?;

                        DeclareBody::Expression { expression, end }
                    }
                };

                Statement::Declare(Declare {
                    span,
                    entries,
                    body,
                })
            }
            TokenKind::Global => {
                let span = state.stream.current().span;
                state.stream.next();

                let mut variables = vec![];
                // `loop` instead of `while` as we don't allow for extra commas.
                loop {
                    variables.push(variables::dynamic_variable(state)?);

                    if state.stream.current().kind == TokenKind::Comma {
                        state.stream.next();
                    } else {
                        break;
                    }
                }

                utils::skip_semicolon(state)?;
                Statement::Global { span, variables }
            }
            TokenKind::Static if matches!(state.stream.peek().kind, TokenKind::Variable(_)) => {
                state.stream.next();

                let mut vars = vec![];

                // `loop` instead of `while` as we don't allow for extra commas.
                loop {
                    let var = variables::simple_variable(state)?;
                    let mut default = None;

                    if state.stream.current().kind == TokenKind::Equals {
                        state.stream.next();

                        default = Some(expressions::lowest_precedence(state)?);
                    }

                    // TODO: group static vars.
                    vars.push(StaticVar {
                        var: Variable::SimpleVariable(var),
                        default,
                    });

                    if state.stream.current().kind == TokenKind::Comma {
                        state.stream.next();
                    } else {
                        break;
                    }
                }

                utils::skip_semicolon(state)?;

                Statement::Static { vars }
            }
            TokenKind::InlineHtml(html) => {
                let s = Statement::InlineHtml(html.clone());
                state.stream.next();
                utils::skip_open_tag(state)?;
                s
            }
            TokenKind::SingleLineComment(comment) => {
                let start = state.stream.current().span;
                let content = comment.clone();
                state.stream.next();
                let end = state.stream.current().span;
                let format = CommentFormat::SingleLine;

                Statement::Comment(Comment {
                    start,
                    end,
                    format,
                    content,
                })
            }
            TokenKind::MultiLineComment(comment) => {
                let start = state.stream.current().span;
                let content = comment.clone();
                state.stream.next();
                let end = state.stream.current().span;
                let format = CommentFormat::MultiLine;

                Statement::Comment(Comment {
                    start,
                    end,
                    format,
                    content,
                })
            }
            TokenKind::HashMarkComment(comment) => {
                let start = state.stream.current().span;
                let content = comment.clone();
                state.stream.next();
                let end = state.stream.current().span;
                let format = CommentFormat::HashMark;

                Statement::Comment(Comment {
                    start,
                    end,
                    format,
                    content,
                })
            }
            TokenKind::DocumentComment(comment) => {
                let start = state.stream.current().span;
                let content = comment.clone();
                state.stream.next();
                let end = state.stream.current().span;
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
                state.stream.next();

                let mut values = Vec::new();
                loop {
                    values.push(expressions::lowest_precedence(state)?);

                    if state.stream.current().kind == TokenKind::Comma {
                        state.stream.next();
                    } else {
                        break;
                    }
                }

                utils::skip_semicolon(state)?;
                Statement::Echo { values }
            }
            TokenKind::Return => {
                state.stream.next();

                if TokenKind::SemiColon == state.stream.current().kind {
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
                let start = state.stream.current().span;

                state.stream.next();

                Statement::Noop(start)
            }
            TokenKind::Try => try_block::try_block(state)?,
            TokenKind::LeftBrace => blocks::block_statement(state)?,
            _ => {
                let expression = expressions::lowest_precedence(state)?;
                let end = utils::skip_semicolon(state)?;

                Statement::Expression { expression, end }
            }
        }
    };

    // A closing PHP tag is valid after the end of any top-level statement.
    if state.stream.current().kind == TokenKind::CloseTag {
        state.stream.next();
    }

    Ok(statement)
}
