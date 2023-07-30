use crate::lexer::token::TokenKind;
use crate::parser::ast::constant::ClassishConstant;
use crate::parser::ast::constant::ConstantEntry;
use crate::parser::ast::constant::ConstantStatement;
use crate::parser::ast::data_type::Type;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::modifiers::ConstantModifierGroup;
use crate::parser::ast::variables::SimpleVariable;
use crate::parser::error;
use crate::parser::error::ParseResult;
use crate::parser::expressions;
use crate::parser::internal::data_type;
use crate::parser::internal::identifiers;
use crate::parser::internal::utils;
use crate::parser::state::State;

pub fn parse(state: &mut State) -> ParseResult<ConstantStatement> {
    let comments = state.stream.comments();
    let start = utils::skip(state, TokenKind::Const)?;

    let mut entries = vec![];

    loop {
        let name = identifiers::constant_identifier(state)?;
        let span = utils::skip(state, TokenKind::Equals)?;
        let value = expressions::create(state)?;

        entries.push(ConstantEntry {
            name,
            equals: span,
            value,
        });

        if state.stream.current().kind == TokenKind::Comma {
            state.stream.next();
        } else {
            break;
        }
    }

    let end = utils::skip_semicolon(state)?;

    Ok(ConstantStatement {
        comments,
        r#const: start,
        entries,
        semicolon: end,
    })
}

pub fn classish(
    state: &mut State,
    class_name: Option<&SimpleIdentifier>,
    modifiers: ConstantModifierGroup,
) -> ParseResult<ClassishConstant> {
    let attributes = state.get_attributes();

    let comments = state.stream.comments();
    let start = utils::skip(state, TokenKind::Const)?;
    let mut ty: Result<Option<Type>, ()> = Err(());

    let mut entries = vec![];
    let mut type_checked = false;

    loop {
        let name = if state.stream.peek().kind == TokenKind::Equals {
            identifiers::identifier_maybe_reserved(state)?
        } else {
            if ty.is_err() {
                ty = Ok(data_type::optional_data_type(state)?);
            }

            identifiers::identifier_maybe_reserved(state)?
        };

        let span = utils::skip(state, TokenKind::Equals)?;
        let value = expressions::create(state)?;

        if !type_checked {
            type_checked = true;

            match &ty.clone().unwrap_or(None) {
                Some(ty) => {
                    if ty.includes_callable() || ty.is_bottom() {
                        let error = error::forbidden_type_used_in_property_or_constant(
                            state,
                            class_name,
                            &SimpleVariable {
                                span,
                                name: name.value.clone(),
                            },
                            ty.clone(),
                        );

                        state.record(error);
                    }
                }
                None => (),
            }
        }
        entries.push(ConstantEntry {
            name,
            equals: span,
            value,
        });

        if state.stream.current().kind == TokenKind::Comma {
            state.stream.next();
        } else {
            break;
        }
    }

    let end = utils::skip_semicolon(state)?;

    Ok(ClassishConstant {
        comments,
        attributes,
        modifiers,
        r#const: start,
        r#type: ty.unwrap_or(None),
        entries,
        semicolon: end,
    })
}
