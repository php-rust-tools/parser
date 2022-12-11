use crate::expect_token;
use crate::lexer::token::TokenKind;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::modifiers::VisibilityModifier;
use crate::parser::ast::traits::Trait;
use crate::parser::ast::traits::TraitMember;
use crate::parser::ast::traits::TraitUsage;
use crate::parser::ast::traits::TraitUsageAdaptation;
use crate::parser::ast::Statement;
use crate::parser::error::ParseResult;
use crate::parser::internal::attributes;
use crate::parser::internal::constants;
use crate::parser::internal::functions;
use crate::parser::internal::identifiers;
use crate::parser::internal::modifiers;
use crate::parser::internal::properties;
use crate::parser::internal::utils;
use crate::parser::state::Scope;
use crate::parser::state::State;
use crate::peek_token;
use crate::scoped;

pub fn usage(state: &mut State) -> ParseResult<TraitUsage> {
    state.next();

    let mut traits = Vec::new();

    while state.current.kind != TokenKind::SemiColon && state.current.kind != TokenKind::LeftBrace {
        let t = identifiers::full_type_name(state)?;
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
            let (r#trait, method): (Option<SimpleIdentifier>, SimpleIdentifier) =
                match state.peek.kind {
                    TokenKind::DoubleColon => {
                        let r#trait = identifiers::full_type_name(state)?;
                        state.next();
                        let method = identifiers::identifier(state)?;
                        (Some(r#trait), method)
                    }
                    _ => (None, identifiers::identifier(state)?),
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
                                    adaptations.push(TraitUsageAdaptation::Visibility {
                                        r#trait,
                                        method,
                                        visibility,
                                    });
                                } else {
                                    let alias: SimpleIdentifier = identifiers::name(state)?;
                                    adaptations.push(TraitUsageAdaptation::Alias {
                                        r#trait,
                                        method,
                                        alias,
                                        visibility: Some(visibility),
                                    });
                                }
                            }
                            _ => {
                                let alias: SimpleIdentifier = identifiers::name(state)?;
                                adaptations.push(TraitUsageAdaptation::Alias {
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
                        insteadof.push(identifiers::full_type_name(state)?);

                        if state.current.kind == TokenKind::Comma {
                            if state.peek.kind == TokenKind::SemiColon {
                                // will fail with unexpected token `,`
                                // as `insteadof` doesn't allow for trailing commas.
                                utils::skip_semicolon(state)?;
                            }

                            state.next();

                            while state.current.kind != TokenKind::SemiColon {
                                insteadof.push(identifiers::full_type_name(state)?);

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

                        adaptations.push(TraitUsageAdaptation::Precedence {
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

    Ok(TraitUsage {
        traits,
        adaptations,
    })
}

pub fn parse(state: &mut State) -> ParseResult<Statement> {
    let start = utils::skip(state, TokenKind::Trait)?;
    let name = identifiers::type_identifier(state)?;
    let class = name.name.to_string();
    let attributes = state.get_attributes();

    let (members, end) = scoped!(state, Scope::Trait(name.clone()), {
        utils::skip_left_brace(state)?;

        let mut members = Vec::new();
        while state.current.kind != TokenKind::RightBrace && !state.is_eof() {
            state.gather_comments();

            if state.current.kind == TokenKind::RightBrace {
                state.clear_comments();
                break;
            }

            members.push(member(state, class.clone())?);
        }

        (members, utils::skip_right_brace(state)?)
    });

    Ok(Statement::Trait(Trait {
        start,
        end,
        name,
        attributes,
        members,
    }))
}

fn member(state: &mut State, class: String) -> ParseResult<TraitMember> {
    let has_attributes = attributes::gather_attributes(state)?;

    if !has_attributes && state.current.kind == TokenKind::Use {
        return usage(state).map(TraitMember::TraitUsage);
    }

    if state.current.kind == TokenKind::Var {
        return properties::parse_var(state, class).map(TraitMember::VariableProperty);
    }

    let modifiers = modifiers::collect(state)?;

    if state.current.kind == TokenKind::Const {
        return constants::classish(state, modifiers::constant_group(modifiers)?)
            .map(TraitMember::Constant);
    }

    if state.current.kind == TokenKind::Function {
        return functions::method(state, modifiers::method_group(modifiers)?)
            .map(TraitMember::Method);
    }

    properties::parse(state, class, modifiers::property_group(modifiers)?)
        .map(TraitMember::Property)
}
