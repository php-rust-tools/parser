use crate::expected_token_err;
use crate::lexer::token::TokenKind;
use crate::parser::ast::Block;
use crate::parser::ast::Case;
use crate::parser::ast::ElseIf;
use crate::parser::ast::Statement;
use crate::parser::error::ParseResult;
use crate::parser::internal::precedences::Precedence;
use crate::parser::internal::utils;
use crate::parser::state::State;
use crate::parser::Parser;

impl Parser {
    pub(in crate::parser) fn switch_statement(&self, state: &mut State) -> ParseResult<Statement> {
        utils::skip(state, TokenKind::Switch)?;

        utils::skip_left_parenthesis(state)?;

        let condition = self.expression(state, Precedence::Lowest)?;

        utils::skip_right_parenthesis(state)?;

        let end_token = if state.current.kind == TokenKind::Colon {
            utils::colon(state)?;
            TokenKind::EndSwitch
        } else {
            utils::skip_left_brace(state)?;
            TokenKind::RightBrace
        };

        let mut cases = Vec::new();
        while state.current.kind != end_token {
            match state.current.kind {
                TokenKind::Case => {
                    state.next();

                    let condition = self.expression(state, Precedence::Lowest)?;

                    utils::skip_any_of(state, &[TokenKind::Colon, TokenKind::SemiColon])?;

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

                    utils::skip_any_of(state, &[TokenKind::Colon, TokenKind::SemiColon])?;

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
            utils::skip(state, TokenKind::EndSwitch)?;
            utils::skip_semicolon(state)?;
        } else {
            utils::skip_right_brace(state)?;
        }

        Ok(Statement::Switch { condition, cases })
    }

    pub(in crate::parser) fn if_statement(&self, state: &mut State) -> ParseResult<Statement> {
        utils::skip(state, TokenKind::If)?;

        utils::skip_left_parenthesis(state)?;

        let condition = self.expression(state, Precedence::Lowest)?;

        utils::skip_right_parenthesis(state)?;

        // FIXME: Tidy up duplication and make the intent a bit clearer.
        match state.current.kind {
            TokenKind::Colon => {
                utils::colon(state)?;

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

                    utils::skip_left_parenthesis(state)?;
                    let condition = self.expression(state, Precedence::Lowest)?;
                    utils::skip_right_parenthesis(state)?;

                    utils::colon(state)?;

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
                    utils::colon(state)?;

                    let body = self.body(state, &TokenKind::EndIf)?;

                    r#else = Some(body);
                }

                utils::skip(state, TokenKind::EndIf)?;

                utils::skip_semicolon(state)?;

                Ok(Statement::If {
                    condition,
                    then,
                    else_ifs,
                    r#else,
                })
            }
            _ => {
                let then = if state.current.kind == TokenKind::LeftBrace {
                    utils::skip_left_brace(state)?;
                    let then = self.body(state, &TokenKind::RightBrace)?;
                    utils::skip_right_brace(state)?;
                    then
                } else {
                    vec![self.statement(state)?]
                };

                let mut else_ifs: Vec<ElseIf> = Vec::new();
                loop {
                    if state.current.kind == TokenKind::ElseIf {
                        state.next();

                        utils::skip_left_parenthesis(state)?;

                        let condition = self.expression(state, Precedence::Lowest)?;

                        utils::skip_right_parenthesis(state)?;

                        utils::skip_left_brace(state)?;

                        let body = self.body(state, &TokenKind::RightBrace)?;

                        utils::skip_right_brace(state)?;

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

                utils::skip(state, TokenKind::Else)?;

                let r#else;
                if state.current.kind == TokenKind::LeftBrace {
                    utils::skip_left_brace(state)?;

                    r#else = self.body(state, &TokenKind::RightBrace)?;

                    utils::skip_right_brace(state)?;
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
