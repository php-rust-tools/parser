use crate::lexer::token::TokenKind;
use crate::parser::ast::Block;
use crate::parser::error::ParseResult;
use crate::parser::Parser;

impl Parser {
    pub(in crate::parser) fn block(&mut self, until: &TokenKind) -> ParseResult<Block> {
        self.skip_comments();

        let mut block = Block::new();

        while !self.is_eof() && &self.current.kind != until {
            block.push(self.statement()?);
            self.skip_comments();
        }

        Ok(block)
    }
}
