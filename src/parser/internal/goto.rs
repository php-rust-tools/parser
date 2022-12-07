use crate::lexer::token::TokenKind;
use crate::parser::ast::Statement;
use crate::parser::error::ParseResult;
use crate::parser::internal::identifiers;
use crate::parser::internal::utils;
use crate::parser::state::State;

pub fn label_statement(state: &mut State) -> ParseResult<Statement> {
    let label = identifiers::ident(state)?;

    utils::skip_colon(state)?;

    Ok(Statement::Label { label })
}

pub fn goto_statement(state: &mut State) -> ParseResult<Statement> {
    utils::skip(state, TokenKind::Goto)?;

    let label = identifiers::ident(state)?;

    utils::skip_semicolon(state)?;

    Ok(Statement::Goto { label })
}
