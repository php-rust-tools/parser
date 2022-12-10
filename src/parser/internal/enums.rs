use crate::lexer::token::TokenKind;
use crate::parser::ast::enums::BackedEnum;
use crate::parser::ast::enums::BackedEnumCase;
use crate::parser::ast::enums::BackedEnumMember;
use crate::parser::ast::enums::BackedEnumType;
use crate::parser::ast::enums::UnitEnum;
use crate::parser::ast::enums::UnitEnumCase;
use crate::parser::ast::enums::UnitEnumMember;
use crate::parser::ast::Statement;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::expressions;
use crate::parser::internal::attributes;
use crate::parser::internal::constants;
use crate::parser::internal::functions;
use crate::parser::internal::identifiers;
use crate::parser::internal::modifiers;
use crate::parser::internal::utils;
use crate::parser::state::Scope;
use crate::parser::state::State;
use crate::scoped;

pub fn parse(state: &mut State) -> ParseResult<Statement> {
    let start = state.current.span;

    utils::skip(state, TokenKind::Enum)?;

    let name = identifiers::ident(state)?;

    let backed_type: Option<BackedEnumType> = if state.current.kind == TokenKind::Colon {
        utils::skip_colon(state)?;

        let identifier = identifiers::ident_of(state, &["string", "int"])?;
        Some(match &identifier.name[..] {
            b"string" => BackedEnumType::String(identifier.span),
            b"int" => BackedEnumType::Int(identifier.span),
            _ => unreachable!(),
        })
    } else {
        None
    };

    let mut implements = Vec::new();
    if state.current.kind == TokenKind::Implements {
        state.next();

        while state.current.kind != TokenKind::LeftBrace {
            implements.push(identifiers::full_name(state)?);

            if state.current.kind == TokenKind::Comma {
                state.next();
            } else {
                break;
            }
        }
    }

    let attributes = state.get_attributes();
    if let Some(backed_type) = backed_type {
        let (members, end) = scoped!(state, Scope::Enum(name.clone(), true), {
            utils::skip_left_brace(state)?;

            let mut members = Vec::new();
            while state.current.kind != TokenKind::RightBrace {
                state.skip_comments();
                members.push(backed_member(state, name.to_string())?);
            }

            (members, utils::skip_right_brace(state)?)
        });

        Ok(Statement::BackedEnum(BackedEnum {
            start,
            end,
            name,
            backed_type,
            attributes,
            implements,
            members,
        }))
    } else {
        let (members, end) = scoped!(state, Scope::Enum(name.clone(), false), {
            utils::skip_left_brace(state)?;

            let mut members = Vec::new();
            while state.current.kind != TokenKind::RightBrace {
                state.skip_comments();
                members.push(unit_member(state, name.to_string())?);
            }

            (members, utils::skip_right_brace(state)?)
        });

        Ok(Statement::UnitEnum(UnitEnum {
            start,
            end,
            name,
            attributes,
            implements,
            members,
        }))
    }
}

fn unit_member(state: &mut State, enum_name: String) -> ParseResult<UnitEnumMember> {
    let has_attributes = attributes::gather_attributes(state)?;

    if !has_attributes && state.current.kind == TokenKind::Case {
        let start = state.current.span;
        state.next();

        let name = identifiers::ident(state)?;

        if state.current.kind == TokenKind::Equals {
            return Err(ParseError::CaseValueForUnitEnum(
                name.to_string(),
                state.named(&enum_name),
                state.current.span,
            ));
        }

        let end = utils::skip_semicolon(state)?;

        return Ok(UnitEnumMember::Case(UnitEnumCase { start, end, name }));
    }

    let modifiers = modifiers::collect(state)?;

    if state.current.kind == TokenKind::Const {
        return constants::classish(state, modifiers::constant_group(modifiers)?)
            .map(UnitEnumMember::Constant);
    }

    functions::method(state, modifiers::enum_method_group(modifiers)?).map(UnitEnumMember::Method)
}

fn backed_member(state: &mut State, enum_name: String) -> ParseResult<BackedEnumMember> {
    let has_attributes = attributes::gather_attributes(state)?;

    if !has_attributes && state.current.kind == TokenKind::Case {
        let start = state.current.span;
        state.next();

        let name = identifiers::ident(state)?;

        if state.current.kind == TokenKind::SemiColon {
            return Err(ParseError::MissingCaseValueForBackedEnum(
                name.to_string(),
                state.named(&enum_name),
                state.current.span,
            ));
        }

        utils::skip(state, TokenKind::Equals)?;

        let value = expressions::lowest_precedence(state)?;

        let end = utils::skip_semicolon(state)?;

        return Ok(BackedEnumMember::Case(BackedEnumCase {
            start,
            end,
            name,
            value,
        }));
    }

    let modifiers = modifiers::collect(state)?;

    if state.current.kind == TokenKind::Const {
        return constants::classish(state, modifiers::constant_group(modifiers)?)
            .map(BackedEnumMember::Constant);
    }

    functions::method(state, modifiers::enum_method_group(modifiers)?).map(BackedEnumMember::Method)
}
