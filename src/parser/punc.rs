use crate::TokenKind;

use crate::expect_token;
use crate::Parser;

use super::ParseResult;

impl Parser {
    pub(crate) fn semi(&mut self) -> ParseResult<()> {
        expect_token!([TokenKind::SemiColon], self, "`;`");
        Ok(())
    }

    pub(crate) fn lbrace(&mut self) -> ParseResult<()> {
        expect_token!([TokenKind::LeftBrace], self, "`{`");
        Ok(())
    }

    pub(crate) fn rbrace(&mut self) -> ParseResult<()> {
        expect_token!([TokenKind::RightBrace], self, "`}`");
        Ok(())
    }

    pub(crate) fn lparen(&mut self) -> ParseResult<()> {
        expect_token!([TokenKind::LeftParen], self, "`(`");
        Ok(())
    }

    pub(crate) fn rparen(&mut self) -> ParseResult<()> {
        expect_token!([TokenKind::RightParen], self, "`)`");
        Ok(())
    }

    pub(crate) fn rbracket(&mut self) -> ParseResult<()> {
        expect_token!([TokenKind::RightBracket], self, "`]`");
        Ok(())
    }

    pub(crate) fn optional_comma(&mut self) -> ParseResult<()> {
        if self.current.kind == TokenKind::Comma {
            expect_token!([TokenKind::Comma], self, "`,`");
        }

        Ok(())
    }

    pub(crate) fn colon(&mut self) -> ParseResult<()> {
        expect_token!([TokenKind::Colon], self, "`:`");

        Ok(())
    }
}
