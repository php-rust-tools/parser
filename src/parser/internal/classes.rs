use crate::lexer::token::Span;
use crate::lexer::token::TokenKind;
use crate::parser::ast::classes::AnonymousClass;
use crate::parser::ast::classes::Class;
use crate::parser::ast::classes::ClassBody;
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
    let attributes = state.get_attributes();

    let modifiers = modifiers::class_group(modifiers::collect(state)?)?;
    let span = utils::skip(state, TokenKind::Class)?;
    let name = identifiers::type_identifier(state)?;
    let current = state.stream.current();
    let extends = if current.kind == TokenKind::Extends {
        let span = current.span;

        state.stream.next();
        let parent = identifiers::full_type_name(state)?;

        Some(ClassExtends { span, parent })
    } else {
        None
    };

    let current = state.stream.current();
    let implements = if current.kind == TokenKind::Implements {
        let span = current.span;

        state.stream.next();

        let interfaces =
            utils::at_least_one_comma_separated_no_trailing::<SimpleIdentifier>(state, &|state| {
                identifiers::full_type_name(state)
            })?;

        Some(ClassImplements { span, interfaces })
    } else {
        None
    };

    let classname = name.value.to_string();
    let body = scoped!(
        state,
        Scope::Class(name.clone(), modifiers.clone(), extends.is_some()),
        {
            let start = utils::skip_left_brace(state)?;

            let mut members = Vec::new();
            while state.stream.current().kind != TokenKind::RightBrace {
                members.push(member(state, classname.clone())?);
            }

            let end = utils::skip_right_brace(state)?;

            ClassBody {
                start,
                members,
                end,
            }
        }
    );

    Ok(Statement::Class(Class {
        span,
        name,
        modifiers,
        extends,
        implements,
        attributes,
        body,
    }))
}

pub fn parse_anonymous(state: &mut State, span: Option<Span>) -> ParseResult<Expression> {
    let span = match span {
        Some(span) => span,
        None => utils::skip(state, TokenKind::New)?,
    };

    attributes::gather_attributes(state)?;

    let class_span = utils::skip(state, TokenKind::Class)?;

    let arguments = if state.stream.current().kind == TokenKind::LeftParen {
        Some(parameters::argument_list(state)?)
    } else {
        None
    };

    let current = state.stream.current();
    let extends = if current.kind == TokenKind::Extends {
        let span = current.span;

        state.stream.next();
        let parent = identifiers::full_name(state)?;

        Some(ClassExtends { span, parent })
    } else {
        None
    };

    let current = state.stream.current();
    let implements = if current.kind == TokenKind::Implements {
        let span = current.span;

        state.stream.next();

        let interfaces =
            utils::at_least_one_comma_separated_no_trailing::<SimpleIdentifier>(state, &|state| {
                identifiers::full_name(state)
            })?;

        Some(ClassImplements { span, interfaces })
    } else {
        None
    };

    let attributes = state.get_attributes();

    let body = scoped!(state, Scope::AnonymousClass(extends.is_some()), {
        let start = utils::skip_left_brace(state)?;

        let mut members = Vec::new();
        while state.stream.current().kind != TokenKind::RightBrace {
            members.push(member(state, "class@anonymous".to_owned())?);
        }

        let end = utils::skip_right_brace(state)?;

        ClassBody {
            start,
            members,
            end,
        }
    });

    Ok(Expression::New {
        target: Box::new(Expression::AnonymousClass(AnonymousClass {
            span: class_span,
            extends,
            implements,
            attributes,
            body,
        })),
        span,
        arguments,
    })
}

fn member(state: &mut State, class: String) -> ParseResult<ClassMember> {
    let has_attributes = attributes::gather_attributes(state)?;

    if !has_attributes && state.stream.current().kind == TokenKind::Use {
        return traits::usage(state).map(ClassMember::TraitUsage);
    }

    if state.stream.current().kind == TokenKind::Var {
        return properties::parse_var(state, class).map(ClassMember::VariableProperty);
    }

    let modifiers = modifiers::collect(state)?;

    if state.stream.current().kind == TokenKind::Const {
        return classish(state, modifiers::constant_group(modifiers)?).map(ClassMember::Constant);
    }

    if state.stream.current().kind == TokenKind::Function {
        return method(state, modifiers::method_group(modifiers)?).map(ClassMember::Method);
    }

    // e.g: public static
    let modifiers = modifiers::property_group(modifiers)?;

    properties::parse(state, class, modifiers).map(ClassMember::Property)
}
