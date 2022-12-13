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

    while state.stream.current().kind != TokenKind::RightParen {
        if state.stream.current().kind == TokenKind::Comma {
            items.push(ListItem {
                key: None,
                value: Expression::Empty,
            });

            state.stream.next();

            continue;
        }

        let mut key = None;

        if state.stream.current().kind == TokenKind::Ellipsis {
            return Err(ParseError::IllegalSpreadOperator(
                state.stream.current().span,
            ));
        }

        if state.stream.current().kind == TokenKind::Ampersand {
            return Err(ParseError::CannotAssignReferenceToNonReferencableValue(
                state.stream.current().span,
            ));
        }

        let span = state.stream.current().span;
        let mut value = expressions::lowest_precedence(state)?;

        if state.stream.current().kind == TokenKind::DoubleArrow {
            if !has_atleast_one_key && !items.is_empty() {
                return Err(ParseError::CannotMixKeyedAndUnkeyedEntries(span));
            }

            state.stream.next();

            key = Some(value);

            if state.stream.current().kind == TokenKind::Ellipsis {
                return Err(ParseError::IllegalSpreadOperator(
                    state.stream.current().span,
                ));
            }

            if state.stream.current().kind == TokenKind::Ampersand {
                return Err(ParseError::CannotAssignReferenceToNonReferencableValue(
                    state.stream.current().span,
                ));
            }

            has_atleast_one_key = true;
            value = expressions::lowest_precedence(state)?;
        } else if has_atleast_one_key {
            return Err(ParseError::CannotMixKeyedAndUnkeyedEntries(span));
        }

        items.push(ListItem { key, value });

        if state.stream.current().kind == TokenKind::Comma {
            state.stream.next();
        } else {
            break;
        }
    }

    utils::skip_right_parenthesis(state)?;

    Ok(Expression::List { items })
}

pub fn short_array_expression(state: &mut State) -> ParseResult<Expression> {
    let start = utils::skip(state, TokenKind::LeftBracket)?;

    let mut items = Vec::new();

    while state.stream.current().kind != TokenKind::RightBracket {
        // TODO: return an error here instead of
        // an empty array element
        // see: https://3v4l.org/uLTVA
        if state.stream.current().kind == TokenKind::Comma {
            items.push(ArrayItem {
                key: None,
                value: Expression::Empty,
                unpack: false,
                by_ref: false,
            });
            state.stream.next();

            continue;
        }

        items.push(array_pair(state)?);

        if state.stream.current().kind != TokenKind::Comma {
            break;
        }

        state.stream.next();
    }

    let end = utils::skip_right_bracket(state)?;

    Ok(Expression::ShortArray { start, items, end })
}

pub fn array_expression(state: &mut State) -> ParseResult<Expression> {
    let span = utils::skip(state, TokenKind::Array)?;
    let start = utils::skip_left_parenthesis(state)?;

    let mut items = vec![];

    while state.stream.current().kind != TokenKind::RightParen {
        let mut key = None;
        let unpack = if state.stream.current().kind == TokenKind::Ellipsis {
            state.stream.next();
            true
        } else {
            false
        };

        let (mut by_ref, amper_span) = if state.stream.current().kind == TokenKind::Ampersand {
            let span = state.stream.current().span;
            state.stream.next();
            (true, span)
        } else {
            (false, (0, 0))
        };

        let mut value = expressions::lowest_precedence(state)?;

        // TODO: return error for `[...$a => $b]`.
        if state.stream.current().kind == TokenKind::DoubleArrow {
            state.stream.next();

            if by_ref {
                return Err(ParseError::UnexpectedToken(
                    TokenKind::Ampersand.to_string(),
                    amper_span,
                ));
            }

            key = Some(value);

            by_ref = if state.stream.current().kind == TokenKind::Ampersand {
                state.stream.next();
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

        if state.stream.current().kind == TokenKind::Comma {
            state.stream.next();
        } else {
            break;
        }
    }

    let end = utils::skip_right_parenthesis(state)?;

    Ok(Expression::Array {
        span,
        start,
        items,
        end,
    })
}

fn array_pair(state: &mut State) -> ParseResult<ArrayItem> {
    let mut key = None;
    let unpack = if state.stream.current().kind == TokenKind::Ellipsis {
        state.stream.next();
        true
    } else {
        false
    };

    let (mut by_ref, amper_span) = if state.stream.current().kind == TokenKind::Ampersand {
        let span = state.stream.current().span;
        state.stream.next();
        (true, span)
    } else {
        (false, (0, 0))
    };

    let mut value = expressions::lowest_precedence(state)?;
    if state.stream.current().kind == TokenKind::DoubleArrow {
        state.stream.next();

        if by_ref {
            return Err(ParseError::UnexpectedToken(
                TokenKind::Ampersand.to_string(),
                amper_span,
            ));
        }

        key = Some(value);
        by_ref = if state.stream.current().kind == TokenKind::Ampersand {
            state.stream.next();
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
