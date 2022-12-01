use crate::lexer::token::TokenKind;
use crate::parser::error::ParseResult;
use crate::parser::Parser;

use crate::expect_token;

impl Parser {
    pub(in crate::parser) fn semi(&mut self) -> ParseResult<()> {
        expect_token!([TokenKind::SemiColon], self, "`;`");
        Ok(())
    }

    pub(in crate::parser) fn lbrace(&mut self) -> ParseResult<()> {
        expect_token!([TokenKind::LeftBrace], self, "`{`");
        Ok(())
    }

    pub(in crate::parser) fn rbrace(&mut self) -> ParseResult<()> {
        expect_token!([TokenKind::RightBrace], self, "`}`");
        Ok(())
    }

    pub(in crate::parser) fn lparen(&mut self) -> ParseResult<()> {
        expect_token!([TokenKind::LeftParen], self, "`(`");
        Ok(())
    }

    pub(in crate::parser) fn rparen(&mut self) -> ParseResult<()> {
        expect_token!([TokenKind::RightParen], self, "`)`");
        Ok(())
    }

    pub(in crate::parser) fn rbracket(&mut self) -> ParseResult<()> {
        expect_token!([TokenKind::RightBracket], self, "`]`");
        Ok(())
    }

    pub(in crate::parser) fn optional_comma(&mut self) -> ParseResult<()> {
        if self.current.kind == TokenKind::Comma {
            expect_token!([TokenKind::Comma], self, "`,`");
        }

        Ok(())
    }

    pub(in crate::parser) fn colon(&mut self) -> ParseResult<()> {
        expect_token!([TokenKind::Colon], self, "`:`");

        Ok(())
    }
}
