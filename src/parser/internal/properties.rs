use crate::lexer::token::TokenKind;
use crate::parser::ast::modifiers::PropertyModifierGroup;
use crate::parser::ast::properties::Property;
use crate::parser::ast::properties::PropertyEntry;
use crate::parser::ast::properties::VariableProperty;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::expressions;
use crate::parser::internal::data_type;
use crate::parser::internal::utils;
use crate::parser::internal::variables;
use crate::parser::state::State;

pub fn parse(
    state: &mut State,
    class: String,
    modifiers: PropertyModifierGroup,
) -> ParseResult<Property> {
    let ty = data_type::optional_data_type(state)?;

    let mut entries = vec![];
    let mut type_checked = false;
    loop {
        let current = state.stream.current();
        let variable = variables::simple_variable(state)?;

        if !type_checked {
            type_checked = true;
            if modifiers.has_readonly() && modifiers.has_static() {
                return Err(ParseError::StaticPropertyUsingReadonlyModifier(
                    class,
                    variable.to_string(),
                    current.span,
                ));
            }

            match &ty {
                Some(ty) => {
                    if ty.includes_callable() || ty.is_bottom() {
                        return Err(ParseError::ForbiddenTypeUsedInProperty(
                            class,
                            variable.to_string(),
                            ty.clone(),
                            current.span,
                        ));
                    }
                }
                None => {
                    if modifiers.has_readonly() {
                        return Err(ParseError::MissingTypeForReadonlyProperty(
                            class,
                            variable.to_string(),
                            current.span,
                        ));
                    }
                }
            }
        }

        let current = state.stream.current();
        if current.kind == TokenKind::Equals {
            if modifiers.has_readonly() {
                return Err(ParseError::ReadonlyPropertyHasDefaultValue(
                    class,
                    variable.to_string(),
                    current.span,
                ));
            }

            state.stream.next();
            let value = expressions::create(state)?;

            entries.push(PropertyEntry::Initialized {
                variable,
                span: current.span,
                value,
            });
        } else {
            entries.push(PropertyEntry::Uninitialized { variable });
        }

        if state.stream.current().kind == TokenKind::Comma {
            state.stream.next();
        } else {
            break;
        }
    }

    let end = utils::skip_semicolon(state)?;

    Ok(Property {
        r#type: ty,
        modifiers,
        attributes: state.get_attributes(),
        entries,
        end,
    })
}

pub fn parse_var(state: &mut State, class: String) -> ParseResult<VariableProperty> {
    utils::skip(state, TokenKind::Var)?;

    let ty = data_type::optional_data_type(state)?;

    let mut entries = vec![];
    let mut type_checked = false;
    loop {
        let variable = variables::simple_variable(state)?;

        if !type_checked {
            type_checked = true;

            if let Some(ty) = &ty {
                if ty.includes_callable() || ty.is_bottom() {
                    return Err(ParseError::ForbiddenTypeUsedInProperty(
                        class,
                        variable.to_string(),
                        ty.clone(),
                        state.stream.current().span,
                    ));
                }
            }
        }

        let current = state.stream.current();
        if current.kind == TokenKind::Equals {
            let span = current.span;
            state.stream.next();
            let value = expressions::create(state)?;

            entries.push(PropertyEntry::Initialized {
                variable,
                span,
                value,
            });
        } else {
            entries.push(PropertyEntry::Uninitialized { variable });
        }

        if state.stream.current().kind == TokenKind::Comma {
            state.stream.next();
        } else {
            break;
        }
    }

    let end = utils::skip_semicolon(state)?;

    Ok(VariableProperty {
        r#type: ty,
        attributes: state.get_attributes(),
        entries,
        end,
    })
}
