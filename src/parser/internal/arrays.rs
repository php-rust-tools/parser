use crate::lexer::token::TokenKind;
use crate::parser::ast::ArrayItem;
use crate::parser::ast::Expression;
use crate::parser::ast::ListItem;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::expressions;
use crate::parser::internal::utils;
use crate::parser::state::State;

pub fn list_expression(state: &mut State) -> ParseResult<Expression> {
    utils::skip(state, TokenKind::List)?;
    utils::skip_left_parenthesis(state)?;

    let mut items = Vec::new();
    let mut has_atleast_one_key = false;

    while state.current.kind != TokenKind::RightParen {
        if state.current.kind == TokenKind::Comma {
            items.push(ListItem {
                key: None,
                value: Expression::Empty,
            });
            state.next();
            continue;
        }

        let mut key = None;

        if state.current.kind == TokenKind::Ellipsis {
            return Err(ParseError::IllegalSpreadOperator(state.current.span));
        }

        if state.current.kind == TokenKind::Ampersand {
            return Err(ParseError::CannotAssignReferenceToNonReferencableValue(
                state.current.span,
            ));
        }

        let mut value = expressions::lowest_precedence(state)?;

        if state.current.kind == TokenKind::DoubleArrow {
            if !has_atleast_one_key && !items.is_empty() {
                return Err(ParseError::CannotMixKeyedAndUnkeyedEntries(
                    state.current.span,
                ));
            }

            state.next();

            key = Some(value);

            if state.current.kind == TokenKind::Ellipsis {
                return Err(ParseError::IllegalSpreadOperator(state.current.span));
            }

            if state.current.kind == TokenKind::Ampersand {
                return Err(ParseError::CannotAssignReferenceToNonReferencableValue(
                    state.current.span,
                ));
            }

            has_atleast_one_key = true;
            value = expressions::lowest_precedence(state)?;
        } else if has_atleast_one_key {
            return Err(ParseError::CannotMixKeyedAndUnkeyedEntries(
                state.current.span,
            ));
        }

        items.push(ListItem { key, value });

        state.skip_comments();
        if state.current.kind == TokenKind::Comma {
            state.next();
            state.skip_comments();
        } else {
            break;
        }
    }

    utils::skip_right_parenthesis(state)?;

    Ok(Expression::List { items })
}

pub fn array_expression(state: &mut State) -> ParseResult<Expression> {
    utils::skip(state, TokenKind::LeftBracket)?;

    let mut items = Vec::new();
    state.skip_comments();

    while state.current.kind != TokenKind::RightBracket {
        // TODO: return an error here instead of
        // an empty array element
        // see: https://3v4l.org/uLTVA
        if state.current.kind == TokenKind::Comma {
            items.push(ArrayItem {
                key: None,
                value: Expression::Empty,
                unpack: false,
                by_ref: false,
            });
            state.next();
            continue;
        }

        items.push(array_pair(state)?);

        state.skip_comments();

        if state.current.kind != TokenKind::Comma {
            break;
        }

        state.next();
        state.skip_comments();
    }

    state.skip_comments();

    utils::skip_right_bracket(state)?;

    Ok(Expression::Array { items })
}

pub fn legacy_array_expression(state: &mut State) -> ParseResult<Expression> {
    utils::skip(state, TokenKind::Array)?;
    utils::skip_left_parenthesis(state)?;

    let mut items = vec![];

    while state.current.kind != TokenKind::RightParen {
        let mut key = None;
        let unpack = if state.current.kind == TokenKind::Ellipsis {
            state.next();
            true
        } else {
            false
        };

        let (mut by_ref, amper_span) = if state.current.kind == TokenKind::Ampersand {
            let span = state.current.span;
            state.next();
            (true, span)
        } else {
            (false, (0, 0))
        };

        let mut value = expressions::lowest_precedence(state)?;

        // TODO: return error for `[...$a => $b]`.
        if state.current.kind == TokenKind::DoubleArrow {
            state.next();

            if by_ref {
                return Err(ParseError::UnexpectedToken(
                    TokenKind::Ampersand.to_string(),
                    amper_span,
                ));
            }

            key = Some(value);

            by_ref = if state.current.kind == TokenKind::Ampersand {
                state.next();
                true
            } else {
                false
            };

            value = expressions::lowest_precedence(state)?;
        }

        items.push(ArrayItem {
            key,
            value,
            unpack,
            by_ref,
        });

        if state.current.kind == TokenKind::Comma {
            state.next();
        } else {
            break;
        }

        state.skip_comments();
    }

    utils::skip_right_parenthesis(state)?;

    Ok(Expression::Array { items })
}

fn array_pair(state: &mut State) -> ParseResult<ArrayItem> {
    let mut key = None;
    let unpack = if state.current.kind == TokenKind::Ellipsis {
        state.next();
        true
    } else {
        false
    };

    let (mut by_ref, amper_span) = if state.current.kind == TokenKind::Ampersand {
        let span = state.current.span;
        state.next();
        (true, span)
    } else {
        (false, (0, 0))
    };

    let mut value = expressions::lowest_precedence(state)?;
    if state.current.kind == TokenKind::DoubleArrow {
        state.next();

        if by_ref {
            return Err(ParseError::UnexpectedToken(
                TokenKind::Ampersand.to_string(),
                amper_span,
            ));
        }

        key = Some(value);
        by_ref = if state.current.kind == TokenKind::Ampersand {
            state.next();
            true
        } else {
            false
        };
        value = expressions::lowest_precedence(state)?;
    }

    Ok(ArrayItem {
        key,
        value,
        unpack,
        by_ref,
    })
}
