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
use crate::parser::ast::functions::ConcreteMethod;
use crate::parser::ast::Statement;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::expressions;
use crate::parser::internal::attributes;
use crate::parser::internal::constants;
use crate::parser::internal::functions;
use crate::parser::internal::functions::Method;
use crate::parser::internal::identifiers;
use crate::parser::internal::modifiers;
use crate::parser::internal::utils;
use crate::parser::state::State;

pub fn parse(state: &mut State) -> ParseResult<Statement> {
    let span = utils::skip(state, TokenKind::Enum)?;

    let name = identifiers::type_identifier(state)?;

    let backed_type: Option<BackedEnumType> = if state.stream.current().kind == TokenKind::Colon {
        let span = utils::skip_colon(state)?;

        let identifier = identifiers::identifier_of(state, &["string", "int"])?;
        Some(match &identifier.value[..] {
            b"string" => BackedEnumType::String(span, identifier.span),
            b"int" => BackedEnumType::Int(span, identifier.span),
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
    let enum_name = name.value.to_string();
    if let Some(backed_type) = backed_type {
        let body = BackedEnumBody {
            left_brace: utils::skip_left_brace(state)?,
            members: {
                let mut members = Vec::new();
                while state.stream.current().kind != TokenKind::RightBrace {
                    members.push(backed_member(state, &enum_name)?);
                }
                members
            },
            right_brace: utils::skip_right_brace(state)?,
        };

        Ok(Statement::BackedEnum(BackedEnum {
            span,
            name,
            backed_type,
            attributes,
            implements,
            body,
        }))
    } else {
        let body = UnitEnumBody {
            left_brace: utils::skip_left_brace(state)?,
            members: {
                let mut members = Vec::new();
                while state.stream.current().kind != TokenKind::RightBrace {
                    members.push(unit_member(state, &enum_name)?);
                }
                members
            },
            right_brace: utils::skip_right_brace(state)?,
        };

        Ok(Statement::UnitEnum(UnitEnum {
            span,
            name,
            attributes,
            implements,
            body,
        }))
    }
}

fn unit_member(state: &mut State, enum_name: &str) -> ParseResult<UnitEnumMember> {
    attributes::gather_attributes(state)?;

    let current = state.stream.current();
    if current.kind == TokenKind::Case {
        let attributes = state.get_attributes();

        let start = current.span;
        state.stream.next();

        let name = identifiers::identifier_maybe_reserved(state)?;

        let current = state.stream.current();
        if current.kind == TokenKind::Equals {
            return Err(ParseError::CaseValueForUnitEnum(
                name.to_string(),
                state.named(enum_name),
                current.span,
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

fn backed_member(state: &mut State, enum_name: &str) -> ParseResult<BackedEnumMember> {
    attributes::gather_attributes(state)?;

    let current = state.stream.current();
    if current.kind == TokenKind::Case {
        let attributes = state.get_attributes();

        let case = current.span;
        state.stream.next();

        let name = identifiers::identifier_maybe_reserved(state)?;

        let current = state.stream.current();
        if current.kind == TokenKind::SemiColon {
            return Err(ParseError::MissingCaseValueForBackedEnum(
                name.to_string(),
                state.named(enum_name),
                current.span,
            ));
        }

        let equals = utils::skip(state, TokenKind::Equals)?;

        let value = expressions::create(state)?;

        let semicolon = utils::skip_semicolon(state)?;

        return Ok(BackedEnumMember::Case(BackedEnumCase {
            attributes,
            case,
            name,
            equals,
            value,
            semicolon,
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
    modifiers: Vec<(Span, TokenKind)>,
    enum_name: &str,
) -> ParseResult<ConcreteMethod> {
    let method = functions::method(
        state,
        functions::MethodType::Concrete,
        modifiers::enum_method_group(modifiers)?,
        enum_name,
    )?;

    match method {
        Method::ConcreteConstructor(constructor) => Err(ParseError::ConstructorInEnum(
            state.named(&enum_name),
            constructor.name.span,
        )),
        Method::Concrete(method) => {
            match method.name.value[..].to_ascii_lowercase().as_slice() {
                b"__get" | b"__set" | b"__serialize" | b"__unserialize" | b"__destruct"
                | b"__wakeup" | b"__sleep" | b"__set_state" | b"__unset" | b"__isset"
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
        Method::Abstract(_) | Method::AbstractConstructor(_) => unreachable!(),
    }
}
