use super::identifiers;
use crate::lexer::token::TokenKind;
use crate::parser::ast::functions::FunctionParameter;
use crate::parser::ast::functions::FunctionParameterList;
use crate::parser::ast::functions::MethodParameter;
use crate::parser::ast::functions::MethodParameterList;
use crate::parser::ast::Arg;
use crate::parser::ast::Expression;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::expressions;
use crate::parser::internal::attributes;
use crate::parser::internal::data_type;
use crate::parser::internal::modifiers;
use crate::parser::internal::utils;
use crate::parser::internal::variables;
use crate::parser::state::Scope;
use crate::parser::state::State;

pub fn function_parameter_list(state: &mut State) -> Result<FunctionParameterList, ParseError> {
    let mut members = Vec::new();

    let list_start = state.stream.current().span;
    utils::skip_left_parenthesis(state)?;

    while !state.stream.is_eof() && state.stream.current().kind != TokenKind::RightParen {
        let start = state.stream.current().span;

        attributes::gather_attributes(state)?;

        let ty = data_type::optional_data_type(state)?;

        let mut variadic = false;
        let mut by_ref = false;

        if state.stream.current().kind == TokenKind::Ampersand {
            state.stream.next();
            by_ref = true;
        }

        if state.stream.current().kind == TokenKind::Ellipsis {
            state.stream.next();
            variadic = true;
        }

        // 2. Then expect a variable.
        let var = variables::simple_variable(state)?;

        let mut default = None;
        if state.stream.current().kind == TokenKind::Equals {
            state.stream.next();
            default = Some(expressions::lowest_precedence(state)?);
        }

        let end = state.stream.current().span;

        members.push(FunctionParameter {
            start,
            end,
            name: var,
            attributes: state.get_attributes(),
            r#type: ty,
            variadic,
            default,
            by_ref,
        });

        if state.stream.current().kind == TokenKind::Comma {
            state.stream.next();
        } else {
            break;
        }
    }

    utils::skip_right_parenthesis(state)?;

    let list_end = state.stream.current().span;

    Ok(FunctionParameterList {
        start: list_start,
        end: list_end,
        members,
    })
}

/// TODO(azjezz): split this into `method_parameter_list` and `abstract_method_parameter_list`?
///               abstract method parameter list won't have a promoted property, so some of the logic
///               here can be avoided for performance.
pub fn method_parameter_list(state: &mut State) -> Result<MethodParameterList, ParseError> {
    let mut class_name = String::new();
    let construct: i8 = match state.scope()? {
        Scope::Method(name, modifiers) => {
            if name.to_string() != "__construct" {
                0
            } else {
                match state.parent()? {
                    // can only have abstract ctor
                    Scope::Interface(_) => 1,
                    // can only have concret ctor
                    Scope::AnonymousClass(_) => {
                        class_name = state.named("class@anonymous");

                        2
                    }
                    // can have either abstract or concret ctor,
                    // depens on method modifiers.
                    Scope::Class(name, _, _) | Scope::Trait(name) => {
                        if modifiers.has_abstract() {
                            1
                        } else {
                            class_name = state.named(name);

                            2
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
        scope => unreachable!("shouldn't reach scope `{:?}`", scope),
    };

    let mut members = Vec::new();

    let list_start = state.stream.current().span;
    utils::skip_left_parenthesis(state)?;

    while !state.stream.is_eof() && state.stream.current().kind != TokenKind::RightParen {
        let start = state.stream.current().span;

        attributes::gather_attributes(state)?;

        let modifiers = modifiers::promoted_property_group(modifiers::collect(state)?)?;

        let ty = data_type::optional_data_type(state)?;

        let mut variadic = false;
        let mut by_ref = false;

        if matches!(state.stream.current().kind, TokenKind::Ampersand) {
            state.stream.next();
            by_ref = true;
        }

        if matches!(state.stream.current().kind, TokenKind::Ellipsis) {
            state.stream.next();
            if !modifiers.is_empty() {
                return Err(ParseError::VariadicPromotedProperty(
                    state.stream.current().span,
                ));
            }

            variadic = true;
        }

        // 2. Then expect a variable.
        let var = variables::simple_variable(state)?;

        if !modifiers.is_empty() {
            match construct {
                0 => {
                    return Err(ParseError::PromotedPropertyOutsideConstructor(
                        state.stream.current().span,
                    ));
                }
                1 => {
                    return Err(ParseError::PromotedPropertyOnAbstractConstructor(
                        state.stream.current().span,
                    ));
                }
                _ => {}
            }

            match &ty {
                Some(ty) => {
                    if ty.includes_callable() || ty.is_bottom() {
                        return Err(ParseError::ForbiddenTypeUsedInProperty(
                            class_name,
                            var.to_string(),
                            ty.clone(),
                            state.stream.current().span,
                        ));
                    }
                }
                None => {
                    if modifiers.has_readonly() {
                        return Err(ParseError::MissingTypeForReadonlyProperty(
                            class_name,
                            var.to_string(),
                            state.stream.current().span,
                        ));
                    }
                }
            }
        }

        let mut default = None;
        if state.stream.current().kind == TokenKind::Equals {
            state.stream.next();
            default = Some(expressions::lowest_precedence(state)?);
        }

        let end = state.stream.current().span;

        members.push(MethodParameter {
            start,
            end,
            name: var,
            attributes: state.get_attributes(),
            r#type: ty,
            variadic,
            default,
            modifiers,
            by_ref,
        });

        if state.stream.current().kind == TokenKind::Comma {
            state.stream.next();
        } else {
            break;
        }
    }

    utils::skip_right_parenthesis(state)?;

    let list_end = state.stream.current().span;

    Ok(MethodParameterList {
        start: list_start,
        end: list_end,
        members,
    })
}

pub fn args_list(state: &mut State) -> ParseResult<Vec<Arg>> {
    utils::skip_left_parenthesis(state)?;

    let mut args = Vec::new();
    let mut has_used_named_arguments = false;

    while !state.stream.is_eof() && state.stream.current().kind != TokenKind::RightParen {
        let mut name = None;
        let mut unpack = false;
        if identifiers::is_identifier_maybe_reserved(&state.stream.current().kind)
            && state.stream.peek().kind == TokenKind::Colon
        {
            name = Some(identifiers::identifier_maybe_reserved(state)?);
            has_used_named_arguments = true;
            state.stream.next();
        } else if state.stream.current().kind == TokenKind::Ellipsis {
            state.stream.next();
            unpack = true;
        }

        if name.is_none() && has_used_named_arguments {
            return Err(ParseError::CannotUsePositionalArgumentAfterNamedArgument(
                state.stream.current().span,
            ));
        }

        if unpack && state.stream.current().kind == TokenKind::RightParen {
            args.push(Arg {
                name: None,
                unpack: false,
                value: Expression::VariadicPlaceholder,
            });

            break;
        }

        let value = expressions::lowest_precedence(state)?;

        args.push(Arg {
            name,
            unpack,
            value,
        });

        if state.stream.current().kind == TokenKind::Comma {
            state.stream.next();
        } else {
            break;
        }
    }

    utils::skip_right_parenthesis(state)?;

    Ok(args)
}
