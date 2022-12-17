use crate::lexer::token::OpenTagKind;
use crate::lexer::token::TokenKind;
use crate::parser;
use crate::parser::ast::Block;
use crate::parser::ast::Statement;
use crate::parser::error::ParseResult;
use crate::parser::internal::utils;
use crate::parser::state::State;

pub fn block_statement(state: &mut State) -> ParseResult<Statement> {
    Ok(Statement::Block(utils::braced(
        state,
        &|state: &mut State| multiple_statements(state, &TokenKind::RightBrace),
    )?))
}

pub fn multiple_statements(state: &mut State, until: &TokenKind) -> ParseResult<Block> {
    let mut block = Block::new();

    while !state.stream.is_eof() && &state.stream.current().kind != until {
        if let TokenKind::OpenTag(OpenTagKind::Full) = state.stream.current().kind {
            state.stream.next();
            continue;
        }

        block.push(parser::statement(state)?);
    }

    Ok(block)
}
