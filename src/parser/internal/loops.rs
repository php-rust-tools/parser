use crate::lexer::token::TokenKind;
use crate::parser;
use crate::parser::ast::loops::DoWhileBody;
use crate::parser::ast::loops::DoWhileLoop;
use crate::parser::ast::loops::ForBody;
use crate::parser::ast::loops::ForIterator;
use crate::parser::ast::loops::ForLoop;
use crate::parser::ast::loops::ForeachBody;
use crate::parser::ast::loops::ForeachIterator;
use crate::parser::ast::loops::ForeachLoop;
use crate::parser::ast::loops::WhileBody;
use crate::parser::ast::loops::WhileLoop;
use crate::parser::ast::Statement;
use crate::parser::error::ParseResult;
use crate::parser::expressions;
use crate::parser::internal::blocks;
use crate::parser::internal::utils;
use crate::parser::state::State;

pub fn foreach_loop(state: &mut State) -> ParseResult<Statement> {
    let foreach = utils::skip(state, TokenKind::Foreach)?;

    let iterator = utils::parenthesized(state, &|state: &mut State| {
        let expression = expressions::create(state)?;

        let r#as = utils::skip(state, TokenKind::As)?;

        let current = state.stream.current();
        let ampersand = if current.kind == TokenKind::Ampersand {
            state.stream.next();
            Some(current.span)
        } else {
            None
        };

        let mut value = expressions::create(state)?;

        let current = state.stream.current();
        if current.kind == TokenKind::DoubleArrow {
            state.stream.next();
            let arrow = current.span;

            let current = state.stream.current();
            let ampersand = if current.kind == TokenKind::Ampersand {
                state.stream.next();
                Some(current.span)
            } else {
                None
            };

            let mut key = expressions::create(state)?;

            std::mem::swap(&mut value, &mut key);

            Ok(ForeachIterator::KeyAndValue {
                expression,
                r#as,
                key,
                arrow,
                ampersand,
                value,
            })
        } else {
            Ok(ForeachIterator::Value {
                expression,
                r#as,
                ampersand,
                value,
            })
        }
    })?;

    let body = if state.stream.current().kind == TokenKind::Colon {
        ForeachBody::Block {
            colon: utils::skip_colon(state)?,
            statements: blocks::multiple_statements(state, &TokenKind::EndForeach)?,
            endforeach: utils::skip(state, TokenKind::EndForeach)?,
            semicolon: utils::skip_semicolon(state)?,
        }
    } else if state.stream.current().kind == TokenKind::LeftBrace {
        ForeachBody::Braced(utils::braced(state, &|state| {
            blocks::multiple_statements(state, &TokenKind::RightBrace)
        })?)
    } else {
        ForeachBody::Statement(parser::statement(state).map(Box::new)?)
    };

    Ok(Statement::Foreach(ForeachLoop {
        foreach,
        iterator,
        body,
    }))
}

pub fn for_loop(state: &mut State) -> ParseResult<Statement> {
    let r#for = utils::skip(state, TokenKind::For)?;

    let iterator = utils::parenthesized(state, &|state| {
        Ok(ForIterator {
            initializations: utils::semicolon_terminated(state, &|state| {
                utils::comma_separated_no_trailing(
                    state,
                    &expressions::create,
                    TokenKind::SemiColon,
                )
            })?,
            conditions: utils::semicolon_terminated(state, &|state| {
                utils::comma_separated_no_trailing(
                    state,
                    &expressions::create,
                    TokenKind::SemiColon,
                )
            })?,
            r#loop: utils::comma_separated_no_trailing(
                state,
                &expressions::create,
                TokenKind::RightParen,
            )?,
        })
    })?;

    let body = if state.stream.current().kind == TokenKind::Colon {
        ForBody::Block {
            colon: utils::skip_colon(state)?,
            statements: blocks::multiple_statements(state, &TokenKind::EndFor)?,
            endfor: utils::skip(state, TokenKind::EndFor)?,
            semicolon: utils::skip_semicolon(state)?,
        }
    } else if state.stream.current().kind == TokenKind::LeftBrace {
        ForBody::Braced(utils::braced(state, &|state| {
            blocks::multiple_statements(state, &TokenKind::RightBrace)
        })?)
    } else {
        ForBody::Statement(parser::statement(state).map(Box::new)?)
    };

    Ok(Statement::For(ForLoop {
        r#for,
        iterator,
        body,
    }))
}

pub fn do_loop(state: &mut State) -> ParseResult<Statement> {
    let r#do = utils::skip(state, TokenKind::Do)?;

    let body = if state.stream.current().kind == TokenKind::LeftBrace {
        DoWhileBody::Braced(utils::braced(state, &|state| {
            blocks::multiple_statements(state, &TokenKind::RightBrace)
        })?)
    } else {
        DoWhileBody::Statement(parser::statement(state).map(Box::new)?)
    };

    let r#while = utils::skip(state, TokenKind::While)?;

    let condition = utils::semicolon_terminated(state, &|state| {
        utils::parenthesized(state, &expressions::create)
    })?;

    Ok(Statement::DoWhile(DoWhileLoop {
        r#do,
        body,
        r#while,
        condition,
    }))
}

pub fn while_loop(state: &mut State) -> ParseResult<Statement> {
    let r#while = utils::skip(state, TokenKind::While)?;

    let condition = utils::parenthesized(state, &expressions::create)?;

    let body = if state.stream.current().kind == TokenKind::Colon {
        WhileBody::Block {
            colon: utils::skip_colon(state)?,
            statements: blocks::multiple_statements(state, &TokenKind::EndWhile)?,
            endwhile: utils::skip(state, TokenKind::EndWhile)?,
            semicolon: utils::skip_semicolon(state)?,
        }
    } else if state.stream.current().kind == TokenKind::LeftBrace {
        WhileBody::Braced(utils::braced(state, &|state| {
            blocks::multiple_statements(state, &TokenKind::RightBrace)
        })?)
    } else {
        WhileBody::Statement(parser::statement(state).map(Box::new)?)
    };

    Ok(Statement::While(WhileLoop {
        r#while,
        condition,
        body,
    }))
}

pub fn continue_statement(state: &mut State) -> ParseResult<Statement> {
    let r#continue = utils::skip(state, TokenKind::Continue)?;

    let mut expression = None;
    if state.stream.current().kind != TokenKind::SemiColon {
        expression = Some(expressions::create(state)?);
    }

    let semicolon = utils::skip_semicolon(state)?;

    Ok(Statement::Continue {
        r#continue,
        expression,
        semicolon,
    })
}

pub fn break_statement(state: &mut State) -> ParseResult<Statement> {
    let r#break = utils::skip(state, TokenKind::Break)?;

    let mut expression = None;
    if state.stream.current().kind != TokenKind::SemiColon {
        expression = Some(expressions::create(state)?);
    }

    let semicolon = utils::skip_semicolon(state)?;

    Ok(Statement::Break {
        r#break,
        expression,
        semicolon,
    })
}
