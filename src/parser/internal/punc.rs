use crate::lexer::token::TokenKind;
use crate::parser::error::ParseResult;
use crate::parser::state::State;
use crate::parser::Parser;

use crate::expect_token;

impl Parser {
    pub(in crate::parser) fn semi(&self, state: &mut State) -> ParseResult<()> {
        expect_token!([TokenKind::SemiColon], state, "`;`");
        Ok(())
    }

    pub(in crate::parser) fn lbrace(&self, state: &mut State) -> ParseResult<()> {
        expect_token!([TokenKind::LeftBrace], state, "`{`");
        Ok(())
    }

    pub(in crate::parser) fn rbrace(&self, state: &mut State) -> ParseResult<()> {
        expect_token!([TokenKind::RightBrace], state, "`}`");
        Ok(())
    }

    pub(in crate::parser) fn lparen(&self, state: &mut State) -> ParseResult<()> {
        expect_token!([TokenKind::LeftParen], state, "`(`");
        Ok(())
    }

    pub(in crate::parser) fn rparen(&self, state: &mut State) -> ParseResult<()> {
        expect_token!([TokenKind::RightParen], state, "`)`");
        Ok(())
    }

    pub(in crate::parser) fn rbracket(&self, state: &mut State) -> ParseResult<()> {
        expect_token!([TokenKind::RightBracket], state, "`]`");
        Ok(())
    }

    pub(in crate::parser) fn optional_comma(&self, state: &mut State) -> ParseResult<()> {
        if state.current.kind == TokenKind::Comma {
            expect_token!([TokenKind::Comma], state, "`,`");
        }

        Ok(())
    }

    pub(in crate::parser) fn colon(&self, state: &mut State) -> ParseResult<()> {
        expect_token!([TokenKind::Colon], state, "`:`");

        Ok(())
    }
}
