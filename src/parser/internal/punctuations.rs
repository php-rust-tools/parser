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
        } else {
            state.next();
        }

        Ok(end)
    }

    pub(in crate::parser) fn left_brace(&self, state: &mut State) -> ParseResult<Span> {
        let span = self.skip(state, TokenKind::LeftBrace)?;
        // A closing PHP tag is valid after a left brace, since
        // that typically indicates the start of a block (control structures).
        if state.current.kind == TokenKind::CloseTag {
            state.next();
        }
        Ok(span)
    }

    pub(in crate::parser) fn right_brace(&self, state: &mut State) -> ParseResult<Span> {
        self.skip(state, TokenKind::RightBrace)
    }

    pub(in crate::parser) fn left_parenthesis(&self, state: &mut State) -> ParseResult<Span> {
        self.skip(state, TokenKind::LeftParen)
    }

    pub(in crate::parser) fn right_parenthesis(&self, state: &mut State) -> ParseResult<Span> {
        self.skip(state, TokenKind::RightParen)
    }

    pub(in crate::parser) fn right_bracket(&self, state: &mut State) -> ParseResult<Span> {
        self.skip(state, TokenKind::RightBracket)
    }

    pub(in crate::parser) fn double_arrow(&self, state: &mut State) -> ParseResult<Span> {
        self.skip(state, TokenKind::DoubleArrow)
    }

    pub(in crate::parser) fn colon(&self, state: &mut State) -> ParseResult<Span> {
        let span = self.skip(state, TokenKind::Colon)?;
        // A closing PHP tag is valid after a colon, since
        // that typically indicates the start of a block (control structures).
        if state.current.kind == TokenKind::CloseTag {
            state.next();
        }
        Ok(span)
    }
}
