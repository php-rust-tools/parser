use crate::lexer::token::Span;
use crate::lexer::token::TokenKind;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::state::State;

pub fn skip_semicolon(state: &mut State) -> ParseResult<Span> {
    state.skip_comments();

    let end = state.current.span;

    if state.current.kind == TokenKind::SemiColon {
        state.next();
        state.skip_comments();
    } else if state.current.kind != TokenKind::CloseTag {
        let found = if state.current.kind == TokenKind::Eof {
            None
        } else {
            Some(state.current.kind.to_string())
        };

        return Err(ParseError::ExpectedToken(
            vec!["`;`".to_string()],
            found,
            end,
        ));
    } else {
        state.next();
    }

    Ok(end)
}

pub fn skip_left_brace(state: &mut State) -> ParseResult<Span> {
    let span = skip(state, TokenKind::LeftBrace)?;
    // A closing PHP tag is valid after a left brace, since
    // that typically indicates the start of a block (control structures).
    if state.current.kind == TokenKind::CloseTag {
        state.next();
    }

    Ok(span)
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
    let span = skip(state, TokenKind::Colon)?;
    // A closing PHP tag is valid after a colon, since
    // that typically indicates the start of a block (control structures).
    if state.current.kind == TokenKind::CloseTag {
        state.next();
    }
    Ok(span)
}

pub fn skip(state: &mut State, kind: TokenKind) -> ParseResult<Span> {
    if state.current.kind == kind {
        let end = state.current.span;

        state.next();
        state.skip_comments();

        Ok(end)
    } else {
        let found = if state.current.kind == TokenKind::Eof {
            None
        } else {
            Some(state.current.kind.to_string())
        };

        Err(ParseError::ExpectedToken(
            vec![format!("`{}`", kind)],
            found,
            state.current.span,
        ))
    }
}

pub fn skip_any_of(state: &mut State, kinds: &[TokenKind]) -> ParseResult<Span> {
    state.skip_comments();

    if kinds.contains(&state.current.kind) {
        let end = state.current.span;

        state.next();
        state.skip_comments();

        Ok(end)
    } else {
        let found = if state.current.kind == TokenKind::Eof {
            None
        } else {
            Some(state.current.kind.to_string())
        };

        Err(ParseError::ExpectedToken(
            kinds.iter().map(|kind| format!("`{}`", kind)).collect(),
            found,
            state.current.span,
        ))
    }
}
