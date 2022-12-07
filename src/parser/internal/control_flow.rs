use crate::expected_token_err;
use crate::lexer::token::TokenKind;
use crate::parser::ast::Block;
use crate::parser::ast::Case;
use crate::parser::ast::ElseIf;
use crate::parser::ast::Statement;
use crate::parser::error::ParseResult;
use crate::parser::internal::precedences::Precedence;
use crate::parser::state::State;
use crate::parser::Parser;

impl Parser {
    pub(in crate::parser) fn switch_statement(&self, state: &mut State) -> ParseResult<Statement> {
        self.skip(state, TokenKind::Switch)?;

        self.left_parenthesis(state)?;

        let condition = self.expression(state, Precedence::Lowest)?;

        self.right_parenthesis(state)?;

        let end_token = if state.current.kind == TokenKind::Colon {
            self.colon(state)?;
            TokenKind::EndSwitch
        } else {
            self.left_brace(state)?;
            TokenKind::RightBrace
        };

        let mut cases = Vec::new();
        while state.current.kind != end_token {
            match state.current.kind {
                TokenKind::Case => {
                    state.next();

                    let condition = self.expression(state, Precedence::Lowest)?;

                    self.skip_any_of(state, &[TokenKind::Colon, TokenKind::SemiColon])?;

                    let mut body = Block::new();

                    while state.current.kind != TokenKind::Case
                        && state.current.kind != TokenKind::Default
                        && state.current.kind != TokenKind::RightBrace
                    {
                        body.push(self.statement(state)?);
                        state.skip_comments();
                    }

                    cases.push(Case {
                        condition: Some(condition),
                        body,
                    });
                }
                TokenKind::Default => {
                    state.next();

                    self.skip_any_of(state, &[TokenKind::Colon, TokenKind::SemiColon])?;

                    let mut body = Block::new();

                    while state.current.kind != TokenKind::Case
                        && state.current.kind != TokenKind::Default
                        && state.current.kind != TokenKind::RightBrace
                    {
                        body.push(self.statement(state)?);
                    }

                    cases.push(Case {
                        condition: None,
                        body,
                    });
                }
                _ => {
                    return expected_token_err!(["`case`", "`default`"], state);
                }
            }
        }

        if end_token == TokenKind::EndSwitch {
            self.skip(state, TokenKind::EndSwitch)?;
            self.semicolon(state)?;
        } else {
            self.right_brace(state)?;
        }

        Ok(Statement::Switch { condition, cases })
    }

    pub(in crate::parser) fn if_statement(&self, state: &mut State) -> ParseResult<Statement> {
        self.skip(state, TokenKind::If)?;

        self.left_parenthesis(state)?;

        let condition = self.expression(state, Precedence::Lowest)?;

        self.right_parenthesis(state)?;

        // FIXME: Tidy up duplication and make the intent a bit clearer.
        match state.current.kind {
            TokenKind::Colon => {
                self.colon(state)?;

                let mut then = vec![];
                while !matches!(
                    state.current.kind,
                    TokenKind::ElseIf | TokenKind::Else | TokenKind::EndIf
                ) {
                    if let TokenKind::OpenTag(_) = state.current.kind {
                        state.next();
                        continue;
                    }

                    then.push(self.statement(state)?);
                }

                let mut else_ifs = vec![];
                loop {
                    if state.current.kind != TokenKind::ElseIf {
                        break;
                    }

                    state.next();

                    self.left_parenthesis(state)?;
                    let condition = self.expression(state, Precedence::Lowest)?;
                    self.right_parenthesis(state)?;

                    self.colon(state)?;

                    let mut body = vec![];
                    while !matches!(
                        state.current.kind,
                        TokenKind::ElseIf | TokenKind::Else | TokenKind::EndIf
                    ) {
                        if let TokenKind::OpenTag(_) = state.current.kind {
                            state.next();
                            continue;
                        }

                        body.push(self.statement(state)?);
                    }

                    else_ifs.push(ElseIf { condition, body });
                }

                let mut r#else = None;
                if state.current.kind == TokenKind::Else {
                    state.next();
                    self.colon(state)?;

                    let body = self.block(state, &TokenKind::EndIf)?;

                    r#else = Some(body);
                }

                self.skip(state, TokenKind::EndIf)?;

                self.semicolon(state)?;

                Ok(Statement::If {
                    condition,
                    then,
                    else_ifs,
                    r#else,
                })
            }
            _ => {
                let then = if state.current.kind == TokenKind::LeftBrace {
                    self.left_brace(state)?;
                    let then = self.block(state, &TokenKind::RightBrace)?;
                    self.right_brace(state)?;
                    then
                } else {
                    vec![self.statement(state)?]
                };

                let mut else_ifs: Vec<ElseIf> = Vec::new();
                loop {
                    if state.current.kind == TokenKind::ElseIf {
                        state.next();

                        self.left_parenthesis(state)?;

                        let condition = self.expression(state, Precedence::Lowest)?;

                        self.right_parenthesis(state)?;

                        self.left_brace(state)?;

                        let body = self.block(state, &TokenKind::RightBrace)?;

                        self.right_brace(state)?;

                        else_ifs.push(ElseIf { condition, body });
                    } else {
                        break;
                    }
                }

                if state.current.kind != TokenKind::Else {
                    return Ok(Statement::If {
                        condition,
                        then,
                        else_ifs,
                        r#else: None,
                    });
                }

                self.skip(state, TokenKind::Else)?;

                let r#else;
                if state.current.kind == TokenKind::LeftBrace {
                    self.left_brace(state)?;

                    r#else = self.block(state, &TokenKind::RightBrace)?;

                    self.right_brace(state)?;
                } else {
                    r#else = vec![self.statement(state)?];
                }

                Ok(Statement::If {
                    condition,
                    then,
                    else_ifs,
                    r#else: Some(r#else),
                })
            }
        }
    }
}
