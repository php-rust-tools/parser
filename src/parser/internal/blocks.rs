use crate::lexer::token::TokenKind;
use crate::parser::ast::Block;
use crate::parser::error::ParseResult;
use crate::parser::state::State;
use crate::parser::Parser;

impl Parser {
    pub(in crate::parser) fn block(
        &self,
        state: &mut State,
        until: &TokenKind,
    ) -> ParseResult<Block> {
        state.skip_comments();

        let mut block = Block::new();

        while !state.is_eof() && &state.current.kind != until {
            block.push(self.statement(state)?);
            state.skip_comments();
        }

        Ok(block)
    }
}
