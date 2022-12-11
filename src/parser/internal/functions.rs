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
use crate::parser::state::Scope;
use crate::parser::state::State;
use crate::scoped;

pub fn anonymous_function(state: &mut State) -> ParseResult<Expression> {
    let start = state.stream.current().span;

    let is_static = if state.stream.current().kind == TokenKind::Static {
        state.stream.next();

        true
    } else {
        false
    };

    utils::skip(state, TokenKind::Function)?;

    let by_ref = if state.stream.current().kind == TokenKind::Ampersand {
        state.stream.next();
        true
    } else {
        false
    };

    let attributes = state.get_attributes();
    let parameters = parameters::function_parameter_list(state)?;

    let mut uses = vec![];
    if state.stream.current().kind == TokenKind::Use {
        state.stream.next();

        utils::skip_left_parenthesis(state)?;

        while state.stream.current().kind != TokenKind::RightParen {
            let mut by_ref = false;
            if state.stream.current().kind == TokenKind::Ampersand {
                state.stream.next();

                by_ref = true;
            }

            // TODO(azjezz): this shouldn't call expr, we should have a function
            // just for variables, so we don't have to go through the whole `match` in `expression(...)`
            let var = match expressions::lowest_precedence(state)? {
                s @ Expression::Variable { .. } => ClosureUse { var: s, by_ref },
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        "expected variable".into(),
                        state.stream.current().span,
                    ))
                }
            };

            uses.push(var);

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

    let (body, end) = scoped!(state, Scope::AnonymousFunction(is_static), {
        utils::skip_left_brace(state)?;

        let body = blocks::body(state, &TokenKind::RightBrace)?;
        let end = utils::skip_right_brace(state)?;

        (body, end)
    });

    Ok(Expression::Closure(Closure {
        start,
        end,
        attributes,
        parameters,
        uses,
        return_ty,
        body,
        r#static: is_static,
        by_ref,
    }))
}

pub fn arrow_function(state: &mut State) -> ParseResult<Expression> {
    let start = state.stream.current().span;

    let is_static = if state.stream.current().kind == TokenKind::Static {
        state.stream.next();

        true
    } else {
        false
    };

    utils::skip(state, TokenKind::Fn)?;

    let by_ref = if state.stream.current().kind == TokenKind::Ampersand {
        state.stream.next();
        true
    } else {
        false
    };

    let attributes = state.get_attributes();
    let parameters = parameters::function_parameter_list(state)?;

    let mut return_type = None;
    if state.stream.current().kind == TokenKind::Colon {
        utils::skip_colon(state)?;

        return_type = Some(data_type::data_type(state)?);
    }

    utils::skip(state, TokenKind::DoubleArrow)?;

    let body = scoped!(state, Scope::ArrowFunction(is_static), {
        Box::new(expressions::lowest_precedence(state)?)
    });

    let end = state.stream.current().span;

    Ok(Expression::ArrowFunction(ArrowFunction {
        start,
        end,
        attributes,
        parameters,
        return_type,
        body,
        by_ref,
        r#static: is_static,
    }))
}

pub fn function(state: &mut State) -> ParseResult<Statement> {
    let start = state.stream.current().span;

    utils::skip(state, TokenKind::Function)?;

    let by_ref = if state.stream.current().kind == TokenKind::Ampersand {
        state.stream.next();
        true
    } else {
        false
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
        start,
        end,
        name,
        attributes,
        parameters,
        return_type,
        body,
        by_ref,
    }))
}

pub fn method(state: &mut State, modifiers: MethodModifierGroup) -> ParseResult<Method> {
    let start = utils::skip(state, TokenKind::Function)?;

    let by_ref = if state.stream.current().kind == TokenKind::Ampersand {
        state.stream.next();
        true
    } else {
        false
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

    // get attributes before processing parameters, otherwise
    // parameters will steal attributes of this method.
    let attributes = state.get_attributes();

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
        start,
        end,
        attributes,
        name,
        parameters,
        body,
        return_type,
        by_ref,
        modifiers,
    })
}
