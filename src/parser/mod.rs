use crate::expect_literal;
use crate::lexer::token::OpenTagKind;
use crate::lexer::token::Token;
use crate::lexer::token::TokenKind;
use crate::parser::ast::declares::Declare;
use crate::parser::ast::declares::DeclareBody;
use crate::parser::ast::declares::DeclareEntry;
use crate::parser::ast::declares::DeclareEntryGroup;
use crate::parser::ast::variables::Variable;
use crate::parser::ast::{Program, Statement, StaticVar};
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
use crate::parser::state::stream::TokenStream;
use crate::parser::state::State;

pub mod ast;
pub mod error;

mod expressions;
mod internal;
mod macros;
mod state;

pub fn parse(tokens: &[Token]) -> ParseResult<Program> {
    let mut stream = TokenStream::new(tokens);
    let mut state = State::new(&mut stream);

    let mut ast = Program::new();

    while !state.stream.is_eof() {
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

            let content = if let TokenKind::InlineHtml = state.stream.current().kind.clone() {
                let content = state.stream.current().value.clone();
                state.stream.next();
                Some(content)
            } else {
                None
            };

            Statement::HaltCompiler { content }
        }
        _ => statement(state)?,
    };

    Ok(statement)
}

fn statement(state: &mut State) -> ParseResult<Statement> {
    let has_attributes = attributes::gather_attributes(state)?;

    let current = state.stream.current();
    let peek = state.stream.peek();
    let statement = if has_attributes {
        match &current.kind {
            TokenKind::Abstract => classes::parse(state)?,
            TokenKind::Readonly if peek.kind != TokenKind::LeftParen => classes::parse(state)?,
            TokenKind::Final => classes::parse(state)?,
            TokenKind::Class => classes::parse(state)?,
            TokenKind::Interface => interfaces::parse(state)?,
            TokenKind::Trait => traits::parse(state)?,
            TokenKind::Enum
                if !matches!(
                    peek.kind,
                    TokenKind::LeftParen | TokenKind::DoubleColon | TokenKind::Colon,
                ) =>
            {
                enums::parse(state)?
            }
            TokenKind::Function
                if identifiers::is_identifier_maybe_soft_reserved(&peek.kind)
                    || peek.kind == TokenKind::Ampersand =>
            {
                if peek.kind == TokenKind::Ampersand {
                    if !identifiers::is_identifier_maybe_soft_reserved(
                        &state.stream.lookahead(1).kind,
                    ) {
                        return Ok(Statement::Expression {
                            expression: expressions::attributes(state)?,
                            ending: utils::skip_ending(state)?,
                        });
                    }

                    functions::function(state)?
                } else {
                    functions::function(state)?
                }
            }
            _ => Statement::Expression {
                expression: expressions::attributes(state)?,
                ending: utils::skip_ending(state)?,
            },
        }
    } else {
        match &current.kind {
            TokenKind::OpenTag(OpenTagKind::Echo) => {
                let span = current.span;
                state.stream.next();

                Statement::EchoOpeningTag(span)
            }
            TokenKind::OpenTag(OpenTagKind::Full) => {
                let span = current.span;
                state.stream.next();

                Statement::FullOpeningTag(span)
            }
            TokenKind::OpenTag(OpenTagKind::Short) => {
                let span = current.span;
                state.stream.next();

                Statement::ShortOpeningTag(span)
            }
            TokenKind::CloseTag => {
                let span = current.span;
                state.stream.next();

                Statement::ClosingTag(span)
            }
            TokenKind::Abstract => classes::parse(state)?,
            TokenKind::Readonly if peek.kind != TokenKind::LeftParen => classes::parse(state)?,
            TokenKind::Final => classes::parse(state)?,
            TokenKind::Class => classes::parse(state)?,
            TokenKind::Interface => interfaces::parse(state)?,
            TokenKind::Trait => traits::parse(state)?,
            TokenKind::Enum
                if !matches!(
                    peek.kind,
                    TokenKind::LeftParen | TokenKind::DoubleColon | TokenKind::Colon,
                ) =>
            {
                enums::parse(state)?
            }
            TokenKind::Function
                if identifiers::is_identifier_maybe_soft_reserved(&peek.kind)
                    || peek.kind == TokenKind::Ampersand =>
            {
                if peek.kind == TokenKind::Ampersand {
                    if !identifiers::is_identifier_maybe_soft_reserved(
                        &state.stream.lookahead(1).kind,
                    ) {
                        return Ok(Statement::Expression {
                            expression: expressions::attributes(state)?,
                            ending: utils::skip_ending(state)?,
                        });
                    }

                    functions::function(state)?
                } else {
                    functions::function(state)?
                }
            }
            TokenKind::Goto => goto::goto_statement(state)?,
            token
                if identifiers::is_identifier_maybe_reserved(token)
                    && peek.kind == TokenKind::Colon =>
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
                        let statements =
                            blocks::multiple_statements_until(state, &TokenKind::RightBrace)?;
                        let end = utils::skip_right_brace(state)?;

                        DeclareBody::Braced {
                            start,
                            statements,
                            end,
                        }
                    }
                    TokenKind::Colon => {
                        let start = utils::skip_colon(state)?;
                        let statements =
                            blocks::multiple_statements_until(state, &TokenKind::EndDeclare)?;
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
                        let expression = expressions::create(state)?;
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
                let span = current.span;
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
            TokenKind::Static if matches!(peek.kind, TokenKind::Variable) => {
                state.stream.next();

                let mut vars = vec![];

                // `loop` instead of `while` as we don't allow for extra commas.
                loop {
                    let var = variables::simple_variable(state)?;
                    let mut default = None;

                    if state.stream.current().kind == TokenKind::Equals {
                        state.stream.next();

                        default = Some(expressions::create(state)?);
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
            TokenKind::InlineHtml => {
                let html = state.stream.current().value.clone();
                state.stream.next();

                Statement::InlineHtml(html)
            }
            TokenKind::Do => loops::do_while_statement(state)?,
            TokenKind::While => loops::while_statement(state)?,
            TokenKind::For => loops::for_statement(state)?,
            TokenKind::Foreach => loops::foreach_statement(state)?,
            TokenKind::Continue => loops::continue_statement(state)?,
            TokenKind::Break => loops::break_statement(state)?,
            TokenKind::Switch => control_flow::switch_statement(state)?,
            TokenKind::If => control_flow::if_statement(state)?,
            TokenKind::Try => try_block::try_block(state)?,
            TokenKind::LeftBrace => blocks::block_statement(state)?,
            TokenKind::SemiColon => {
                let start = current.span;

                state.stream.next();

                Statement::Noop(start)
            }
            TokenKind::Echo => {
                state.stream.next();

                let mut values = Vec::new();
                loop {
                    values.push(expressions::create(state)?);

                    if state.stream.current().kind == TokenKind::Comma {
                        state.stream.next();
                    } else {
                        break;
                    }
                }

                Statement::Echo {
                    echo: current.span,
                    values,
                    ending: utils::skip_ending(state)?,
                }
            }
            TokenKind::Return => {
                state.stream.next();

                let value = if matches!(
                    state.stream.current().kind,
                    TokenKind::SemiColon | TokenKind::CloseTag
                ) {
                    None
                } else {
                    expressions::create(state).map(Some)?
                };

                Statement::Return {
                    r#return: current.span,
                    value,
                    ending: utils::skip_ending(state)?,
                }
            }
            _ => Statement::Expression {
                expression: expressions::create(state)?,
                ending: utils::skip_ending(state)?,
            },
        }
    };

    Ok(statement)
}
