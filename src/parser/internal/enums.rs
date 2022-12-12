use crate::lexer::token::Span;
use crate::lexer::token::TokenKind;
use crate::parser::ast::enums::BackedEnum;
use crate::parser::ast::enums::BackedEnumBody;
use crate::parser::ast::enums::BackedEnumCase;
use crate::parser::ast::enums::BackedEnumMember;
use crate::parser::ast::enums::BackedEnumType;
use crate::parser::ast::enums::UnitEnum;
use crate::parser::ast::enums::UnitEnumBody;
use crate::parser::ast::enums::UnitEnumCase;
use crate::parser::ast::enums::UnitEnumMember;
use crate::parser::ast::functions::Method;
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
    let span = utils::skip(state, TokenKind::Enum)?;

    let name = identifiers::type_identifier(state)?;

    let backed_type: Option<BackedEnumType> = if state.stream.current().kind == TokenKind::Colon {
        utils::skip_colon(state)?;

        let identifier = identifiers::identifier_of(state, &["string", "int"])?;
        Some(match &identifier.value[..] {
            b"string" => BackedEnumType::String(identifier.span),
            b"int" => BackedEnumType::Int(identifier.span),
            _ => unreachable!(),
        })
    } else {
        None
    };

    let mut implements = Vec::new();
    if state.stream.current().kind == TokenKind::Implements {
        state.stream.next();

        while state.stream.current().kind != TokenKind::LeftBrace {
            implements.push(identifiers::full_type_name(state)?);

            if state.stream.current().kind == TokenKind::Comma {
                state.stream.next();
            } else {
                break;
            }
        }
    }

    let attributes = state.get_attributes();
    if let Some(backed_type) = backed_type {
        let body = scoped!(state, Scope::Enum(name.clone(), true), {
            let start = utils::skip_left_brace(state)?;

            let mut members = Vec::new();
            while state.stream.current().kind != TokenKind::RightBrace {
                members.push(backed_member(state, name.to_string())?);
            }

            let end = utils::skip_right_brace(state)?;

            BackedEnumBody {
                start,
                members,
                end,
            }
        });

        Ok(Statement::BackedEnum(BackedEnum {
            span,
            name,
            backed_type,
            attributes,
            implements,
            body,
        }))
    } else {
        let body = scoped!(state, Scope::Enum(name.clone(), false), {
            let start = utils::skip_left_brace(state)?;

            let mut members = Vec::new();
            while state.stream.current().kind != TokenKind::RightBrace {
                members.push(unit_member(state, name.to_string())?);
            }

            let end = utils::skip_right_brace(state)?;

            UnitEnumBody {
                start,
                members,
                end,
            }
        });

        Ok(Statement::UnitEnum(UnitEnum {
            span,
            name,
            attributes,
            implements,
            body,
        }))
    }
}

fn unit_member(state: &mut State, enum_name: String) -> ParseResult<UnitEnumMember> {
    attributes::gather_attributes(state)?;

    if state.stream.current().kind == TokenKind::Case {
        let attributes = state.get_attributes();

        let start = state.stream.current().span;
        state.stream.next();

        let name = identifiers::identifier_maybe_reserved(state)?;

        if state.stream.current().kind == TokenKind::Equals {
            return Err(ParseError::CaseValueForUnitEnum(
                name.to_string(),
                state.named(&enum_name),
                state.stream.current().span,
            ));
        }

        let end = utils::skip_semicolon(state)?;

        return Ok(UnitEnumMember::Case(UnitEnumCase {
            start,
            end,
            name,
            attributes,
        }));
    }

    let modifiers = modifiers::collect(state)?;

    if state.stream.current().kind == TokenKind::Const {
        return constants::classish(state, modifiers::constant_group(modifiers)?)
            .map(UnitEnumMember::Constant);
    }

    method(state, modifiers, enum_name).map(UnitEnumMember::Method)
}

fn backed_member(state: &mut State, enum_name: String) -> ParseResult<BackedEnumMember> {
    attributes::gather_attributes(state)?;

    if state.stream.current().kind == TokenKind::Case {
        let attributes = state.get_attributes();

        let start = state.stream.current().span;
        state.stream.next();

        let name = identifiers::identifier_maybe_reserved(state)?;

        if state.stream.current().kind == TokenKind::SemiColon {
            return Err(ParseError::MissingCaseValueForBackedEnum(
                name.to_string(),
                state.named(&enum_name),
                state.stream.current().span,
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
            attributes,
        }));
    }

    let modifiers = modifiers::collect(state)?;

    if state.stream.current().kind == TokenKind::Const {
        return constants::classish(state, modifiers::constant_group(modifiers)?)
            .map(BackedEnumMember::Constant);
    }

    method(state, modifiers, enum_name).map(BackedEnumMember::Method)
}

fn method(
    state: &mut State,
    modifiers: Vec<(Span, TokenKind, Span)>,
    enum_name: String,
) -> ParseResult<Method> {
    let method = functions::method(state, modifiers::enum_method_group(modifiers)?)?;

    match method.name.value[..].to_ascii_lowercase().as_slice() {
        b"__get" | b"__set" | b"__serialize" | b"__unserialize" | b"__destruct"
        | b"__construct" | b"__wakeup" | b"__sleep" | b"__set_state" | b"__unset" | b"__isset"
        | b"__debuginfo" | b"__clone" | b"__tostring" => {
            return Err(ParseError::EnumMayNotIncludesMagicMethod(
                state.named(&enum_name),
                method.name.to_string(),
                method.name.span,
            ))
        }
        _ => {}
    }

    Ok(method)
}
