use crate::lexer::token::Span;
use crate::lexer::token::TokenKind;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::interfaces::Interface;
use crate::parser::ast::interfaces::InterfaceExtends;
use crate::parser::ast::interfaces::InterfaceMember;
use crate::parser::ast::modifiers::ConstantModifier;
use crate::parser::ast::modifiers::ConstantModifierGroup;
use crate::parser::ast::modifiers::MethodModifier;
use crate::parser::ast::modifiers::MethodModifierGroup;
use crate::parser::ast::Statement;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::internal::attributes;
use crate::parser::internal::constants;
use crate::parser::internal::functions::method;
use crate::parser::internal::identifiers;
use crate::parser::internal::modifiers;
use crate::parser::internal::utils;
use crate::parser::state::Scope;
use crate::parser::state::State;
use crate::scoped;

pub fn parse(state: &mut State) -> ParseResult<Statement> {
    let start = utils::skip(state, TokenKind::Interface)?;

    let name = identifiers::type_identifier(state)?;

    let extends = if state.current.kind == TokenKind::Extends {
        let span = state.current.span;

        state.next();

        let parents = utils::at_least_one_comma_separated::<SimpleIdentifier>(state, &|state| {
            identifiers::full_name(state)
        })?;

        Some(InterfaceExtends { span, parents })
    } else {
        None
    };

    let attributes = state.get_attributes();

    let (members, end) = scoped!(state, Scope::Interface(name.clone()), {
        utils::skip_left_brace(state)?;

        let mut members = Vec::new();
        while state.current.kind != TokenKind::RightBrace {
            state.skip_comments();
            members.push(member(state)?);
        }

        (members, utils::skip_right_brace(state)?)
    });

    Ok(Statement::Interface(Interface {
        start,
        end,
        name,
        attributes,
        extends,
        members,
    }))
}

fn member(state: &mut State) -> ParseResult<InterfaceMember> {
    attributes::gather_attributes(state)?;

    let modifiers = modifiers::collect(state)?;

    if state.current.kind == TokenKind::Const {
        constants::classish(state, constant_modifiers(modifiers)?).map(InterfaceMember::Constant)
    } else {
        method(state, method_modifiers(modifiers)?).map(InterfaceMember::Method)
    }
}

#[inline(always)]
fn method_modifiers(input: Vec<(Span, TokenKind, Span)>) -> ParseResult<MethodModifierGroup> {
    let modifiers = input
        .iter()
        .map(|(start, token, end)| match token {
            TokenKind::Public => Ok(MethodModifier::Public {
                start: *start,
                end: *end,
            }),
            TokenKind::Static => Ok(MethodModifier::Static {
                start: *start,
                end: *end,
            }),
            _ => Err(ParseError::CannotUseModifierOnInterfaceMethod(
                token.to_string(),
                *start,
            )),
        })
        .collect::<ParseResult<Vec<MethodModifier>>>()?;

    Ok(MethodModifierGroup { modifiers })
}

#[inline(always)]
fn constant_modifiers(input: Vec<(Span, TokenKind, Span)>) -> ParseResult<ConstantModifierGroup> {
    let modifiers = input
        .iter()
        .map(|(start, token, end)| match token {
            TokenKind::Public => Ok(ConstantModifier::Public {
                start: *start,
                end: *end,
            }),
            TokenKind::Final => Ok(ConstantModifier::Final {
                start: *start,
                end: *end,
            }),
            _ => Err(ParseError::CannotUseModifierOnInterfaceConstant(
                token.to_string(),
                *start,
            )),
        })
        .collect::<ParseResult<Vec<ConstantModifier>>>()?;

    Ok(ConstantModifierGroup { modifiers })
}
