use crate::lexer::token::TokenKind;
use crate::parser::error::ParseResult;
use crate::parser::state::State;
use crate::parser::Parser;

use crate::expect_token;

impl Parser {
    pub(in crate::parser) fn semi(&self, state: &mut State) -> ParseResult<()> {
        if state.current.kind != TokenKind::CloseTag {
            expect_token!([TokenKind::SemiColon => Ok(())], state, "`;`")
        } else {
            Ok(())
        }
    }

    pub(in crate::parser) fn lbrace(&self, state: &mut State) -> ParseResult<()> {
        state.skip_comments();
        expect_token!([TokenKind::LeftBrace], state, "`{`");
        state.skip_comments();
        Ok(())
    }

    pub(in crate::parser) fn rbrace(&self, state: &mut State) -> ParseResult<()> {
        state.skip_comments();
        expect_token!([TokenKind::RightBrace], state, "`}`");
        state.skip_comments();
        Ok(())
    }

    pub(in crate::parser) fn lparen(&self, state: &mut State) -> ParseResult<()> {
        expect_token!([TokenKind::LeftParen => Ok(())], state, "`(`")
    }

    pub(in crate::parser) fn rparen(&self, state: &mut State) -> ParseResult<()> {
        expect_token!([TokenKind::RightParen => Ok(())], state, "`)`")
    }

    pub(in crate::parser) fn rbracket(&self, state: &mut State) -> ParseResult<()> {
        expect_token!([TokenKind::RightBracket => Ok(())], state, "`]`")
    }

    pub(in crate::parser) fn colon(&self, state: &mut State) -> ParseResult<()> {
        expect_token!([TokenKind::Colon => Ok(())], state, "`:`")
    }
}
