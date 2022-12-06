use crate::lexer::token::Span;
use crate::lexer::token::TokenKind;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::state::State;
use crate::parser::Parser;

impl Parser {
    pub(in crate::parser) fn semicolon(&self, state: &mut State) -> ParseResult<Span> {
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
                state.current.span,
            ));
        }

        Ok(end)
    }

    pub(in crate::parser) fn left_brace(&self, state: &mut State) -> ParseResult<Span> {
        state.skip_comments();

        let end = state.current.span;

        if state.current.kind == TokenKind::LeftBrace {
            state.next();
            state.skip_comments();
        } else {
            let found = if state.current.kind == TokenKind::Eof {
                None
            } else {
                Some(state.current.kind.to_string())
            };

            return Err(ParseError::ExpectedToken(
                vec!["`{`".to_string()],
                found,
                state.current.span,
            ));
        }

        Ok(end)
    }

    pub(in crate::parser) fn right_brace(&self, state: &mut State) -> ParseResult<Span> {
        state.skip_comments();

        let end = state.current.span;

        if state.current.kind == TokenKind::RightBrace {
            state.next();
            state.skip_comments();
        } else {
            let found = if state.current.kind == TokenKind::Eof {
                None
            } else {
                Some(state.current.kind.to_string())
            };

            return Err(ParseError::ExpectedToken(
                vec!["`}`".to_string()],
                found,
                state.current.span,
            ));
        }

        Ok(end)
    }

    pub(in crate::parser) fn left_parenthesis(&self, state: &mut State) -> ParseResult<Span> {
        state.skip_comments();

        let end = state.current.span;

        if state.current.kind == TokenKind::LeftParen {
            state.next();
            state.skip_comments();
        } else {
            let found = if state.current.kind == TokenKind::Eof {
                None
            } else {
                Some(state.current.kind.to_string())
            };

            return Err(ParseError::ExpectedToken(
                vec!["`(`".to_string()],
                found,
                state.current.span,
            ));
        }

        Ok(end)
    }

    pub(in crate::parser) fn right_parenthesis(&self, state: &mut State) -> ParseResult<Span> {
        state.skip_comments();

        let end = state.current.span;

        if state.current.kind == TokenKind::RightParen {
            state.next();
            state.skip_comments();
        } else {
            let found = if state.current.kind == TokenKind::Eof {
                None
            } else {
                Some(state.current.kind.to_string())
            };

            return Err(ParseError::ExpectedToken(
                vec!["`)`".to_string()],
                found,
                state.current.span,
            ));
        }

        Ok(end)
    }

    pub(in crate::parser) fn right_bracket(&self, state: &mut State) -> ParseResult<Span> {
        state.skip_comments();

        let end = state.current.span;

        if state.current.kind == TokenKind::RightBracket {
            state.next();
            state.skip_comments();
        } else {
            let found = if state.current.kind == TokenKind::Eof {
                None
            } else {
                Some(state.current.kind.to_string())
            };

            return Err(ParseError::ExpectedToken(
                vec!["`]`".to_string()],
                found,
                state.current.span,
            ));
        }

        Ok(end)
    }

    pub(in crate::parser) fn colon(&self, state: &mut State) -> ParseResult<Span> {
        state.skip_comments();

        let end = state.current.span;

        if state.current.kind == TokenKind::Colon {
            state.next();
            state.skip_comments();
        } else {
            let found = if state.current.kind == TokenKind::Eof {
                None
            } else {
                Some(state.current.kind.to_string())
            };

            return Err(ParseError::ExpectedToken(
                vec!["`:`".to_string()],
                found,
                state.current.span,
            ));
        }

        Ok(end)
    }
}
