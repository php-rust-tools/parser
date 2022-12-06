use crate::lexer::token::Span;
use crate::lexer::token::TokenKind;
use crate::parser::ast::modifiers::ClassModifier;
use crate::parser::ast::modifiers::ClassModifierGroup;
use crate::parser::ast::modifiers::ConstantModifier;
use crate::parser::ast::modifiers::ConstantModifierGroup;
use crate::parser::ast::modifiers::MethodModifier;
use crate::parser::ast::modifiers::MethodModifierGroup;
use crate::parser::ast::modifiers::PromotedPropertyModifier;
use crate::parser::ast::modifiers::PromotedPropertyModifierGroup;
use crate::parser::ast::modifiers::PropertyModifier;
use crate::parser::ast::modifiers::PropertyModifierGroup;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::state::State;
use crate::parser::Parser;

impl Parser {
    pub(in crate::parser) fn get_class_modifier_group(
        &self,
        input: Vec<(Span, TokenKind, Span)>,
    ) -> ParseResult<ClassModifierGroup> {
        let mut has_final = false;
        let mut has_abstract = false;

        let modifiers = input
            .iter()
            .map(|(start, token, end)| match token {
                TokenKind::Readonly => Ok(ClassModifier::Readonly {
                    start: *start,
                    end: *end,
                }),
                TokenKind::Final => {
                    has_final = true;
                    if has_abstract {
                        Err(ParseError::FinalModifierOnAbstractClassMember(*start))
                    } else {
                        Ok(ClassModifier::Final {
                            start: *start,
                            end: *end,
                        })
                    }
                }
                TokenKind::Abstract => {
                    has_abstract = true;
                    if has_final {
                        Err(ParseError::FinalModifierOnAbstractClassMember(*start))
                    } else {
                        Ok(ClassModifier::Abstract {
                            start: *start,
                            end: *end,
                        })
                    }
                }
                _ => Err(ParseError::CannotUseModifierOnClass(
                    token.to_string(),
                    *start,
                )),
            })
            .collect::<ParseResult<Vec<ClassModifier>>>()?;

        Ok(ClassModifierGroup { modifiers })
    }

    pub(in crate::parser) fn get_method_modifier_group(
        &self,
        input: Vec<(Span, TokenKind, Span)>,
    ) -> ParseResult<MethodModifierGroup> {
        let mut has_final = false;
        let mut has_abstract = false;

        let modifiers = input
            .iter()
            .map(|(start, token, end)| match token {
                TokenKind::Final => {
                    has_final = true;
                    if has_abstract {
                        Err(ParseError::FinalModifierOnAbstractClassMember(*start))
                    } else {
                        Ok(MethodModifier::Final {
                            start: *start,
                            end: *end,
                        })
                    }
                }
                TokenKind::Abstract => {
                    has_abstract = true;
                    if has_final {
                        Err(ParseError::FinalModifierOnAbstractClassMember(*start))
                    } else {
                        Ok(MethodModifier::Abstract {
                            start: *start,
                            end: *end,
                        })
                    }
                }
                TokenKind::Private => Ok(MethodModifier::Private {
                    start: *start,
                    end: *end,
                }),
                TokenKind::Protected => Ok(MethodModifier::Protected {
                    start: *start,
                    end: *end,
                }),
                TokenKind::Public => Ok(MethodModifier::Public {
                    start: *start,
                    end: *end,
                }),
                TokenKind::Static => Ok(MethodModifier::Static {
                    start: *start,
                    end: *end,
                }),
                _ => Err(ParseError::CannotUseModifierOnClassMethod(
                    token.to_string(),
                    *start,
                )),
            })
            .collect::<ParseResult<Vec<MethodModifier>>>()?;

        Ok(MethodModifierGroup { modifiers })
    }

    pub(in crate::parser) fn get_enum_method_modifier_group(
        &self,
        input: Vec<(Span, TokenKind, Span)>,
    ) -> ParseResult<MethodModifierGroup> {
        let modifiers = input
            .iter()
            .map(|(start, token, end)| match token {
                TokenKind::Final => Ok(MethodModifier::Final {
                    start: *start,
                    end: *end,
                }),
                TokenKind::Private => Ok(MethodModifier::Private {
                    start: *start,
                    end: *end,
                }),
                TokenKind::Protected => Ok(MethodModifier::Protected {
                    start: *start,
                    end: *end,
                }),
                TokenKind::Public => Ok(MethodModifier::Public {
                    start: *start,
                    end: *end,
                }),
                TokenKind::Static => Ok(MethodModifier::Static {
                    start: *start,
                    end: *end,
                }),
                _ => Err(ParseError::CannotUseModifierOnEnumMethod(
                    token.to_string(),
                    *start,
                )),
            })
            .collect::<ParseResult<Vec<MethodModifier>>>()?;

        Ok(MethodModifierGroup { modifiers })
    }

    pub(in crate::parser) fn get_interface_method_modifier_group(
        &self,
        input: Vec<(Span, TokenKind, Span)>,
    ) -> ParseResult<MethodModifierGroup> {
        let modifiers = input
            .iter()
            .map(|(start, token, end)| match token {
                TokenKind::Public => Ok(MethodModifier::Public {
                    start: *start,
                    end: *end,
                }),
                TokenKind::Static => Ok(MethodModifier::Static {
                    start: *start,
                    end: *end,
                }),
                _ => Err(ParseError::CannotUseModifierOnInterfaceMethod(
                    token.to_string(),
                    *start,
                )),
            })
            .collect::<ParseResult<Vec<MethodModifier>>>()?;

        Ok(MethodModifierGroup { modifiers })
    }

    pub(in crate::parser) fn get_property_modifier_group(
        &self,
        input: Vec<(Span, TokenKind, Span)>,
    ) -> ParseResult<PropertyModifierGroup> {
        let modifiers = input
            .iter()
            .map(|(start, token, end)| match token {
                TokenKind::Readonly => Ok(PropertyModifier::Readonly {
                    start: *start,
                    end: *end,
                }),
                TokenKind::Static => Ok(PropertyModifier::Static {
                    start: *start,
                    end: *end,
                }),
                // TODO(azjezz): figure out more about the logic of `var` keyword.
                TokenKind::Public | TokenKind::Var => Ok(PropertyModifier::Public {
                    start: *start,
                    end: *end,
                }),
                TokenKind::Protected => Ok(PropertyModifier::Protected {
                    start: *start,
                    end: *end,
                }),
                TokenKind::Private => Ok(PropertyModifier::Private {
                    start: *start,
                    end: *end,
                }),
                _ => Err(ParseError::CannotUseModifierOnProperty(
                    token.to_string(),
                    *start,
                )),
            })
            .collect::<ParseResult<Vec<PropertyModifier>>>()?;

        Ok(PropertyModifierGroup { modifiers })
    }

    pub(in crate::parser) fn get_promoted_property_modifier_group(
        &self,
        input: Vec<(Span, TokenKind, Span)>,
    ) -> ParseResult<PromotedPropertyModifierGroup> {
        let modifiers = input
            .iter()
            .map(|(start, token, end)| match token {
                TokenKind::Readonly => Ok(PromotedPropertyModifier::Readonly {
                    start: *start,
                    end: *end,
                }),
                TokenKind::Private => Ok(PromotedPropertyModifier::Private {
                    start: *start,
                    end: *end,
                }),
                TokenKind::Protected => Ok(PromotedPropertyModifier::Protected {
                    start: *start,
                    end: *end,
                }),
                TokenKind::Public => Ok(PromotedPropertyModifier::Public {
                    start: *start,
                    end: *end,
                }),
                _ => Err(ParseError::CannotUseModifierOnPromotedProperty(
                    token.to_string(),
                    *start,
                )),
            })
            .collect::<ParseResult<Vec<PromotedPropertyModifier>>>()?;

        Ok(PromotedPropertyModifierGroup { modifiers })
    }

    pub(in crate::parser) fn get_constant_modifier_group(
        &self,
        input: Vec<(Span, TokenKind, Span)>,
    ) -> ParseResult<ConstantModifierGroup> {
        let mut has_final = false;
        let mut has_private = false;

        let modifiers = input
            .iter()
            .map(|(start, token, end)| match token {
                TokenKind::Protected => Ok(ConstantModifier::Protected {
                    start: *start,
                    end: *end,
                }),
                TokenKind::Public => Ok(ConstantModifier::Public {
                    start: *start,
                    end: *end,
                }),
                TokenKind::Private => {
                    has_private = true;
                    if has_final {
                        Err(ParseError::FinalModifierOnPrivateConstant(*start))
                    } else {
                        Ok(ConstantModifier::Private {
                            start: *start,
                            end: *end,
                        })
                    }
                }
                TokenKind::Final => {
                    has_final = true;
                    if has_private {
                        Err(ParseError::FinalModifierOnPrivateConstant(*start))
                    } else {
                        Ok(ConstantModifier::Final {
                            start: *start,
                            end: *end,
                        })
                    }
                }
                _ => Err(ParseError::CannotUseModifierOnConstant(
                    token.to_string(),
                    *start,
                )),
            })
            .collect::<ParseResult<Vec<ConstantModifier>>>()?;

        Ok(ConstantModifierGroup { modifiers })
    }

    pub(in crate::parser) fn get_interface_constant_modifier_group(
        &self,
        input: Vec<(Span, TokenKind, Span)>,
    ) -> ParseResult<ConstantModifierGroup> {
        let modifiers = input
            .iter()
            .map(|(start, token, end)| match token {
                TokenKind::Public => Ok(ConstantModifier::Public {
                    start: *start,
                    end: *end,
                }),
                TokenKind::Final => Ok(ConstantModifier::Final {
                    start: *start,
                    end: *end,
                }),
                _ => Err(ParseError::CannotUseModifierOnInterfaceConstant(
                    token.to_string(),
                    *start,
                )),
            })
            .collect::<ParseResult<Vec<ConstantModifier>>>()?;

        Ok(ConstantModifierGroup { modifiers })
    }

    pub(in crate::parser) fn modifiers(
        &self,
        state: &mut State,
    ) -> ParseResult<Vec<(Span, TokenKind, Span)>> {
        let mut collected: Vec<(Span, TokenKind, Span)> = vec![];
        let mut collected_tokens: Vec<TokenKind> = vec![];

        while let TokenKind::Private
        | TokenKind::Protected
        | TokenKind::Public
        | TokenKind::Final
        | TokenKind::Abstract
        | TokenKind::Static
        | TokenKind::Var
        | TokenKind::Readonly = state.current.kind.clone()
        {
            if collected_tokens.contains(&state.current.kind) {
                return Err(ParseError::MultipleModifiers(
                    state.current.kind.to_string(),
                    state.current.span,
                ));
            }

            // garud against multiple visibility modifiers, we don't care where these modifiers are used.
            match state.current.kind {
                TokenKind::Private
                    if collected_tokens.contains(&TokenKind::Protected)
                        || collected_tokens.contains(&TokenKind::Public) =>
                {
                    return Err(ParseError::MultipleVisibilityModifiers(state.current.span));
                }
                TokenKind::Protected
                    if collected_tokens.contains(&TokenKind::Private)
                        || collected_tokens.contains(&TokenKind::Public) =>
                {
                    return Err(ParseError::MultipleVisibilityModifiers(state.current.span));
                }
                TokenKind::Public
                    if collected_tokens.contains(&TokenKind::Private)
                        || collected_tokens.contains(&TokenKind::Protected) =>
                {
                    return Err(ParseError::MultipleVisibilityModifiers(state.current.span));
                }
                _ => {}
            };

            let start = state.current.span;
            let end = state.peek.span;
            collected.push((start, state.current.kind.clone(), end));
            collected_tokens.push(state.current.kind.clone());

            state.next();
        }

        Ok(collected)
    }
}
