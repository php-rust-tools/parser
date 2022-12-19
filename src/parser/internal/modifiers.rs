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

#[inline(always)]
pub fn class_group(input: Vec<(Span, TokenKind)>) -> ParseResult<ClassModifierGroup> {
    let mut has_final = false;
    let mut has_abstract = false;

    let modifiers = input
        .iter()
        .map(|(span, token)| match token {
            TokenKind::Readonly => Ok(ClassModifier::Readonly(*span)),
            TokenKind::Final => {
                has_final = true;
                if has_abstract {
                    Err(ParseError::FinalModifierOnAbstractClassMember(*span))
                } else {
                    Ok(ClassModifier::Final(*span))
                }
            }
            TokenKind::Abstract => {
                has_abstract = true;
                if has_final {
                    Err(ParseError::FinalModifierOnAbstractClassMember(*span))
                } else {
                    Ok(ClassModifier::Abstract(*span))
                }
            }
            _ => Err(ParseError::CannotUseModifierOnClass(
                token.to_string(),
                *span,
            )),
        })
        .collect::<ParseResult<Vec<ClassModifier>>>()?;

    Ok(ClassModifierGroup { modifiers })
}

#[inline(always)]
pub fn method_group(input: Vec<(Span, TokenKind)>) -> ParseResult<MethodModifierGroup> {
    let mut has_final = false;
    let mut has_abstract = false;

    let modifiers = input
        .iter()
        .map(|(span, token)| match token {
            TokenKind::Final => {
                has_final = true;
                if has_abstract {
                    Err(ParseError::FinalModifierOnAbstractClassMember(*span))
                } else {
                    Ok(MethodModifier::Final(*span))
                }
            }
            TokenKind::Abstract => {
                has_abstract = true;
                if has_final {
                    Err(ParseError::FinalModifierOnAbstractClassMember(*span))
                } else {
                    Ok(MethodModifier::Abstract(*span))
                }
            }
            TokenKind::Private => Ok(MethodModifier::Private(*span)),
            TokenKind::Protected => Ok(MethodModifier::Protected(*span)),
            TokenKind::Public => Ok(MethodModifier::Public(*span)),
            TokenKind::Static => Ok(MethodModifier::Static(*span)),
            _ => Err(ParseError::CannotUseModifierOnClassMethod(
                token.to_string(),
                *span,
            )),
        })
        .collect::<ParseResult<Vec<MethodModifier>>>()?;

    Ok(MethodModifierGroup { modifiers })
}

#[inline(always)]
pub fn interface_method_group(input: Vec<(Span, TokenKind)>) -> ParseResult<MethodModifierGroup> {
    let modifiers = input
        .iter()
        .map(|(span, token)| match token {
            TokenKind::Public => Ok(MethodModifier::Public(*span)),
            TokenKind::Static => Ok(MethodModifier::Static(*span)),
            _ => Err(ParseError::CannotUseModifierOnInterfaceMethod(
                token.to_string(),
                *span,
            )),
        })
        .collect::<ParseResult<Vec<MethodModifier>>>()?;

    Ok(MethodModifierGroup { modifiers })
}

pub fn enum_method_group(input: Vec<(Span, TokenKind)>) -> ParseResult<MethodModifierGroup> {
    let modifiers = input
        .iter()
        .map(|(span, token)| match token {
            TokenKind::Final => Ok(MethodModifier::Final(*span)),
            TokenKind::Private => Ok(MethodModifier::Private(*span)),
            TokenKind::Protected => Ok(MethodModifier::Protected(*span)),
            TokenKind::Public => Ok(MethodModifier::Public(*span)),
            TokenKind::Static => Ok(MethodModifier::Static(*span)),
            _ => Err(ParseError::CannotUseModifierOnEnumMethod(
                token.to_string(),
                *span,
            )),
        })
        .collect::<ParseResult<Vec<MethodModifier>>>()?;

    Ok(MethodModifierGroup { modifiers })
}

#[inline(always)]
pub fn property_group(input: Vec<(Span, TokenKind)>) -> ParseResult<PropertyModifierGroup> {
    let modifiers = input
        .iter()
        .map(|(span, token)| match token {
            TokenKind::Readonly => Ok(PropertyModifier::Readonly(*span)),
            TokenKind::Static => Ok(PropertyModifier::Static(*span)),
            TokenKind::Public => Ok(PropertyModifier::Public(*span)),
            TokenKind::Protected => Ok(PropertyModifier::Protected(*span)),
            TokenKind::Private => Ok(PropertyModifier::Private(*span)),
            _ => Err(ParseError::CannotUseModifierOnProperty(
                token.to_string(),
                *span,
            )),
        })
        .collect::<ParseResult<Vec<PropertyModifier>>>()?;

    Ok(PropertyModifierGroup { modifiers })
}

#[inline(always)]
pub fn promoted_property_group(
    input: Vec<(Span, TokenKind)>,
) -> ParseResult<PromotedPropertyModifierGroup> {
    let modifiers = input
        .iter()
        .map(|(span, token)| match token {
            TokenKind::Readonly => Ok(PromotedPropertyModifier::Readonly(*span)),
            TokenKind::Private => Ok(PromotedPropertyModifier::Private(*span)),
            TokenKind::Protected => Ok(PromotedPropertyModifier::Protected(*span)),
            TokenKind::Public => Ok(PromotedPropertyModifier::Public(*span)),
            _ => Err(ParseError::CannotUseModifierOnPromotedProperty(
                token.to_string(),
                *span,
            )),
        })
        .collect::<ParseResult<Vec<PromotedPropertyModifier>>>()?;

    Ok(PromotedPropertyModifierGroup { modifiers })
}

pub fn constant_group(input: Vec<(Span, TokenKind)>) -> ParseResult<ConstantModifierGroup> {
    let mut has_final = false;
    let mut has_private = false;

    let modifiers = input
        .iter()
        .map(|(span, token)| match token {
            TokenKind::Protected => Ok(ConstantModifier::Protected(*span)),
            TokenKind::Public => Ok(ConstantModifier::Public(*span)),
            TokenKind::Private => {
                has_private = true;
                if has_final {
                    Err(ParseError::FinalModifierOnPrivateConstant(*span))
                } else {
                    Ok(ConstantModifier::Private(*span))
                }
            }
            TokenKind::Final => {
                has_final = true;
                if has_private {
                    Err(ParseError::FinalModifierOnPrivateConstant(*span))
                } else {
                    Ok(ConstantModifier::Final(*span))
                }
            }
            _ => Err(ParseError::CannotUseModifierOnConstant(
                token.to_string(),
                *span,
            )),
        })
        .collect::<ParseResult<Vec<ConstantModifier>>>()?;

    Ok(ConstantModifierGroup { modifiers })
}

pub fn interface_constant_group(
    input: Vec<(Span, TokenKind)>,
) -> ParseResult<ConstantModifierGroup> {
    let modifiers = input
        .iter()
        .map(|(span, token)| match token {
            TokenKind::Public => Ok(ConstantModifier::Public(*span)),
            TokenKind::Final => Ok(ConstantModifier::Final(*span)),
            _ => Err(ParseError::CannotUseModifierOnInterfaceConstant(
                token.to_string(),
                *span,
            )),
        })
        .collect::<ParseResult<Vec<ConstantModifier>>>()?;

    Ok(ConstantModifierGroup { modifiers })
}

pub fn collect(state: &mut State) -> ParseResult<Vec<(Span, TokenKind)>> {
    let mut collected: Vec<(Span, TokenKind)> = vec![];
    let mut collected_tokens: Vec<TokenKind> = vec![];

    let collectable_tokens = vec![
        TokenKind::Private,
        TokenKind::Protected,
        TokenKind::Public,
        TokenKind::Final,
        TokenKind::Abstract,
        TokenKind::Static,
        TokenKind::Readonly,
    ];

    let mut current = state.stream.current().clone();
    let mut current_kind = current.kind;
    let mut current_span = current.span;

    while collectable_tokens.contains(&current_kind) {
        if collected_tokens.contains(&current_kind) {
            return Err(ParseError::MultipleModifiers(
                current_kind.to_string(),
                current_span,
            ));
        }

        // garud against multiple visibility modifiers, we don't care where these modifiers are used.
        match current_kind {
            TokenKind::Private
                if collected_tokens.contains(&TokenKind::Protected)
                    || collected_tokens.contains(&TokenKind::Public) =>
            {
                return Err(ParseError::MultipleVisibilityModifiers(current_span));
            }
            TokenKind::Protected
                if collected_tokens.contains(&TokenKind::Private)
                    || collected_tokens.contains(&TokenKind::Public) =>
            {
                return Err(ParseError::MultipleVisibilityModifiers(current_span));
            }
            TokenKind::Public
                if collected_tokens.contains(&TokenKind::Private)
                    || collected_tokens.contains(&TokenKind::Protected) =>
            {
                return Err(ParseError::MultipleVisibilityModifiers(current_span));
            }
            _ => {}
        };

        collected_tokens.push(current_kind.clone());
        collected.push((current_span, current_kind));

        state.stream.next();

        current = state.stream.current().clone();
        current_kind = current.kind;
        current_span = current.span;
    }

    Ok(collected)
}
