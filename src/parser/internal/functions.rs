use crate::expected_scope;
use crate::lexer::token::TokenKind;
use crate::parser::ast::functions::ArrowFunction;
use crate::parser::ast::functions::Closure;
use crate::parser::ast::functions::ClosureUse;
use crate::parser::ast::functions::ClosureUseVariable;
use crate::parser::ast::functions::Function;
use crate::parser::ast::functions::FunctionBody;
use crate::parser::ast::functions::Method;
use crate::parser::ast::functions::MethodBody;
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
    let current = state.stream.current();
    let r#static = if current.kind == TokenKind::Static {
        state.stream.next();

        Some(current.span)
    } else {
        None
    };

    let function = utils::skip(state, TokenKind::Function)?;

    let current = state.stream.current();
    let ampersand = if current.kind == TokenKind::Ampersand {
        state.stream.next();

        Some(current.span)
    } else {
        None
    };

    let parameters = parameters::function_parameter_list(state)?;

    let current = state.stream.current();
    let uses = if current.kind == TokenKind::Use {
        state.stream.next();

        Some(ClosureUse {
            comments: state.stream.comments(),
            r#use: current.span,
            left_parenthesis: utils::skip_left_parenthesis(state)?,
            variables: utils::comma_separated::<ClosureUseVariable>(
                state,
                &|state| {
                    let use_comments = state.stream.comments();
                    let current = state.stream.current();
                    let use_ampersand = if current.kind == TokenKind::Ampersand {
                        state.stream.next();

                        Some(current.span)
                    } else {
                        None
                    };

                    let var = variables::simple_variable(state)?;

                    Ok(ClosureUseVariable {
                        comments: use_comments,
                        variable: var,
                        ampersand: use_ampersand,
                    })
                },
                TokenKind::RightParen,
            )?,
            right_parenthesis: utils::skip_right_parenthesis(state)?,
        })
    } else {
        None
    };

    let mut return_ty = None;
    if state.stream.current().kind == TokenKind::Colon {
        utils::skip_colon(state)?;

        return_ty = Some(data_type::data_type(state)?);
    }

    let body = FunctionBody {
        comments: state.stream.comments(),
        left_brace: utils::skip_left_brace(state)?,
        statements: blocks::multiple_statements_until(state, &TokenKind::RightBrace)?,
        right_brace: utils::skip_right_brace(state)?,
    };

    Ok(Expression::Closure(Closure {
        comments,
        function,
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

    let double_arrow = utils::skip(state, TokenKind::DoubleArrow)?;

    let body = Box::new(expressions::create(state)?);

    Ok(Expression::ArrowFunction(ArrowFunction {
        comments,
        attributes,
        r#static,
        r#fn,
        ampersand,
        parameters,
        return_type,
        double_arrow,
        body,
    }))
}

pub fn function(state: &mut State) -> ParseResult<Statement> {
    let comments = state.stream.comments();

    let function = utils::skip(state, TokenKind::Function)?;

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

    let body = FunctionBody {
        comments: state.stream.comments(),
        left_brace: utils::skip_left_brace(state)?,
        statements: blocks::multiple_statements_until(state, &TokenKind::RightBrace)?,
        right_brace: utils::skip_right_brace(state)?,
    };

    Ok(Statement::Function(Function {
        comments,
        function,
        name,
        attributes,
        parameters,
        return_type,
        body,
        ampersand,
    }))
}

pub fn method(state: &mut State, modifiers: MethodModifierGroup) -> ParseResult<Method> {
    let comments = state.stream.comments();
    let attributes = state.get_attributes();
    let function = utils::skip(state, TokenKind::Function)?;

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

    let (parameters, body, return_type) =
        scoped!(state, Scope::Method(name.clone(), modifiers.clone()), {
            let parameters = parameters::method_parameter_list(state)?;

            let mut return_type = None;

            if state.stream.current().kind == TokenKind::Colon {
                utils::skip_colon(state)?;

                return_type = Some(data_type::data_type(state)?);
            }

            let body = if !has_body {
                MethodBody::Abstract(utils::skip_semicolon(state)?)
            } else {
                MethodBody::Block {
                    left_brace: utils::skip_left_brace(state)?,
                    statements: blocks::multiple_statements_until(state, &TokenKind::RightBrace)?,
                    right_brace: utils::skip_right_brace(state)?,
                }
            };

            (parameters, body, return_type)
        });

    Ok(Method {
        comments,
        function,
        attributes,
        name,
        parameters,
        body,
        return_type,
        ampersand,
        modifiers,
    })
}
