use trunk_lexer::TokenKind;

use crate::{Parser, ParseError};

use super::ParseResult;

impl Parser {
    pub(crate) fn semi(&mut self) -> ParseResult<()> {
        Ok(expect!(self, TokenKind::SemiColon, "expected semi colon"))
    }

    pub(crate) fn lbrace(&mut self) -> ParseResult<()> {
        Ok(expect!(self, TokenKind::LeftBrace, "expected {"))
    }

    pub(crate) fn rbrace(&mut self) -> ParseResult<()> {
        Ok(expect!(self, TokenKind::RightBrace, "expected }"))
    }

    pub(crate) fn lparen(&mut self) -> ParseResult<()> {
        Ok(expect!(self, TokenKind::LeftParen, "expected ("))
    }

    pub(crate) fn rparen(&mut self) -> ParseResult<()> {
        Ok(expect!(self, TokenKind::RightParen, "expected )"))
    }
}