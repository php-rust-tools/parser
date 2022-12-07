use crate::lexer::token::TokenKind;
use crate::parser::ast::Statement;
use crate::parser::error::ParseResult;
use crate::parser::internal::precedences::Precedence;
use crate::parser::state::State;
use crate::parser::Parser;

impl Parser {
    pub(in crate::parser) fn foreach_loop(&self, state: &mut State) -> ParseResult<Statement> {
        self.skip(state, TokenKind::Foreach)?;

        self.left_parenthesis(state)?;

        let expr = self.expression(state, Precedence::Lowest)?;

        self.skip(state, TokenKind::As)?;

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

        self.right_parenthesis(state)?;

        let end_token = if state.current.kind == TokenKind::Colon {
            self.colon(state)?;
            TokenKind::EndForeach
        } else {
            self.left_brace(state)?;
            TokenKind::RightBrace
        };

        let body = self.block(state, &end_token)?;

        if end_token == TokenKind::EndForeach {
            self.skip(state, TokenKind::EndForeach)?;
            self.semicolon(state)?;
        } else {
            self.right_brace(state)?;
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
        self.skip(state, TokenKind::For)?;

        self.left_parenthesis(state)?;

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

        self.semicolon(state)?;

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
        self.semicolon(state)?;

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

        self.right_parenthesis(state)?;

        let end_token = if state.current.kind == TokenKind::Colon {
            self.colon(state)?;
            TokenKind::EndFor
        } else {
            self.left_brace(state)?;
            TokenKind::RightBrace
        };

        let then = self.block(state, &end_token)?;

        if end_token == TokenKind::EndFor {
            self.skip(state, TokenKind::EndFor)?;
            self.semicolon(state)?;
        } else {
            self.right_brace(state)?;
        };

        Ok(Statement::For {
            init,
            condition,
            r#loop,
            then,
        })
    }

    pub(in crate::parser) fn do_loop(&self, state: &mut State) -> ParseResult<Statement> {
        self.skip(state, TokenKind::Do)?;

        self.left_brace(state)?;
        let body = self.block(state, &TokenKind::RightBrace)?;
        self.right_brace(state)?;

        self.skip(state, TokenKind::While)?;

        self.left_parenthesis(state)?;
        let condition = self.expression(state, Precedence::Lowest)?;
        self.right_parenthesis(state)?;
        self.semicolon(state)?;

        Ok(Statement::DoWhile { condition, body })
    }

    pub(in crate::parser) fn while_loop(&self, state: &mut State) -> ParseResult<Statement> {
        self.skip(state, TokenKind::While)?;

        self.left_parenthesis(state)?;

        let condition = self.expression(state, Precedence::Lowest)?;

        self.right_parenthesis(state)?;

        let body = if state.current.kind == TokenKind::SemiColon {
            self.semicolon(state)?;
            vec![]
        } else {
            let end_token = if state.current.kind == TokenKind::Colon {
                self.colon(state)?;
                TokenKind::EndWhile
            } else {
                self.left_brace(state)?;
                TokenKind::RightBrace
            };

            let body = self.block(state, &end_token)?;

            if end_token == TokenKind::RightBrace {
                self.right_brace(state)?;
            } else {
                self.skip(state, TokenKind::EndWhile)?;
                self.semicolon(state)?;
            }

            body
        };

        Ok(Statement::While { condition, body })
    }

    pub(in crate::parser) fn continue_statement(
        &self,
        state: &mut State,
    ) -> ParseResult<Statement> {
        self.skip(state, TokenKind::Continue)?;

        let mut num = None;
        if state.current.kind != TokenKind::SemiColon {
            num = Some(self.expression(state, Precedence::Lowest)?);
        }

        self.semicolon(state)?;

        Ok(Statement::Continue { num })
    }

    pub(in crate::parser) fn break_statement(&self, state: &mut State) -> ParseResult<Statement> {
        self.skip(state, TokenKind::Break)?;

        let mut num = None;
        if state.current.kind != TokenKind::SemiColon {
            num = Some(self.expression(state, Precedence::Lowest)?);
        }

        self.semicolon(state)?;

        Ok(Statement::Break { num })
    }
}
