use crate::lexer::token::TokenKind;
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

        utils::at_least_one_comma_separated::<Identifier>(state, &identifiers::full_name)?
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
