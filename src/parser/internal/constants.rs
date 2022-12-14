use crate::lexer::token::TokenKind;
use crate::parser::ast::constant::ClassishConstant;
use crate::parser::ast::constant::Constant;
use crate::parser::ast::constant::ConstantEntry;
use crate::parser::ast::modifiers::ConstantModifierGroup;
use crate::parser::error::ParseResult;
use crate::parser::expressions;
use crate::parser::internal::identifiers;
use crate::parser::internal::utils;
use crate::parser::state::State;

pub fn parse(state: &mut State) -> ParseResult<Constant> {
    let start = utils::skip(state, TokenKind::Const)?;

    let mut entries = vec![];

    loop {
        let name = identifiers::constant_identifier(state)?;
        let span = utils::skip(state, TokenKind::Equals)?;
        let value = expressions::create(state)?;

        entries.push(ConstantEntry { name, span, value });

        if state.stream.current().kind == TokenKind::Comma {
            state.stream.next();
        } else {
            break;
        }
    }

    let end = utils::skip_semicolon(state)?;

    Ok(Constant {
        start,
        end,
        entries,
    })
}

pub fn classish(
    state: &mut State,
    modifiers: ConstantModifierGroup,
) -> ParseResult<ClassishConstant> {
    let attributes = state.get_attributes();

    let start = utils::skip(state, TokenKind::Const)?;

    let mut entries = vec![];

    loop {
        let name = identifiers::identifier_maybe_reserved(state)?;
        let span = utils::skip(state, TokenKind::Equals)?;
        let value = expressions::create(state)?;

        entries.push(ConstantEntry { name, span, value });

        if state.stream.current().kind == TokenKind::Comma {
            state.stream.next();
        } else {
            break;
        }
    }

    let end = utils::skip_semicolon(state)?;

    Ok(ClassishConstant {
        start,
        end,
        attributes,
        modifiers,
        entries,
    })
}
