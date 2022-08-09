use trunk_lexer::TokenKind;

use crate::Block;

use super::{Parser, ParseResult};

impl Parser {
    pub(crate) fn block(&mut self, until: &TokenKind) -> ParseResult<Block> {
        self.skip_comments();
        
        let mut block = Block::new();

        while ! self.is_eof() && &self.current.kind != until {
            block.push(self.statement()?);
        }

        Ok(block)
    }
}