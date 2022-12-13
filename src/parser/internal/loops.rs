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

    let expr = expressions::create(state)?;

    utils::skip(state, TokenKind::As)?;

    let mut by_ref = state.stream.current().kind == TokenKind::Ampersand;
    if by_ref {
        state.stream.next();
    }

    let mut key_var = None;
    let mut value_var = expressions::create(state)?;

    if state.stream.current().kind == TokenKind::DoubleArrow {
        state.stream.next();

        key_var = Some(value_var.clone());

        by_ref = state.stream.current().kind == TokenKind::Ampersand;
        if by_ref {
            state.stream.next();
        }

        value_var = expressions::create(state)?;
    }

    utils::skip_right_parenthesis(state)?;

    let body = if state.stream.current().kind == TokenKind::Colon {
        utils::skip_colon(state)?;
        let then = blocks::body(state, &TokenKind::EndForeach)?;
        utils::skip(state, TokenKind::EndForeach)?;
        utils::skip_semicolon(state)?;
        then
    } else if state.stream.current().kind == TokenKind::LeftBrace {
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
        if state.stream.current().kind == TokenKind::SemiColon {
            break;
        }

        init.push(expressions::create(state)?);

        if state.stream.current().kind == TokenKind::Comma {
            state.stream.next();
        } else {
            break;
        }
    }

    utils::skip_semicolon(state)?;

    let mut condition = Vec::new();
    loop {
        if state.stream.current().kind == TokenKind::SemiColon {
            break;
        }

        condition.push(expressions::create(state)?);

        if state.stream.current().kind == TokenKind::Comma {
            state.stream.next();
        } else {
            break;
        }
    }
    utils::skip_semicolon(state)?;

    let mut r#loop = Vec::new();
    loop {
        if state.stream.current().kind == TokenKind::RightParen {
            break;
        }

        r#loop.push(expressions::create(state)?);

        if state.stream.current().kind == TokenKind::Comma {
            state.stream.next();
        } else {
            break;
        }
    }

    utils::skip_right_parenthesis(state)?;

    let then = if state.stream.current().kind == TokenKind::Colon {
        utils::skip_colon(state)?;
        let then = blocks::body(state, &TokenKind::EndFor)?;
        utils::skip(state, TokenKind::EndFor)?;
        utils::skip_semicolon(state)?;
        then
    } else if state.stream.current().kind == TokenKind::LeftBrace {
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

    let body = if state.stream.current().kind == TokenKind::LeftBrace {
        utils::skip_left_brace(state)?;
        let body = blocks::body(state, &TokenKind::RightBrace)?;
        utils::skip_right_brace(state)?;
        body
    } else {
        vec![parser::statement(state)?]
    };

    utils::skip(state, TokenKind::While)?;

    utils::skip_left_parenthesis(state)?;
    let condition = expressions::create(state)?;
    utils::skip_right_parenthesis(state)?;
    utils::skip_semicolon(state)?;

    Ok(Statement::DoWhile { condition, body })
}

pub fn while_loop(state: &mut State) -> ParseResult<Statement> {
    utils::skip(state, TokenKind::While)?;

    utils::skip_left_parenthesis(state)?;

    let condition = expressions::create(state)?;

    utils::skip_right_parenthesis(state)?;

    let body = if state.stream.current().kind == TokenKind::SemiColon {
        utils::skip_semicolon(state)?;
        vec![]
    } else if state.stream.current().kind == TokenKind::Colon {
        utils::skip_colon(state)?;
        let then = blocks::body(state, &TokenKind::EndWhile)?;
        utils::skip(state, TokenKind::EndWhile)?;
        utils::skip_semicolon(state)?;
        then
    } else if state.stream.current().kind == TokenKind::LeftBrace {
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
    if state.stream.current().kind != TokenKind::SemiColon {
        num = Some(expressions::create(state)?);
    }

    utils::skip_semicolon(state)?;

    Ok(Statement::Continue { num })
}

pub fn break_statement(state: &mut State) -> ParseResult<Statement> {
    utils::skip(state, TokenKind::Break)?;

    let mut num = None;
    if state.stream.current().kind != TokenKind::SemiColon {
        num = Some(expressions::create(state)?);
    }

    utils::skip_semicolon(state)?;

    Ok(Statement::Break { num })
}
