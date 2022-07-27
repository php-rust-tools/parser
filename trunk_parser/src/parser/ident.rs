use crate::Parser;
use trunk_lexer::TokenKind;
use super::{ParseResult, ParseError};

impl Parser {
    pub(crate) fn ident(&mut self) -> ParseResult<String> {
        Ok(expect!(self, TokenKind::Identifier(i), i, "expected identifier"))
    }
}