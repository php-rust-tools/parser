use crate::lexer::token::TokenKind;
use crate::parser::ast::Block;
use crate::parser::ast::Statement;
use crate::parser::error::ParseResult;
use crate::parser::internal::utils;
use crate::parser::state::State;
use crate::parser::Parser;

impl Parser {
    pub(in crate::parser) fn block_statement(&self, state: &mut State) -> ParseResult<Statement> {
        utils::skip_left_brace(state)?;

        let body = self.body(state, &TokenKind::RightBrace)?;

        utils::skip_right_brace(state)?;

        Ok(Statement::Block { body })
    }

    pub(in crate::parser) fn body(
        &self,
        state: &mut State,
        until: &TokenKind,
    ) -> ParseResult<Block> {
        state.skip_comments();

        let mut block = Block::new();

        while !state.is_eof() && &state.current.kind != until {
            if let TokenKind::OpenTag(_) = state.current.kind {
                state.next();
                continue;
            }

            block.push(self.statement(state)?);
            state.skip_comments();
        }

        Ok(block)
    }
}
