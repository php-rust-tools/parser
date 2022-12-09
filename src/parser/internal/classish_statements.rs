use crate::expect_token;
use crate::expected_scope;
use crate::lexer::token::TokenKind;
use crate::parser::ast::constant::ClassishConstant;
use crate::parser::ast::constant::ConstantEntry;
use crate::parser::ast::identifiers::Identifier;
use crate::parser::ast::modifiers::ConstantModifierGroup;
use crate::parser::ast::modifiers::VisibilityModifier;
use crate::parser::ast::Statement;
use crate::parser::ast::TraitAdaptation;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::expressions;
use crate::parser::internal::attributes;
use crate::parser::internal::data_type;
use crate::parser::internal::functions;
use crate::parser::internal::identifiers;
use crate::parser::internal::modifiers;
use crate::parser::internal::utils;
use crate::parser::state::Scope;
use crate::parser::state::State;
use crate::peek_token;

pub fn class_like_statement(state: &mut State) -> ParseResult<Statement> {
    let has_attributes = attributes::gather_attributes(state)?;

    let modifiers = modifiers::collect(state)?;

    if !has_attributes && state.current.kind == TokenKind::Use {
        return parse_classish_uses(state);
    }

    if state.current.kind == TokenKind::Const {
        return Ok(Statement::ClassishConstant(constant(
            state,
            modifiers::constant_group(modifiers)?,
        )?));
    }

    if state.current.kind == TokenKind::Function {
        return Ok(Statement::Method(functions::method(
            state,
            modifiers::method_group(modifiers)?,
        )?));
    }

    // e.g: public static
    let modifiers = modifiers::property_group(modifiers)?;
    // e.g: string
    let ty = data_type::optional_data_type(state)?;
    // e.g: $name
    let var = identifiers::var(state)?;

    let mut value = None;
    // e.g: = "foo";
    if state.current.kind == TokenKind::Equals {
        state.next();
        value = Some(expressions::lowest_precedence(state)?);
    }

    let class_name: String = expected_scope!([
            Scope::Trait(name) | Scope::Class(name, _, _) => state.named(&name),
            Scope::AnonymousClass(_) => state.named("class@anonymous"),
        ], state);

    if modifiers.has_readonly() {
        if modifiers.has_static() {
            return Err(ParseError::StaticPropertyUsingReadonlyModifier(
                class_name,
                var.to_string(),
                state.current.span,
            ));
        }

        if value.is_some() {
            return Err(ParseError::ReadonlyPropertyHasDefaultValue(
                class_name,
                var.to_string(),
                state.current.span,
            ));
        }
    }

    match &ty {
        Some(ty) => {
            if ty.includes_callable() || ty.is_bottom() {
                return Err(ParseError::ForbiddenTypeUsedInProperty(
                    class_name,
                    var.to_string(),
                    ty.clone(),
                    state.current.span,
                ));
            }
        }
        None => {
            if modifiers.has_readonly() {
                return Err(ParseError::MissingTypeForReadonlyProperty(
                    class_name,
                    var.to_string(),
                    state.current.span,
                ));
            }
        }
    }

    utils::skip_semicolon(state)?;

    Ok(Statement::Property {
        var,
        value,
        r#type: ty,
        modifiers,
        attributes: state.get_attributes(),
    })
}

fn parse_classish_uses(state: &mut State) -> ParseResult<Statement> {
    state.next();

    let mut traits = Vec::new();

    while state.current.kind != TokenKind::SemiColon && state.current.kind != TokenKind::LeftBrace {
        let t = identifiers::full_name(state)?;
        traits.push(t);

        if state.current.kind == TokenKind::Comma {
            if state.peek.kind == TokenKind::SemiColon {
                // will fail with unexpected token `,`
                // as `use` doesn't allow for trailing commas.
                utils::skip_semicolon(state)?;
            } else if state.peek.kind == TokenKind::LeftBrace {
                // will fail with unexpected token `{`
                // as `use` doesn't allow for trailing commas.
                utils::skip_left_brace(state)?;
            } else {
                state.next();
            }
        } else {
            break;
        }
    }

    let mut adaptations = Vec::new();
    if state.current.kind == TokenKind::LeftBrace {
        utils::skip_left_brace(state)?;

        while state.current.kind != TokenKind::RightBrace {
            let (r#trait, method): (Option<Identifier>, Identifier) = match state.peek.kind {
                TokenKind::DoubleColon => {
                    let r#trait = identifiers::full_name(state)?;
                    state.next();
                    let method = identifiers::ident(state)?;
                    (Some(r#trait), method)
                }
                _ => (None, identifiers::ident(state)?),
            };

            expect_token!([
                    TokenKind::As => {
                        match state.current.kind {
                            TokenKind::Public | TokenKind::Protected | TokenKind::Private => {
                                let visibility = peek_token!([
                                    TokenKind::Public => VisibilityModifier::Public {
                                        start: state.current.span,
                                        end: state.peek.span
                                    },
                                    TokenKind::Protected => VisibilityModifier::Protected {
                                        start: state.current.span,
                                        end: state.peek.span
                                    },
                                    TokenKind::Private => VisibilityModifier::Private {
                                        start: state.current.span,
                                        end: state.peek.span
                                    },
                                ], state, ["`private`", "`protected`", "`public`"]);
                                state.next();

                                if state.current.kind == TokenKind::SemiColon {
                                    adaptations.push(TraitAdaptation::Visibility {
                                        r#trait,
                                        method,
                                        visibility,
                                    });
                                } else {
                                    let alias: Identifier = identifiers::name(state)?;
                                    adaptations.push(TraitAdaptation::Alias {
                                        r#trait,
                                        method,
                                        alias,
                                        visibility: Some(visibility),
                                    });
                                }
                            }
                            _ => {
                                let alias: Identifier = identifiers::name(state)?;
                                adaptations.push(TraitAdaptation::Alias {
                                    r#trait,
                                    method,
                                    alias,
                                    visibility: None,
                                });
                            }
                        }
                    },
                    TokenKind::Insteadof => {
                        let mut insteadof = Vec::new();
                        insteadof.push(identifiers::full_name(state)?);

                        if state.current.kind == TokenKind::Comma {
                            if state.peek.kind == TokenKind::SemiColon {
                                // will fail with unexpected token `,`
                                // as `insteadof` doesn't allow for trailing commas.
                                utils::skip_semicolon(state)?;
                            }

                            state.next();

                            while state.current.kind != TokenKind::SemiColon {
                                insteadof.push(identifiers::full_name(state)?);

                                if state.current.kind == TokenKind::Comma {
                                    if state.peek.kind == TokenKind::SemiColon {
                                        // will fail with unexpected token `,`
                                        // as `insteadof` doesn't allow for trailing commas.
                                        utils::skip_semicolon(state)?;
                                    } else {
                                        state.next();
                                    }
                                } else {
                                    break;
                                }
                            }
                        }

                        adaptations.push(TraitAdaptation::Precedence {
                            r#trait,
                            method,
                            insteadof,
                        });
                    }
                ], state, ["`as`", "`insteadof`"]);

            utils::skip_semicolon(state)?;
        }

        utils::skip_right_brace(state)?;
    } else {
        utils::skip_semicolon(state)?;
    }

    Ok(Statement::TraitUse {
        traits,
        adaptations,
    })
}

pub fn constant(
    state: &mut State,
    modifiers: ConstantModifierGroup,
) -> ParseResult<ClassishConstant> {
    let start = state.current.span;

    state.next();

    let attributes = state.get_attributes();

    let mut entries = vec![];

    loop {
        let name = identifiers::ident(state)?;

        utils::skip(state, TokenKind::Equals)?;

        let value = expressions::lowest_precedence(state)?;

        entries.push(ConstantEntry { name, value });

        if state.current.kind == TokenKind::Comma {
            state.next();
        } else {
            break;
        }
    }

    let end = utils::skip_semicolon(state)?;

    Ok(ClassishConstant {
        start,
        end,
        attributes,
        modifiers,
        entries,
    })
}
