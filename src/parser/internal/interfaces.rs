use crate::lexer::token::Span;
use crate::lexer::token::TokenKind;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::interfaces::Interface;
use crate::parser::ast::interfaces::InterfaceBody;
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
    let span = utils::skip(state, TokenKind::Interface)?;

    let name = identifiers::type_identifier(state)?;

    let extends = if state.stream.current().kind == TokenKind::Extends {
        let span = state.stream.current().span;

        state.stream.next();

        let parents = utils::at_least_one_comma_separated::<SimpleIdentifier>(state, &|state| {
            identifiers::full_type_name(state)
        })?;

        Some(InterfaceExtends { span, parents })
    } else {
        None
    };

    let attributes = state.get_attributes();

    let body = scoped!(state, Scope::Interface(name.clone()), {
        let start = utils::skip_left_brace(state)?;

        let mut members = Vec::new();
        while state.stream.current().kind != TokenKind::RightBrace {
            members.push(member(state)?);
        }

        let end = utils::skip_right_brace(state)?;

        InterfaceBody {
            start,
            members,
            end,
        }
    });

    Ok(Statement::Interface(Interface {
        span,
        name,
        attributes,
        extends,
        body,
    }))
}

fn member(state: &mut State) -> ParseResult<InterfaceMember> {
    attributes::gather_attributes(state)?;

    let modifiers = modifiers::collect(state)?;

    if state.stream.current().kind == TokenKind::Const {
        constants::classish(state, constant_modifiers(modifiers)?).map(InterfaceMember::Constant)
    } else {
        method(state, method_modifiers(modifiers)?).map(InterfaceMember::Method)
    }
}

#[inline(always)]
fn method_modifiers(input: Vec<(Span, TokenKind)>) -> ParseResult<MethodModifierGroup> {
    let modifiers = input
        .iter()
        .map(|(span, token)| match token {
            TokenKind::Public => Ok(MethodModifier::Public(*span)),
            TokenKind::Static => Ok(MethodModifier::Static(*span)),
            _ => Err(ParseError::CannotUseModifierOnInterfaceMethod(
                token.to_string(),
                *span,
            )),
        })
        .collect::<ParseResult<Vec<MethodModifier>>>()?;

    Ok(MethodModifierGroup { modifiers })
}

#[inline(always)]
fn constant_modifiers(input: Vec<(Span, TokenKind)>) -> ParseResult<ConstantModifierGroup> {
    let modifiers = input
        .iter()
        .map(|(span, token)| match token {
            TokenKind::Public => Ok(ConstantModifier::Public(*span)),
            TokenKind::Final => Ok(ConstantModifier::Final(*span)),
            _ => Err(ParseError::CannotUseModifierOnInterfaceConstant(
                token.to_string(),
                *span,
            )),
        })
        .collect::<ParseResult<Vec<ConstantModifier>>>()?;

    Ok(ConstantModifierGroup { modifiers })
}
