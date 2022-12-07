use crate::lexer::token::TokenKind;
use crate::parser::ast::Statement;
use crate::parser::error::ParseResult;
use crate::parser::internal::precedences::Precedence;
use crate::parser::internal::utils;
use crate::parser::state::State;
use crate::parser::Parser;

impl Parser {
    pub(in crate::parser) fn foreach_loop(&self, state: &mut State) -> ParseResult<Statement> {
        utils::skip(state, TokenKind::Foreach)?;

        utils::skip_left_parenthesis(state)?;

        let expr = self.expression(state, Precedence::Lowest)?;

        utils::skip(state, TokenKind::As)?;

        let mut by_ref = state.current.kind == TokenKind::Ampersand;
        if by_ref {
            state.next();
        }

        let mut key_var = None;
        let mut value_var = self.expression(state, Precedence::Lowest)?;

        if state.current.kind == TokenKind::DoubleArrow {
            state.next();

            key_var = Some(value_var.clone());

            by_ref = state.current.kind == TokenKind::Ampersand;
            if by_ref {
                state.next();
            }

            value_var = self.expression(state, Precedence::Lowest)?;
        }

        utils::skip_right_parenthesis(state)?;

        let end_token = if state.current.kind == TokenKind::Colon {
            utils::colon(state)?;
            TokenKind::EndForeach
        } else {
            utils::skip_left_brace(state)?;
            TokenKind::RightBrace
        };

        let body = self.body(state, &end_token)?;

        if end_token == TokenKind::EndForeach {
            utils::skip(state, TokenKind::EndForeach)?;
            utils::skip_semicolon(state)?;
        } else {
            utils::skip_right_brace(state)?;
        }

        Ok(Statement::Foreach {
            expr,
            by_ref,
            key_var,
            value_var,
            body,
        })
    }

    pub(in crate::parser) fn for_loop(&self, state: &mut State) -> ParseResult<Statement> {
        utils::skip(state, TokenKind::For)?;

        utils::skip_left_parenthesis(state)?;

        let mut init = Vec::new();
        loop {
            if state.current.kind == TokenKind::SemiColon {
                break;
            }

            init.push(self.expression(state, Precedence::Lowest)?);

            if state.current.kind == TokenKind::Comma {
                state.next();
            } else {
                break;
            }
        }

        utils::skip_semicolon(state)?;

        let mut condition = Vec::new();
        loop {
            if state.current.kind == TokenKind::SemiColon {
                break;
            }

            condition.push(self.expression(state, Precedence::Lowest)?);

            if state.current.kind == TokenKind::Comma {
                state.next();
            } else {
                break;
            }
        }
        utils::skip_semicolon(state)?;

        let mut r#loop = Vec::new();
        loop {
            if state.current.kind == TokenKind::RightParen {
                break;
            }

            r#loop.push(self.expression(state, Precedence::Lowest)?);

            if state.current.kind == TokenKind::Comma {
                state.next();
            } else {
                break;
            }
        }

        utils::skip_right_parenthesis(state)?;

        let end_token = if state.current.kind == TokenKind::Colon {
            utils::colon(state)?;
            TokenKind::EndFor
        } else {
            utils::skip_left_brace(state)?;
            TokenKind::RightBrace
        };

        let then = self.body(state, &end_token)?;

        if end_token == TokenKind::EndFor {
            utils::skip(state, TokenKind::EndFor)?;
            utils::skip_semicolon(state)?;
        } else {
            utils::skip_right_brace(state)?;
        };

        Ok(Statement::For {
            init,
            condition,
            r#loop,
            then,
        })
    }

    pub(in crate::parser) fn do_loop(&self, state: &mut State) -> ParseResult<Statement> {
        utils::skip(state, TokenKind::Do)?;

        utils::skip_left_brace(state)?;
        let body = self.body(state, &TokenKind::RightBrace)?;
        utils::skip_right_brace(state)?;

        utils::skip(state, TokenKind::While)?;

        utils::skip_left_parenthesis(state)?;
        let condition = self.expression(state, Precedence::Lowest)?;
        utils::skip_right_parenthesis(state)?;
        utils::skip_semicolon(state)?;

        Ok(Statement::DoWhile { condition, body })
    }

    pub(in crate::parser) fn while_loop(&self, state: &mut State) -> ParseResult<Statement> {
        utils::skip(state, TokenKind::While)?;

        utils::skip_left_parenthesis(state)?;

        let condition = self.expression(state, Precedence::Lowest)?;

        utils::skip_right_parenthesis(state)?;

        let body = if state.current.kind == TokenKind::SemiColon {
            utils::skip_semicolon(state)?;
            vec![]
        } else {
            let end_token = if state.current.kind == TokenKind::Colon {
                utils::colon(state)?;
                TokenKind::EndWhile
            } else {
                utils::skip_left_brace(state)?;
                TokenKind::RightBrace
            };

            let body = self.body(state, &end_token)?;

            if end_token == TokenKind::RightBrace {
                utils::skip_right_brace(state)?;
            } else {
                utils::skip(state, TokenKind::EndWhile)?;
                utils::skip_semicolon(state)?;
            }

            body
        };

        Ok(Statement::While { condition, body })
    }

    pub(in crate::parser) fn continue_statement(
        &self,
        state: &mut State,
    ) -> ParseResult<Statement> {
        utils::skip(state, TokenKind::Continue)?;

        let mut num = None;
        if state.current.kind != TokenKind::SemiColon {
            num = Some(self.expression(state, Precedence::Lowest)?);
        }

        utils::skip_semicolon(state)?;

        Ok(Statement::Continue { num })
    }

    pub(in crate::parser) fn break_statement(&self, state: &mut State) -> ParseResult<Statement> {
        utils::skip(state, TokenKind::Break)?;

        let mut num = None;
        if state.current.kind != TokenKind::SemiColon {
            num = Some(self.expression(state, Precedence::Lowest)?);
        }

        utils::skip_semicolon(state)?;

        Ok(Statement::Break { num })
    }
}
