use crate::lexer::token::TokenKind;
use crate::parser;
use crate::parser::ast::Statement;
use crate::parser::error::ParseResult;
use crate::parser::expressions;
use crate::parser::internal::blocks;
use crate::parser::internal::utils;
use crate::parser::state::State;

pub fn foreach_loop(state: &mut State) -> ParseResult<Statement> {
    utils::skip(state, TokenKind::Foreach)?;

    utils::skip_left_parenthesis(state)?;

    let expr = expressions::lowest_precedence(state)?;

    utils::skip(state, TokenKind::As)?;

    let mut by_ref = state.current.kind == TokenKind::Ampersand;
    if by_ref {
        state.next();
    }

    let mut key_var = None;
    let mut value_var = expressions::lowest_precedence(state)?;

    if state.current.kind == TokenKind::DoubleArrow {
        state.next();

        key_var = Some(value_var.clone());

        by_ref = state.current.kind == TokenKind::Ampersand;
        if by_ref {
            state.next();
        }

        value_var = expressions::lowest_precedence(state)?;
    }

    utils::skip_right_parenthesis(state)?;

    let body = if state.current.kind == TokenKind::Colon {
        utils::skip_colon(state)?;
        let then = blocks::body(state, &TokenKind::EndForeach)?;
        utils::skip(state, TokenKind::EndForeach)?;
        utils::skip_semicolon(state)?;
        then
    } else if state.current.kind == TokenKind::LeftBrace {
        utils::skip_left_brace(state)?;
        let then = blocks::body(state, &TokenKind::RightBrace)?;
        utils::skip_right_brace(state)?;
        then
    } else {
        vec![parser::statement(state)?]
    };

    Ok(Statement::Foreach {
        expr,
        by_ref,
        key_var,
        value_var,
        body,
    })
}

pub fn for_loop(state: &mut State) -> ParseResult<Statement> {
    utils::skip(state, TokenKind::For)?;

    utils::skip_left_parenthesis(state)?;

    let mut init = Vec::new();
    loop {
        if state.current.kind == TokenKind::SemiColon {
            break;
        }

        init.push(expressions::lowest_precedence(state)?);

        if state.current.kind == TokenKind::Comma {
            state.next();
        } else {
            break;
        }
    }

    utils::skip_semicolon(state)?;

    let mut condition = Vec::new();
    loop {
        if state.current.kind == TokenKind::SemiColon {
            break;
        }

        condition.push(expressions::lowest_precedence(state)?);

        if state.current.kind == TokenKind::Comma {
            state.next();
        } else {
            break;
        }
    }
    utils::skip_semicolon(state)?;

    let mut r#loop = Vec::new();
    loop {
        if state.current.kind == TokenKind::RightParen {
            break;
        }

        r#loop.push(expressions::lowest_precedence(state)?);

        if state.current.kind == TokenKind::Comma {
            state.next();
        } else {
            break;
        }
    }

    utils::skip_right_parenthesis(state)?;

    let then = if state.current.kind == TokenKind::Colon {
        utils::skip_colon(state)?;
        let then = blocks::body(state, &TokenKind::EndFor)?;
        utils::skip(state, TokenKind::EndFor)?;
        utils::skip_semicolon(state)?;
        then
    } else if state.current.kind == TokenKind::LeftBrace {
        utils::skip_left_brace(state)?;
        let then = blocks::body(state, &TokenKind::RightBrace)?;
        utils::skip_right_brace(state)?;
        then
    } else {
        vec![parser::statement(state)?]
    };

    Ok(Statement::For {
        init,
        condition,
        r#loop,
        then,
    })
}

pub fn do_loop(state: &mut State) -> ParseResult<Statement> {
    utils::skip(state, TokenKind::Do)?;

    let body = if state.current.kind == TokenKind::LeftBrace {
        utils::skip_left_brace(state)?;
        let body = blocks::body(state, &TokenKind::RightBrace)?;
        utils::skip_right_brace(state)?;
        body
    } else {
        vec![parser::statement(state)?]
    };

    utils::skip(state, TokenKind::While)?;

    utils::skip_left_parenthesis(state)?;
    let condition = expressions::lowest_precedence(state)?;
    utils::skip_right_parenthesis(state)?;
    utils::skip_semicolon(state)?;

    Ok(Statement::DoWhile { condition, body })
}

pub fn while_loop(state: &mut State) -> ParseResult<Statement> {
    utils::skip(state, TokenKind::While)?;

    utils::skip_left_parenthesis(state)?;

    let condition = expressions::lowest_precedence(state)?;

    utils::skip_right_parenthesis(state)?;

    let body = if state.current.kind == TokenKind::SemiColon {
        utils::skip_semicolon(state)?;
        vec![]
    } else if state.current.kind == TokenKind::Colon {
        utils::skip_colon(state)?;
        let then = blocks::body(state, &TokenKind::EndWhile)?;
        utils::skip(state, TokenKind::EndWhile)?;
        utils::skip_semicolon(state)?;
        then
    } else if state.current.kind == TokenKind::LeftBrace {
        utils::skip_left_brace(state)?;
        let then = blocks::body(state, &TokenKind::RightBrace)?;
        utils::skip_right_brace(state)?;
        then
    } else {
        vec![parser::statement(state)?]
    };

    Ok(Statement::While { condition, body })
}

pub fn continue_statement(state: &mut State) -> ParseResult<Statement> {
    utils::skip(state, TokenKind::Continue)?;

    let mut num = None;
    if state.current.kind != TokenKind::SemiColon {
        num = Some(expressions::lowest_precedence(state)?);
    }

    utils::skip_semicolon(state)?;

    Ok(Statement::Continue { num })
}

pub fn break_statement(state: &mut State) -> ParseResult<Statement> {
    utils::skip(state, TokenKind::Break)?;

    let mut num = None;
    if state.current.kind != TokenKind::SemiColon {
        num = Some(expressions::lowest_precedence(state)?);
    }

    utils::skip_semicolon(state)?;

    Ok(Statement::Break { num })
}
