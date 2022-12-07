use crate::expect_token;
use crate::lexer::token::TokenKind;
use crate::parser::ast::enums::BackedEnum;
use crate::parser::ast::enums::BackedEnumType;
use crate::parser::ast::enums::UnitEnum;
use crate::parser::ast::identifiers::Identifier;
use crate::parser::ast::Expression;
use crate::parser::ast::Statement;
use crate::parser::error::ParseResult;
use crate::parser::internal::attributes;
use crate::parser::internal::classish_statements;
use crate::parser::internal::identifiers;
use crate::parser::internal::modifiers;
use crate::parser::internal::parameters;
use crate::parser::internal::utils;
use crate::parser::state::Scope;
use crate::parser::state::State;
use crate::scoped;

pub fn class_definition(state: &mut State) -> ParseResult<Statement> {
    let modifiers = modifiers::class_group(modifiers::collect(state)?)?;

    utils::skip(state, TokenKind::Class)?;

    let name = identifiers::ident(state)?;

    let mut has_parent = false;
    let mut extends: Option<Identifier> = None;

    if state.current.kind == TokenKind::Extends {
        state.next();
        extends = Some(identifiers::full_name(state)?);
        has_parent = true;
    }

    let implements = if state.current.kind == TokenKind::Implements {
        state.next();

        at_least_one_comma_separated::<Identifier>(state, &identifiers::full_name)?
    } else {
        Vec::new()
    };

    let attributes = state.get_attributes();
    utils::skip_left_brace(state)?;

    let body = scoped!(
        state,
        Scope::Class(name.clone(), modifiers.clone(), has_parent),
        {
            let mut body = Vec::new();
            while state.current.kind != TokenKind::RightBrace {
                state.gather_comments();

                if state.current.kind == TokenKind::RightBrace {
                    state.clear_comments();
                    break;
                }

                body.push(classish_statements::class_like_statement(state)?);
            }

            body
        }
    );

    utils::skip_right_brace(state)?;

    Ok(Statement::Class {
        name,
        attributes,
        extends,
        implements,
        body,
        modifiers,
    })
}

pub fn interface_definition(state: &mut State) -> ParseResult<Statement> {
    utils::skip(state, TokenKind::Interface)?;

    let name = identifiers::ident(state)?;

    scoped!(state, Scope::Interface(name.clone()), {
        let extends = if state.current.kind == TokenKind::Extends {
            state.next();

            at_least_one_comma_separated::<Identifier>(state, &|state| {
                identifiers::full_name(state)
            })?
        } else {
            Vec::new()
        };

        utils::skip_left_brace(state)?;

        let attributes = state.get_attributes();

        let mut body = Vec::new();
        while state.current.kind != TokenKind::RightBrace && !state.is_eof() {
            state.gather_comments();

            if state.current.kind == TokenKind::RightBrace {
                state.clear_comments();
                break;
            }

            body.push(classish_statements::interface_statement(state)?);
        }
        utils::skip_right_brace(state)?;

        Ok(Statement::Interface {
            name,
            attributes,
            extends,
            body,
        })
    })
}

pub fn trait_definition(state: &mut State) -> ParseResult<Statement> {
    utils::skip(state, TokenKind::Trait)?;

    let name = identifiers::ident(state)?;

    scoped!(state, Scope::Trait(name.clone()), {
        utils::skip_left_brace(state)?;

        let attributes = state.get_attributes();

        let mut body = Vec::new();
        while state.current.kind != TokenKind::RightBrace && !state.is_eof() {
            state.gather_comments();

            if state.current.kind == TokenKind::RightBrace {
                state.clear_comments();
                break;
            }

            body.push(classish_statements::class_like_statement(state)?);
        }
        utils::skip_right_brace(state)?;

        Ok(Statement::Trait {
            name,
            attributes,
            body,
        })
    })
}

pub fn anonymous_class_definition(state: &mut State) -> ParseResult<Expression> {
    utils::skip(state, TokenKind::New)?;

    attributes::gather_attributes(state)?;

    utils::skip(state, TokenKind::Class)?;

    let mut args = vec![];

    if state.current.kind == TokenKind::LeftParen {
        args = parameters::args_list(state)?;
    }

    let mut has_parent = false;
    let mut extends: Option<Identifier> = None;

    if state.current.kind == TokenKind::Extends {
        state.next();
        extends = Some(identifiers::full_name(state)?);
        has_parent = true;
    }

    scoped!(state, Scope::AnonymousClass(has_parent), {
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

        utils::skip_left_brace(state)?;

        let attributes = state.get_attributes();

        let mut body = Vec::new();
        while state.current.kind != TokenKind::RightBrace && !state.is_eof() {
            body.push(classish_statements::class_like_statement(state)?);
        }

        utils::skip_right_brace(state)?;

        Ok(Expression::New {
            target: Box::new(Expression::AnonymousClass {
                attributes,
                extends,
                implements,
                body,
            }),
            args,
        })
    })
}

pub fn enum_definition(state: &mut State) -> ParseResult<Statement> {
    let start = state.current.span;

    utils::skip(state, TokenKind::Enum)?;

    let name = identifiers::ident(state)?;

    let backed_type: Option<BackedEnumType> = if state.current.kind == TokenKind::Colon {
        utils::skip_colon(state)?;

        expect_token!([
                TokenKind::Identifier(s) if s == b"string" || s == b"int" => {
                    Some(match &s[..] {
                        b"string" => BackedEnumType::String,
                        b"int" => BackedEnumType::Int,
                        _ => unreachable!(),
                    })
                },
            ], state, ["`string`", "`int`",])
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

            // TODO(azjezz): we know members might have corrupted start span, we could updated it here?
            // as we know the correct start span is `state.current.span`.
            let mut members = Vec::new();
            while state.current.kind != TokenKind::RightBrace {
                state.skip_comments();
                members.push(classish_statements::backed_enum_member(state)?);
            }

            let end = utils::skip_right_brace(state)?;

            (members, end)
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
                members.push(classish_statements::unit_enum_member(state)?);
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

fn at_least_one_comma_separated<T>(
    state: &mut State,
    func: &(dyn Fn(&mut State) -> ParseResult<T>),
) -> ParseResult<Vec<T>> {
    let mut result: Vec<T> = vec![];
    loop {
        result.push(func(state)?);
        if state.current.kind != TokenKind::Comma {
            break;
        }

        state.next();
    }

    Ok(result)
}
