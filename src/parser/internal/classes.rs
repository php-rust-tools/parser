use crate::lexer::token::TokenKind;
use crate::parser::ast::classes::AnonymousClass;
use crate::parser::ast::classes::Class;
use crate::parser::ast::classes::ClassExtends;
use crate::parser::ast::classes::ClassImplements;
use crate::parser::ast::classes::ClassMember;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::Expression;
use crate::parser::ast::Statement;
use crate::parser::error::ParseResult;
use crate::parser::internal::attributes;
use crate::parser::internal::constants::classish;
use crate::parser::internal::functions::method;
use crate::parser::internal::identifiers;
use crate::parser::internal::modifiers;
use crate::parser::internal::parameters;
use crate::parser::internal::properties;
use crate::parser::internal::traits;
use crate::parser::internal::utils;
use crate::parser::state::Scope;
use crate::parser::state::State;
use crate::scoped;

pub fn parse(state: &mut State) -> ParseResult<Statement> {
    let modifiers = modifiers::class_group(modifiers::collect(state)?)?;

    let start = utils::skip(state, TokenKind::Class)?;

    let name = identifiers::type_identifier(state)?;

    let extends = if state.current.kind == TokenKind::Extends {
        let span = state.current.span;

        state.next();
        let parent = identifiers::full_type_name(state)?;

        Some(ClassExtends { span, parent })
    } else {
        None
    };

    let implements = if state.current.kind == TokenKind::Implements {
        let span = state.current.span;

        state.next();

        let interfaces =
            utils::at_least_one_comma_separated::<SimpleIdentifier>(state, &|state| {
                identifiers::full_type_name(state)
            })?;

        Some(ClassImplements { span, interfaces })
    } else {
        None
    };

    let attributes = state.get_attributes();
    utils::skip_left_brace(state)?;

    let classname = name.name.to_string();
    let members = scoped!(
        state,
        Scope::Class(name.clone(), modifiers, extends.is_some()),
        {
            let mut members = Vec::new();
            while state.current.kind != TokenKind::RightBrace {
                state.gather_comments();

                if state.current.kind == TokenKind::RightBrace {
                    state.clear_comments();
                    break;
                }

                members.push(member(state, classname.clone())?);
            }

            members
        }
    );

    let end = utils::skip_right_brace(state)?;

    Ok(Statement::Class(Class {
        start,
        end,
        name,
        extends,
        implements,
        attributes,
        members,
    }))
}

pub fn parse_anonymous(state: &mut State) -> ParseResult<Expression> {
    let span = utils::skip(state, TokenKind::New)?;

    attributes::gather_attributes(state)?;

    let start = utils::skip(state, TokenKind::Class)?;

    let mut args = vec![];

    if state.current.kind == TokenKind::LeftParen {
        args = parameters::args_list(state)?;
    }

    let extends = if state.current.kind == TokenKind::Extends {
        let span = state.current.span;

        state.next();
        let parent = identifiers::full_name(state)?;

        Some(ClassExtends { span, parent })
    } else {
        None
    };

    let implements = if state.current.kind == TokenKind::Implements {
        let span = state.current.span;

        state.next();

        let interfaces =
            utils::at_least_one_comma_separated::<SimpleIdentifier>(state, &|state| {
                identifiers::full_name(state)
            })?;

        Some(ClassImplements { span, interfaces })
    } else {
        None
    };

    let attributes = state.get_attributes();
    utils::skip_left_brace(state)?;

    let members = scoped!(state, Scope::AnonymousClass(extends.is_some()), {
        let mut members = Vec::new();
        while state.current.kind != TokenKind::RightBrace {
            state.gather_comments();

            if state.current.kind == TokenKind::RightBrace {
                state.clear_comments();
                break;
            }

            members.push(member(state, "class@anonymous".to_owned())?);
        }

        members
    });

    let end = utils::skip_right_brace(state)?;

    Ok(Expression::New {
        target: Box::new(Expression::AnonymousClass(AnonymousClass {
            start,
            end,
            extends,
            implements,
            attributes,
            members,
        })),
        span,
        args,
    })
}

fn member(state: &mut State, class: String) -> ParseResult<ClassMember> {
    let has_attributes = attributes::gather_attributes(state)?;

    if !has_attributes && state.current.kind == TokenKind::Use {
        return traits::usage(state).map(ClassMember::TraitUsage);
    }

    if state.current.kind == TokenKind::Var {
        return properties::parse_var(state, class).map(ClassMember::VariableProperty);
    }

    let modifiers = modifiers::collect(state)?;

    if state.current.kind == TokenKind::Const {
        return classish(state, modifiers::constant_group(modifiers)?).map(ClassMember::Constant);
    }

    if state.current.kind == TokenKind::Function {
        return method(state, modifiers::method_group(modifiers)?).map(ClassMember::Method);
    }

    // e.g: public static
    let modifiers = modifiers::property_group(modifiers)?;

    properties::parse(state, class, modifiers).map(ClassMember::Property)
}
