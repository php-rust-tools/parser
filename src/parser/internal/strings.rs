use crate::expect_token;
use crate::expected_token_err;
use crate::lexer::error::SyntaxError;
use crate::lexer::token::DocStringIndentationKind;
use crate::lexer::token::TokenKind;
use crate::parser::ast::identifiers::Identifier;
use crate::parser::ast::literals::Literal;
use crate::parser::ast::literals::LiteralInteger;
use crate::parser::ast::literals::LiteralString;
use crate::parser::ast::operators::ArithmeticOperation;
use crate::parser::ast::variables::Variable;
use crate::parser::ast::Expression;
use crate::parser::ast::StringPart;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::expressions::create;
use crate::parser::internal::identifiers;
use crate::parser::internal::utils;
use crate::parser::internal::variables;
use crate::parser::state::State;

#[inline(always)]
pub fn interpolated(state: &mut State) -> ParseResult<Expression> {
    let mut parts = Vec::new();

    while state.stream.current().kind != TokenKind::DoubleQuote {
        if let Some(part) = part(state)? {
            parts.push(part);
        }
    }

    state.stream.next();

    Ok(Expression::InterpolatedString { parts })
}

#[inline(always)]
pub fn shell_exec(state: &mut State) -> ParseResult<Expression> {
    state.stream.next();

    let mut parts = Vec::new();

    while state.stream.current().kind != TokenKind::Backtick {
        if let Some(part) = part(state)? {
            parts.push(part);
        }
    }

    state.stream.next();

    Ok(Expression::ShellExec { parts })
}

#[inline(always)]
pub fn heredoc(state: &mut State) -> ParseResult<Expression> {
    let span = state.stream.current().span;
    state.stream.next();

    let mut parts = Vec::new();

    while !matches!(
        state.stream.current().kind,
        TokenKind::EndDocString(_, _, _)
    ) {
        if let Some(part) = part(state)? {
            parts.push(part);
        }
    }

    let (indentation_type, indentation_amount) = match &state.stream.current().kind {
        TokenKind::EndDocString(_, indentation_type, indentation_amount) => {
            (indentation_type.clone(), *indentation_amount)
        }
        _ => unreachable!(),
    };

    state.stream.next();

    let mut new_line = true;
    if indentation_type != DocStringIndentationKind::None {
        let indentation_char: u8 = indentation_type.into();

        for part in parts.iter_mut() {
            // We only need to strip and validate indentation
            // for individual lines, so we can skip checks if
            // we know we're not on a new line.
            if !new_line {
                continue;
            }

            match part {
                StringPart::Literal(bytes) => {
                    // 1. If this line doesn't start with any whitespace,
                    //    we can return an error early because we know
                    //    the label was indented.
                    if !bytes.starts_with(&[b' ']) && !bytes.starts_with(&[b'\t']) {
                        return Err(ParseError::SyntaxError(
                            SyntaxError::InvalidDocBodyIndentationLevel(indentation_amount, span),
                        ));
                    }

                    // 2. If this line doesn't start with the correct
                    //    type of whitespace, we can also return an error.
                    if !bytes.starts_with(&[indentation_char]) {
                        return Err(ParseError::SyntaxError(SyntaxError::InvalidDocIndentation(
                            span,
                        )));
                    }

                    // 3. We now know that the whitespace at the start of
                    //    this line is correct, so we need to check that the
                    //    amount of whitespace is correct too. In this case,
                    //    the amount of whitespace just needs to be at least
                    //    the same, so we can create a vector containing the
                    //    minimum and check using `starts_with()`.
                    let expected_whitespace_buffer = vec![indentation_char; indentation_amount];
                    if !bytes.starts_with(&expected_whitespace_buffer) {
                        return Err(ParseError::SyntaxError(
                            SyntaxError::InvalidDocBodyIndentationLevel(indentation_amount, span),
                        ));
                    }

                    // 4. All of the above checks have passed, so we know
                    //    there are no more possible errors. Let's now
                    //    strip the leading whitespace accordingly.
                    *bytes = bytes
                        .strip_prefix(&expected_whitespace_buffer[..])
                        .unwrap()
                        .into();
                    new_line = bytes.ends_with(&[b'\n']);
                }
                _ => continue,
            }
        }
    }

    Ok(Expression::Heredoc { parts })
}

#[inline(always)]
pub fn nowdoc(state: &mut State) -> ParseResult<Expression> {
    let span = state.stream.current().span;

    state.stream.next();

    let mut string_part = expect_token!([
        TokenKind::StringPart(s) => s,
    ], state, "constant string");

    let (indentation_type, indentation_amount) = match &state.stream.current().kind {
        TokenKind::EndDocString(_, indentation_type, indentation_amount) => {
            (indentation_type.clone(), *indentation_amount)
        }
        _ => unreachable!(),
    };

    state.stream.next();

    if indentation_type != DocStringIndentationKind::None {
        let indentation_char: u8 = indentation_type.into();

        let mut lines = string_part
            .split(|b| *b == b'\n')
            .map(|s| s.to_vec())
            .collect::<Vec<Vec<u8>>>();

        for line in lines.iter_mut() {
            if line.is_empty() {
                continue;
            }

            // 1. If this line doesn't start with any whitespace,
            //    we can return an error early because we know
            //    the label was indented.
            if !line.starts_with(&[b' ']) && !line.starts_with(&[b'\t']) {
                return Err(ParseError::SyntaxError(
                    SyntaxError::InvalidDocBodyIndentationLevel(indentation_amount, span),
                ));
            }

            // 2. If this line doesn't start with the correct
            //    type of whitespace, we can also return an error.
            if !line.starts_with(&[indentation_char]) {
                return Err(ParseError::SyntaxError(SyntaxError::InvalidDocIndentation(
                    span,
                )));
            }

            // 3. We now know that the whitespace at the start of
            //    this line is correct, so we need to check that the
            //    amount of whitespace is correct too. In this case,
            //    the amount of whitespace just needs to be at least
            //    the same, so we can create a vector containing the
            //    minimum and check using `starts_with()`.
            let expected_whitespace_buffer = vec![indentation_char; indentation_amount];
            if !line.starts_with(&expected_whitespace_buffer) {
                return Err(ParseError::SyntaxError(
                    SyntaxError::InvalidDocBodyIndentationLevel(indentation_amount, span),
                ));
            }

            // 4. All of the above checks have passed, so we know
            //    there are no more possible errors. Let's now
            //    strip the leading whitespace accordingly.
            *line = line
                .strip_prefix(&expected_whitespace_buffer[..])
                .unwrap()
                .into();
        }

        let mut bytes = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            bytes.extend(line);
            if i < lines.len() - 1 {
                bytes.push(b'\n');
            }
        }
        string_part = bytes.into();
    }

    Ok(Expression::Nowdoc { value: string_part })
}

fn part(state: &mut State) -> ParseResult<Option<StringPart>> {
    Ok(match &state.stream.current().kind {
        TokenKind::StringPart(s) => {
            let part = if s.len() > 0 {
                Some(StringPart::Literal(s.clone()))
            } else {
                None
            };

            state.stream.next();
            part
        }
        TokenKind::DollarLeftBrace => {
            let variable = variables::dynamic_variable(state)?;

            Some(StringPart::Expression(Box::new(Expression::Variable(
                variable,
            ))))
        }
        TokenKind::LeftBrace => {
            // "{$expr}"
            state.stream.next();
            let e = create(state)?;
            utils::skip_right_brace(state)?;
            Some(StringPart::Expression(Box::new(e)))
        }
        TokenKind::Variable(_) => {
            // "$expr", "$expr[0]", "$expr[name]", "$expr->a"
            let variable = Expression::Variable(variables::dynamic_variable(state)?);
            let current = state.stream.current();
            let e = match &current.kind {
                TokenKind::LeftBracket => {
                    state.stream.next();
                    // Full expression syntax is not allowed here,
                    // so we can't call expression.
                    let current = state.stream.current();
                    let index = match &current.kind {
                        TokenKind::LiteralInteger(value) => {
                            state.stream.next();

                            Expression::Literal(Literal::Integer(LiteralInteger {
                                span: current.span,
                                value: value.clone(),
                            }))
                        }
                        TokenKind::Minus => {
                            let span = current.span;
                            state.stream.next();
                            let literal = state.stream.current();
                            if let TokenKind::LiteralInteger(value) = &literal.kind {
                                state.stream.next();

                                Expression::ArithmeticOperation(ArithmeticOperation::Negative {
                                    span,
                                    right: Box::new(Expression::Literal(Literal::Integer(
                                        LiteralInteger {
                                            span: literal.span,
                                            value: value.clone(),
                                        },
                                    ))),
                                })
                            } else {
                                return expected_token_err!("an integer", state);
                            }
                        }
                        TokenKind::Identifier(ident) => {
                            state.stream.next();

                            Expression::Literal(Literal::String(LiteralString {
                                span: current.span,
                                value: ident.clone(),
                            }))
                        }
                        TokenKind::Variable(_) => Expression::Variable(Variable::SimpleVariable(
                            variables::simple_variable(state)?,
                        )),
                        _ => {
                            return expected_token_err!(
                                ["`-`", "an integer", "an identifier", "a variable"],
                                state
                            );
                        }
                    };

                    utils::skip_right_bracket(state)?;

                    Expression::ArrayIndex {
                        array: Box::new(variable),
                        index: Some(Box::new(index)),
                    }
                }
                TokenKind::Arrow => {
                    let span = current.span;
                    state.stream.next();
                    Expression::PropertyFetch {
                        target: Box::new(variable),
                        span,
                        property: Box::new(Expression::Identifier(Identifier::SimpleIdentifier(
                            identifiers::identifier_maybe_reserved(state)?,
                        ))),
                    }
                }
                TokenKind::NullsafeArrow => {
                    let span = current.span;
                    state.stream.next();
                    Expression::NullsafePropertyFetch {
                        target: Box::new(variable),
                        span,
                        property: Box::new(Expression::Identifier(Identifier::SimpleIdentifier(
                            identifiers::identifier_maybe_reserved(state)?,
                        ))),
                    }
                }
                _ => variable,
            };
            Some(StringPart::Expression(Box::new(e)))
        }
        _ => {
            return expected_token_err!(["`${`", "`{$", "`\"`", "a variable"], state);
        }
    })
}
