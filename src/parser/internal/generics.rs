use crate::lexer::token::TokenKind;
use crate::parser::ast::generics::Generic;
use crate::parser::ast::generics::GenericGroup;
use crate::parser::error::ParseResult;
use crate::parser::internal::data_type;
use crate::parser::internal::utils;
use crate::parser::state::State;

pub fn parse(state: &mut State) -> ParseResult<GenericGroup> {
    let start = state.current.span;
    utils::skip(state, TokenKind::Generic)?;
    let mut members = vec![];
    while state.current.kind != TokenKind::GreaterThan {
        let r#type = data_type::data_type(state)?;
        members.push(Generic { r#type });

        if state.current.kind == TokenKind::Comma {
            state.next();
        } else {
            break;
        }
    }

    let end = state.current.span;
    utils::skip(state, TokenKind::GreaterThan)?;

    Ok(GenericGroup {
        start,
        end,
        members,
    })
}
