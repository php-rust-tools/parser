use trunk_lexer::TokenKind;

use crate::Block;

use super::{Parser, ParseResult};

impl Parser {
    pub fn block(&mut self, until: &TokenKind) -> ParseResult<Block> {
        let mut block = Block::new();

        while ! self.is_eof() && &self.current.kind != until {
            block.push(self.statement()?);
        }

        Ok(block)
    }
}