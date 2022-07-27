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
}