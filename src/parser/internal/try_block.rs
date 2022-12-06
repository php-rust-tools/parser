use crate::lexer::token::TokenKind;
use crate::parser::ast::try_block::CatchBlock;
use crate::parser::ast::try_block::CatchType;
use crate::parser::ast::try_block::FinallyBlock;
use crate::parser::ast::try_block::TryBlock;
use crate::parser::ast::Statement;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::internal::precedences::Precedence;
use crate::parser::state::State;
use crate::parser::Parser;

impl Parser {
    pub(in crate::parser) fn try_block(&self, state: &mut State) -> ParseResult<Statement> {
        let start = state.current.span;

        state.next();
        self.lbrace(state)?;

        let body = self.block(state, &TokenKind::RightBrace)?;

        self.rbrace(state)?;

        let mut catches = Vec::new();
        loop {
            if state.current.kind != TokenKind::Catch {
                break;
            }

            let catch_start = state.current.span;

            state.next();
            self.lparen(state)?;

            let types = self.get_try_block_catch_type(state)?;
            let var = if state.current.kind == TokenKind::RightParen {
                None
            } else {
                // TODO(azjezz): this is a variable, no an expression?
                Some(self.expression(state, Precedence::Lowest)?)
            };

            self.rparen(state)?;
            self.lbrace(state)?;

            let catch_body = self.block(state, &TokenKind::RightBrace)?;

            self.rbrace(state)?;

            let catch_end = state.current.span;

            catches.push(CatchBlock {
                start: catch_start,
                end: catch_end,
                types,
                var,
                body: catch_body,
            })
        }

        let mut finally = None;
        if state.current.kind == TokenKind::Finally {
            let finally_start = state.current.span;
            state.next();
            self.lbrace(state)?;

            let finally_body = self.block(state, &TokenKind::RightBrace)?;

            self.rbrace(state)?;
            let finally_end = state.current.span;

            finally = Some(FinallyBlock {
                start: finally_start,
                end: finally_end,
                body: finally_body,
            });
        }

        if catches.is_empty() && finally.is_none() {
            return Err(ParseError::TryWithoutCatchOrFinally(start));
        }

        let end = state.current.span;

        Ok(Statement::Try(TryBlock {
            start,
            end,
            body,
            catches,
            finally,
        }))
    }

    fn get_try_block_catch_type(&self, state: &mut State) -> ParseResult<CatchType> {
        let id = self.full_name(state)?;

        if state.current.kind == TokenKind::Pipe {
            state.next();

            let mut types = vec![id];

            while !state.is_eof() {
                let id = self.full_name(state)?;
                types.push(id);

                if state.current.kind != TokenKind::Pipe {
                    break;
                }

                state.next();
            }

            return Ok(CatchType::Union(types));
        }

        Ok(CatchType::Identifier(id))
    }
}
