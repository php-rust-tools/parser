use trunk_lexer::TokenKind;

use crate::{ParseError, Parser};

use super::ParseResult;

impl Parser {
    pub(crate) fn semi(&mut self) -> ParseResult<()> {
        expect!(self, TokenKind::SemiColon, (), "expected semi colon");
        Ok(())
    }

    pub(crate) fn lbrace(&mut self) -> ParseResult<()> {
        expect!(self, TokenKind::LeftBrace, (), "expected {");
        Ok(())
    }

    pub(crate) fn rbrace(&mut self) -> ParseResult<()> {
        expect!(self, TokenKind::RightBrace, (), "expected }");
        Ok(())
    }

    pub(crate) fn lparen(&mut self) -> ParseResult<()> {
        expect!(self, TokenKind::LeftParen, (), "expected (");
        Ok(())
    }

    pub(crate) fn rparen(&mut self) -> ParseResult<()> {
        expect!(self, TokenKind::RightParen, (), "expected )");
        Ok(())
    }

    pub(crate) fn rbracket(&mut self) -> ParseResult<()> {
        expect!(self, TokenKind::RightBracket, (), "expected ]");
        Ok(())
    }

    pub(crate) fn optional_comma(&mut self) -> ParseResult<()> {
        if self.current.kind == TokenKind::Comma {
            expect!(self, TokenKind::Comma, (), "expected ,");
        }

        Ok(())
    }

    pub(crate) fn colon(&mut self) -> ParseResult<()> {
        expect!(self, TokenKind::Colon, "expected :");

        Ok(())
    }
}
