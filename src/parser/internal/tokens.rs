use crate::lexer::token::Span;
use crate::lexer::token::TokenKind;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::state::State;
use crate::parser::Parser;

impl Parser {
    pub(in crate::parser) fn skip(&self, state: &mut State, kind: TokenKind) -> ParseResult<Span> {
        state.skip_comments();

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

    pub(in crate::parser) fn skip_any_of(
        &self,
        state: &mut State,
        kinds: &[TokenKind],
    ) -> ParseResult<Span> {
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
}
