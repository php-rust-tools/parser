use crate::lexer::token::TokenKind;
use crate::parser::ast::templates::Template;
use crate::parser::ast::templates::TemplateGroup;
use crate::parser::ast::templates::TemplateTypeConstraint;
use crate::parser::ast::templates::TemplateVariance;
use crate::parser::error::ParseResult;
use crate::parser::internal::data_type;
use crate::parser::internal::identifiers;
use crate::parser::internal::utils;
use crate::parser::state::State;

pub fn parse(state: &mut State) -> ParseResult<TemplateGroup> {
    let start = state.current.span;
    utils::skip(state, TokenKind::LessThan)?;

    let mut templates = vec![];
    while state.current.kind != TokenKind::GreaterThan {
        // +|-|n T as|super|= T
        let variance = match &state.current.kind {
            TokenKind::Plus => {
                let span = state.current.span;
                state.next();

                TemplateVariance::Covariance(span)
            }
            TokenKind::Minus => {
                let span = state.current.span;
                state.next();

                TemplateVariance::Contravariance(span)
            }
            _ => TemplateVariance::Invaraint,
        };

        let name = identifiers::ident_maybe_reserved(state)?;

        let constraint = match &state.current.kind {
            TokenKind::Equals => {
                let span = state.current.span;
                state.next();
                let ty = data_type::data_type(state)?;
                TemplateTypeConstraint::Equal(span, ty)
            }
            TokenKind::As => {
                let span = state.current.span;
                state.next();
                let ty = data_type::data_type(state)?;
                TemplateTypeConstraint::SubType(span, ty)
            }
            TokenKind::Super => {
                let span = state.current.span;
                state.next();
                let ty = data_type::data_type(state)?;
                TemplateTypeConstraint::SuperType(span, ty)
            }
            _ => TemplateTypeConstraint::None,
        };

        templates.push(Template {
            name,
            variance,
            constraint,
        });

        state.skip_comments();

        if state.current.kind == TokenKind::Comma {
            state.next();
        } else {
            break;
        }
    }

    let end = state.current.span;
    utils::skip(state, TokenKind::GreaterThan)?;

    Ok(TemplateGroup {
        start,
        end,
        members: templates,
    })
}
