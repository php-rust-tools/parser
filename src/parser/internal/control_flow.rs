use crate::expected_token_err;
use crate::lexer::token::OpenTagKind;
use crate::lexer::token::TokenKind;
use crate::parser;
use crate::parser::ast::Block;
use crate::parser::ast::Case;
use crate::parser::ast::DefaultMatchArm;
use crate::parser::ast::ElseIf;
use crate::parser::ast::Expression;
use crate::parser::ast::MatchArm;
use crate::parser::ast::Statement;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::expressions;
use crate::parser::internal::blocks;
use crate::parser::internal::utils;
use crate::parser::state::State;

pub fn match_expression(state: &mut State) -> ParseResult<Expression> {
    utils::skip(state, TokenKind::Match)?;

    utils::skip_left_parenthesis(state)?;

    let condition = Box::new(expressions::lowest_precedence(state)?);

    utils::skip_right_parenthesis(state)?;
    utils::skip_left_brace(state)?;

    let mut default = None;
    let mut arms = Vec::new();
    while state.stream.current().kind != TokenKind::RightBrace {
        if state.stream.current().kind == TokenKind::Default {
            if default.is_some() {
                return Err(ParseError::MatchExpressionWithMultipleDefaultArms(
                    state.stream.current().span,
                ));
            }

            state.stream.next();

            // match conditions can have an extra comma at the end, including `default`.
            if state.stream.current().kind == TokenKind::Comma {
                state.stream.next();
            }

            utils::skip_double_arrow(state)?;

            let body = expressions::lowest_precedence(state)?;

            default = Some(Box::new(DefaultMatchArm { body }));
        } else {
            let mut conditions = Vec::new();
            while state.stream.current().kind != TokenKind::DoubleArrow {
                conditions.push(expressions::lowest_precedence(state)?);

                if state.stream.current().kind == TokenKind::Comma {
                    state.stream.next();
                } else {
                    break;
                }
            }

            if !conditions.is_empty() {
                utils::skip_double_arrow(state)?;
            } else {
                break;
            }

            let body = expressions::lowest_precedence(state)?;

            arms.push(MatchArm { conditions, body });
        }

        if state.stream.current().kind == TokenKind::Comma {
            state.stream.next();
        } else {
            break;
        }
    }

    utils::skip_right_brace(state)?;

    Ok(Expression::Match {
        condition,
        default,
        arms,
    })
}

pub fn switch_statement(state: &mut State) -> ParseResult<Statement> {
    utils::skip(state, TokenKind::Switch)?;

    utils::skip_left_parenthesis(state)?;

    let condition = expressions::lowest_precedence(state)?;

    utils::skip_right_parenthesis(state)?;

    let end_token = if state.stream.current().kind == TokenKind::Colon {
        utils::skip_colon(state)?;
        TokenKind::EndSwitch
    } else {
        utils::skip_left_brace(state)?;
        TokenKind::RightBrace
    };

    let mut cases = Vec::new();
    while state.stream.current().kind != end_token {
        match state.stream.current().kind {
            TokenKind::Case => {
                state.stream.next();

                let condition = expressions::lowest_precedence(state)?;

                utils::skip_any_of(state, &[TokenKind::Colon, TokenKind::SemiColon])?;
                utils::skip_close_tag(state)?;

                let mut body = Block::new();

                while state.stream.current().kind != TokenKind::Case
                    && state.stream.current().kind != TokenKind::Default
                    && state.stream.current().kind != TokenKind::RightBrace
                    && state.stream.current().kind != end_token
                {
                    body.push(parser::statement(state)?);
                }

                cases.push(Case {
                    condition: Some(condition),
                    body,
                });
            }
            TokenKind::Default => {
                state.stream.next();

                utils::skip_any_of(state, &[TokenKind::Colon, TokenKind::SemiColon])?;
                utils::skip_close_tag(state)?;

                let mut body = Block::new();

                while state.stream.current().kind != TokenKind::Case
                    && state.stream.current().kind != TokenKind::Default
                    && state.stream.current().kind != end_token
                {
                    body.push(parser::statement(state)?);
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

pub fn if_statement(state: &mut State) -> ParseResult<Statement> {
    utils::skip(state, TokenKind::If)?;

    utils::skip_left_parenthesis(state)?;

    let condition = expressions::lowest_precedence(state)?;

    utils::skip_right_parenthesis(state)?;

    // FIXME: Tidy up duplication and make the intent a bit clearer.
    match state.stream.current().kind {
        TokenKind::Colon => {
            utils::skip_colon(state)?;

            let mut then = vec![];
            while !matches!(
                state.stream.current().kind,
                TokenKind::ElseIf | TokenKind::Else | TokenKind::EndIf
            ) {
                if let TokenKind::OpenTag(OpenTagKind::Full) = state.stream.current().kind {
                    state.stream.next();
                    continue;
                }

                then.push(parser::statement(state)?);
            }

            let mut else_ifs = vec![];
            loop {
                if state.stream.current().kind != TokenKind::ElseIf {
                    break;
                }

                state.stream.next();

                utils::skip_left_parenthesis(state)?;
                let condition = expressions::lowest_precedence(state)?;
                utils::skip_right_parenthesis(state)?;

                utils::skip_colon(state)?;

                let mut body = vec![];
                while !matches!(
                    state.stream.current().kind,
                    TokenKind::ElseIf | TokenKind::Else | TokenKind::EndIf
                ) {
                    if let TokenKind::OpenTag(OpenTagKind::Full) = state.stream.current().kind {
                        state.stream.next();
                        continue;
                    }

                    body.push(parser::statement(state)?);
                }

                else_ifs.push(ElseIf { condition, body });
            }

            let mut r#else = None;
            if state.stream.current().kind == TokenKind::Else {
                state.stream.next();
                utils::skip_colon(state)?;

                let body = blocks::body(state, &TokenKind::EndIf)?;

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
            let then = if state.stream.current().kind == TokenKind::LeftBrace {
                utils::skip_left_brace(state)?;
                let then = blocks::body(state, &TokenKind::RightBrace)?;
                utils::skip_right_brace(state)?;
                then
            } else {
                vec![parser::statement(state)?]
            };

            let mut else_ifs: Vec<ElseIf> = Vec::new();
            loop {
                if state.stream.current().kind == TokenKind::ElseIf {
                    state.stream.next();

                    utils::skip_left_parenthesis(state)?;

                    let condition = expressions::lowest_precedence(state)?;

                    utils::skip_right_parenthesis(state)?;

                    let body = if state.stream.current().kind == TokenKind::LeftBrace {
                        utils::skip_left_brace(state)?;
                        let then = blocks::body(state, &TokenKind::RightBrace)?;
                        utils::skip_right_brace(state)?;
                        then
                    } else {
                        vec![parser::statement(state)?]
                    };

                    else_ifs.push(ElseIf { condition, body });
                } else {
                    break;
                }
            }

            if state.stream.current().kind != TokenKind::Else {
                return Ok(Statement::If {
                    condition,
                    then,
                    else_ifs,
                    r#else: None,
                });
            }

            utils::skip(state, TokenKind::Else)?;

            let r#else;
            if state.stream.current().kind == TokenKind::LeftBrace {
                utils::skip_left_brace(state)?;

                r#else = blocks::body(state, &TokenKind::RightBrace)?;

                utils::skip_right_brace(state)?;
            } else {
                r#else = vec![parser::statement(state)?];
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
