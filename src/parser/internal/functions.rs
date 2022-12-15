use crate::expected_scope;
use crate::lexer::token::TokenKind;
use crate::parser::ast::functions::ArrowFunction;
use crate::parser::ast::functions::Closure;
use crate::parser::ast::functions::ClosureUse;
use crate::parser::ast::functions::Function;
use crate::parser::ast::functions::Method;
use crate::parser::ast::modifiers::MethodModifierGroup;
use crate::parser::ast::Expression;
use crate::parser::ast::Statement;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::expressions;
use crate::parser::internal::blocks;
use crate::parser::internal::data_type;
use crate::parser::internal::identifiers;
use crate::parser::internal::parameters;
use crate::parser::internal::utils;
use crate::parser::internal::variables;
use crate::parser::state::Scope;
use crate::parser::state::State;
use crate::scoped;

pub fn anonymous_function(state: &mut State) -> ParseResult<Expression> {
    let comments = state.stream.comments();
    let attributes = state.get_attributes();
    let start = state.stream.current().span;

    let current = state.stream.current();
    let r#static = if current.kind == TokenKind::Static {
        state.stream.next();

        Some(current.span)
    } else {
        None
    };

    utils::skip(state, TokenKind::Function)?;

    let current = state.stream.current();
    let ampersand = if current.kind == TokenKind::Ampersand {
        state.stream.next();

        Some(current.span)
    } else {
        None
    };

    let parameters = parameters::function_parameter_list(state)?;

    let mut uses = vec![];
    if state.stream.current().kind == TokenKind::Use {
        state.stream.next();

        utils::skip_left_parenthesis(state)?;

        while state.stream.current().kind != TokenKind::RightParen {
            let use_comments = state.stream.comments();
            let current = state.stream.current();
            let use_ampersand = if current.kind == TokenKind::Ampersand {
                state.stream.next();

                Some(current.span)
            } else {
                None
            };

            let var = variables::simple_variable(state)?;

            uses.push(ClosureUse {
                comments: use_comments,
                variable: var,
                ampersand: use_ampersand,
            });

            if state.stream.current().kind == TokenKind::Comma {
                state.stream.next();
            } else {
                break;
            }
        }

        utils::skip_right_parenthesis(state)?;
    }

    let mut return_ty = None;
    if state.stream.current().kind == TokenKind::Colon {
        utils::skip_colon(state)?;

        return_ty = Some(data_type::data_type(state)?);
    }

    let (body, end) = scoped!(state, Scope::AnonymousFunction(r#static.is_some()), {
        utils::skip_left_brace(state)?;

        let body = blocks::body(state, &TokenKind::RightBrace)?;
        let end = utils::skip_right_brace(state)?;

        (body, end)
    });

    Ok(Expression::Closure(Closure {
        comments,
        start,
        end,
        attributes,
        parameters,
        uses,
        return_ty,
        body,
        r#static,
        ampersand,
    }))
}

pub fn arrow_function(state: &mut State) -> ParseResult<Expression> {
    let comments = state.stream.comments();
    let current = state.stream.current();
    let r#static = if current.kind == TokenKind::Static {
        state.stream.next();

        Some(current.span)
    } else {
        None
    };

    let r#fn = utils::skip(state, TokenKind::Fn)?;

    let current = state.stream.current();
    let ampersand = if state.stream.current().kind == TokenKind::Ampersand {
        state.stream.next();

        Some(current.span)
    } else {
        None
    };

    let attributes = state.get_attributes();
    let parameters = parameters::function_parameter_list(state)?;

    let mut return_type = None;
    if state.stream.current().kind == TokenKind::Colon {
        utils::skip_colon(state)?;

        return_type = Some(data_type::data_type(state)?);
    }

    utils::skip(state, TokenKind::DoubleArrow)?;

    let body = scoped!(state, Scope::ArrowFunction(r#static.is_some()), {
        Box::new(expressions::create(state)?)
    });

    Ok(Expression::ArrowFunction(ArrowFunction {
        comments,
        attributes,
        r#static,
        r#fn,
        ampersand,
        parameters,
        return_type,
        body,
    }))
}

pub fn function(state: &mut State) -> ParseResult<Statement> {
    let comments = state.stream.comments();
    let start = state.stream.current().span;

    utils::skip(state, TokenKind::Function)?;

    let current = state.stream.current();
    let ampersand = if current.kind == TokenKind::Ampersand {
        state.stream.next();

        Some(current.span)
    } else {
        None
    };

    let name = identifiers::identifier_maybe_soft_reserved(state)?;

    // get attributes before processing parameters, otherwise
    // parameters will steal attributes of this function.
    let attributes = state.get_attributes();

    let parameters = parameters::function_parameter_list(state)?;

    let mut return_type = None;

    if state.stream.current().kind == TokenKind::Colon {
        utils::skip_colon(state)?;

        return_type = Some(data_type::data_type(state)?);
    }

    let (body, end) = scoped!(state, Scope::Function(name.clone()), {
        utils::skip_left_brace(state)?;

        let body = blocks::body(state, &TokenKind::RightBrace)?;
        let end = utils::skip_right_brace(state)?;

        (body, end)
    });

    Ok(Statement::Function(Function {
        comments,
        start,
        end,
        name,
        attributes,
        parameters,
        return_type,
        body,
        ampersand,
    }))
}

pub fn method(state: &mut State, modifiers: MethodModifierGroup) -> ParseResult<Method> {
    let attributes = state.get_attributes();
    let comments = state.stream.comments();

    let start = utils::skip(state, TokenKind::Function)?;

    let current = state.stream.current();
    let ampersand = if current.kind == TokenKind::Ampersand {
        state.stream.next();

        Some(current.span)
    } else {
        None
    };

    let name = identifiers::identifier_maybe_reserved(state)?;

    let has_body = expected_scope!([
            Scope::Class(_, class_modifiers, _) => {
                if !class_modifiers.has_abstract() && modifiers.has_abstract() {
                    return Err(ParseError::AbstractModifierOnNonAbstractClassMethod(
                        state.stream.current().span,
                    ));
                }

                !modifiers.has_abstract()
            },
            Scope::Trait(_) => !modifiers.has_abstract(),
            Scope::Interface(_) => false,
            Scope::Enum(enum_name, _) => {
                if name.to_string() == "__construct" {
                    return Err(ParseError::ConstructorInEnum(
                        state.named(&enum_name),
                        state.stream.current().span,
                    ));
                }

                true
            },
            Scope::AnonymousClass(_) => true,
        ], state);

    let (parameters, body, return_type, end) =
        scoped!(state, Scope::Method(name.clone(), modifiers.clone()), {
            let parameters = parameters::method_parameter_list(state)?;

            let mut return_type = None;

            if state.stream.current().kind == TokenKind::Colon {
                utils::skip_colon(state)?;

                return_type = Some(data_type::data_type(state)?);
            }

            if !has_body {
                let end = utils::skip_semicolon(state)?;

                (parameters, None, return_type, end)
            } else {
                utils::skip_left_brace(state)?;

                let body = blocks::body(state, &TokenKind::RightBrace)?;

                let end = utils::skip_right_brace(state)?;

                (parameters, Some(body), return_type, end)
            }
        });

    Ok(Method {
        comments,
        start,
        end,
        attributes,
        name,
        parameters,
        body,
        return_type,
        ampersand,
        modifiers,
    })
}
