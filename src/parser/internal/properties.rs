use crate::lexer::token::TokenKind;
use crate::parser::ast::modifiers::PropertyModifierGroup;
use crate::parser::ast::properties::Property;
use crate::parser::ast::properties::PropertyEntry;
use crate::parser::ast::properties::VariableProperty;
use crate::parser::ast::properties::VariablePropertyEntry;
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
    loop {
        let variable = variables::simple_variable(state)?;
        let mut value = None;
        if state.stream.current().kind == TokenKind::Equals {
            state.stream.next();
            value = Some(expressions::lowest_precedence(state)?);
        }

        if modifiers.has_readonly() {
            if modifiers.has_static() {
                return Err(ParseError::StaticPropertyUsingReadonlyModifier(
                    class,
                    variable.to_string(),
                    state.stream.current().span,
                ));
            }

            if value.is_some() {
                return Err(ParseError::ReadonlyPropertyHasDefaultValue(
                    class,
                    variable.to_string(),
                    state.stream.current().span,
                ));
            }
        }

        match &ty {
            Some(ty) => {
                if ty.includes_callable() || ty.is_bottom() {
                    return Err(ParseError::ForbiddenTypeUsedInProperty(
                        class,
                        variable.to_string(),
                        ty.clone(),
                        state.stream.current().span,
                    ));
                }
            }
            None => {
                if modifiers.has_readonly() {
                    return Err(ParseError::MissingTypeForReadonlyProperty(
                        class,
                        variable.to_string(),
                        state.stream.current().span,
                    ));
                }
            }
        }

        entries.push(PropertyEntry { variable, value });

        if state.stream.current().kind == TokenKind::Comma {
            state.stream.next();
        } else {
            break;
        }
    }

    utils::skip_semicolon(state)?;

    Ok(Property {
        r#type: ty,
        modifiers,
        attributes: state.get_attributes(),
        entries,
    })
}

pub fn parse_var(state: &mut State, class: String) -> ParseResult<VariableProperty> {
    utils::skip(state, TokenKind::Var)?;

    let ty = data_type::optional_data_type(state)?;

    let mut entries = vec![];
    loop {
        let variable = variables::simple_variable(state)?;
        let mut value = None;
        if state.stream.current().kind == TokenKind::Equals {
            state.stream.next();
            value = Some(expressions::lowest_precedence(state)?);
        }

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

        entries.push(VariablePropertyEntry { variable, value });

        if state.stream.current().kind == TokenKind::Comma {
            state.stream.next();
        } else {
            break;
        }
    }

    utils::skip_semicolon(state)?;

    Ok(VariableProperty {
        r#type: ty,
        attributes: state.get_attributes(),
        entries,
    })
}
