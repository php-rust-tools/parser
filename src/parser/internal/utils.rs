use crate::lexer::token::Span;
use crate::lexer::token::TokenKind;
use crate::parser::ast::utils::Braced;
use crate::parser::ast::utils::Bracketed;
use crate::parser::ast::utils::CommaSeparated;
use crate::parser::ast::utils::Parenthesized;
use crate::parser::ast::utils::SemicolonTerminated;
use crate::parser::ast::Ending;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::state::State;

pub fn skip_ending(state: &mut State) -> ParseResult<Ending> {
    let current = state.stream.current();

    if current.kind == TokenKind::CloseTag {
        state.stream.next();

        Ok(Ending::CloseTag(current.span))
    } else if current.kind == TokenKind::SemiColon {
        state.stream.next();

        Ok(Ending::Semicolon(current.span))
    } else {
        let found = if state.stream.current().kind == TokenKind::Eof {
            None
        } else {
            Some(state.stream.current().kind.to_string())
        };

        Err(ParseError::ExpectedToken(
            vec!["`;`".to_string()],
            found,
            current.span,
        ))
    }
}

pub fn skip_semicolon(state: &mut State) -> ParseResult<Span> {
    let current = state.stream.current();

    if current.kind == TokenKind::SemiColon {
        state.stream.next();

        Ok(current.span)
    } else {
        let found = if state.stream.current().kind == TokenKind::Eof {
            None
        } else {
            Some(state.stream.current().kind.to_string())
        };

        Err(ParseError::ExpectedToken(
            vec!["`;`".to_string()],
            found,
            current.span,
        ))
    }
}

pub fn skip_left_brace(state: &mut State) -> ParseResult<Span> {
    skip(state, TokenKind::LeftBrace)
}

pub fn skip_right_brace(state: &mut State) -> ParseResult<Span> {
    skip(state, TokenKind::RightBrace)
}

pub fn skip_left_parenthesis(state: &mut State) -> ParseResult<Span> {
    skip(state, TokenKind::LeftParen)
}

pub fn skip_right_parenthesis(state: &mut State) -> ParseResult<Span> {
    skip(state, TokenKind::RightParen)
}

pub fn skip_left_bracket(state: &mut State) -> ParseResult<Span> {
    skip(state, TokenKind::LeftBracket)
}

pub fn skip_right_bracket(state: &mut State) -> ParseResult<Span> {
    skip(state, TokenKind::RightBracket)
}

pub fn skip_double_arrow(state: &mut State) -> ParseResult<Span> {
    skip(state, TokenKind::DoubleArrow)
}

pub fn skip_double_colon(state: &mut State) -> ParseResult<Span> {
    skip(state, TokenKind::DoubleColon)
}

pub fn skip_colon(state: &mut State) -> ParseResult<Span> {
    skip(state, TokenKind::Colon)
}

pub fn skip(state: &mut State, kind: TokenKind) -> ParseResult<Span> {
    let current = state.stream.current().clone();

    if current.kind == kind {
        let end = current.span;

        state.stream.next();

        Ok(end)
    } else {
        let found = if current.kind == TokenKind::Eof {
            None
        } else {
            Some(current.kind.to_string())
        };

        Err(ParseError::ExpectedToken(
            vec![format!("`{}`", kind)],
            found,
            current.span,
        ))
    }
}

pub fn skip_any_of(state: &mut State, kinds: &[TokenKind]) -> ParseResult<Span> {
    let current = state.stream.current().clone();

    if kinds.contains(&current.kind) {
        let end = current.span;

        state.stream.next();

        Ok(end)
    } else {
        let found = if current.kind == TokenKind::Eof {
            None
        } else {
            Some(current.kind.to_string())
        };

        Err(ParseError::ExpectedToken(
            kinds.iter().map(|kind| format!("`{}`", kind)).collect(),
            found,
            current.span,
        ))
    }
}

/// Parse an item that is surrounded by parentheses.
///
/// This function will skip the left parenthesis, call the given function,
/// and then skip the right parenthesis.
pub fn parenthesized<T>(
    state: &mut State,
    func: &(dyn Fn(&mut State) -> ParseResult<T>),
) -> ParseResult<Parenthesized<T>> {
    let left_parenthesis = skip_left_parenthesis(state)?;
    let inner = func(state)?;
    let right_parenthesis = skip_right_parenthesis(state)?;

    Ok(Parenthesized {
        left_parenthesis,
        inner,
        right_parenthesis,
    })
}

/// Parse an item that is surrounded by braces.
///
/// This function will skip the left brace, call the given function,
/// and then skip the right brace.
pub fn braced<T>(
    state: &mut State,
    func: &(dyn Fn(&mut State) -> ParseResult<T>),
) -> ParseResult<Braced<T>> {
    let left_brace = skip_left_brace(state)?;
    let inner = func(state)?;
    let right_brace = skip_right_brace(state)?;

    Ok(Braced {
        left_brace,
        inner,
        right_brace,
    })
}

/// Parse an item that is surrounded by brackets.
///
/// This function will skip the left bracket, call the given function,
/// and then skip the right bracket.
pub fn bracketed<T>(
    state: &mut State,
    func: &(dyn Fn(&mut State) -> ParseResult<T>),
) -> ParseResult<Bracketed<T>> {
    let left_bracket = skip_left_bracket(state)?;
    let inner = func(state)?;
    let right_bracket = skip_right_bracket(state)?;

    Ok(Bracketed {
        left_bracket,
        inner,
        right_bracket,
    })
}

pub fn semicolon_terminated<T>(
    state: &mut State,
    func: &(dyn Fn(&mut State) -> ParseResult<T>),
) -> ParseResult<SemicolonTerminated<T>> {
    let inner = func(state)?;
    let semicolon = skip_semicolon(state)?;

    Ok(SemicolonTerminated { inner, semicolon })
}

/// Parse a comma-separated list of items, allowing a trailing comma.
pub fn comma_separated<T>(
    state: &mut State,
    func: &(dyn Fn(&mut State) -> ParseResult<T>),
    until: TokenKind,
) -> ParseResult<CommaSeparated<T>> {
    let mut inner: Vec<T> = vec![];
    let mut commas: Vec<Span> = vec![];
    let mut current = state.stream.current();

    while current.kind != until {
        inner.push(func(state)?);

        current = state.stream.current();
        if current.kind != TokenKind::Comma {
            break;
        }

        commas.push(current.span);

        state.stream.next();

        current = state.stream.current();
    }

    Ok(CommaSeparated { inner, commas })
}

/// Parse a comma-separated list of items, not allowing trailing commas.
pub fn comma_separated_no_trailing<T>(
    state: &mut State,
    func: &(dyn Fn(&mut State) -> ParseResult<T>),
    until: TokenKind,
) -> ParseResult<CommaSeparated<T>> {
    let mut inner: Vec<T> = vec![];
    let mut commas: Vec<Span> = vec![];
    let mut current = state.stream.current();

    while current.kind != until {
        inner.push(func(state)?);

        current = state.stream.current();
        if current.kind != TokenKind::Comma {
            break;
        }

        // If the next token is the until token, we don't want to consume the comma.
        // This ensures that trailing commas are not allowed.
        if state.stream.peek().kind == until {
            break;
        }

        commas.push(current.span);

        state.stream.next();

        current = state.stream.current();
    }

    Ok(CommaSeparated { inner, commas })
}

/// Parse a comma-separated list of items, requiring at least one item, and not allowing trailing commas.
pub fn at_least_one_comma_separated_no_trailing<T>(
    state: &mut State,
    func: &(dyn Fn(&mut State) -> ParseResult<T>),
) -> ParseResult<CommaSeparated<T>> {
    let mut inner: Vec<T> = vec![];
    let mut commas: Vec<Span> = vec![];

    loop {
        inner.push(func(state)?);

        let current = state.stream.current();
        if current.kind != TokenKind::Comma {
            break;
        }

        commas.push(current.span);

        state.stream.next();
    }

    Ok(CommaSeparated { inner, commas })
}
