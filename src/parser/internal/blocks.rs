use crate::lexer::token::TokenKind;
use crate::parser;
use crate::parser::ast::Block;
use crate::parser::ast::Statement;
use crate::parser::error::ParseResult;
use crate::parser::internal::utils;
use crate::parser::state::State;

pub fn block_statement(state: &mut State) -> ParseResult<Statement> {
    utils::skip_left_brace(state)?;

    let body = body(state, &TokenKind::RightBrace)?;

    utils::skip_right_brace(state)?;

    Ok(Statement::Block { body })
}

pub fn body(state: &mut State, until: &TokenKind) -> ParseResult<Block> {
    let mut block = Block::new();

    while !state.stream.is_eof() && &state.stream.current().kind != until {
        if let TokenKind::OpenTag(_) = state.stream.current().kind {
            state.stream.next();
            continue;
        }

        block.push(parser::statement(state)?);
    }

    Ok(block)
}
