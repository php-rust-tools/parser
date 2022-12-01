use crate::lexer::token::TokenKind;
use crate::parser::ast::Expression;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::precedence::Precedence;
use crate::parser::state::State;
use crate::parser::Parser;

impl Parser {
    pub(in crate::parser) fn dynamic_variable(&self, state: &mut State) -> ParseResult<Expression> {
        state.next();

        Ok(match &state.current.kind {
            TokenKind::LeftBrace => {
                state.next();

                let name = self.expression(state, Precedence::Lowest)?;

                self.rbrace(state)?;

                Expression::DynamicVariable {
                    name: Box::new(name),
                }
            }
            TokenKind::Variable(variable) => {
                let variable = variable.clone();

                state.next();

                Expression::DynamicVariable {
                    name: Box::new(Expression::Variable { name: variable }),
                }
            }
            _ => {
                return Err(ParseError::UnexpectedToken(
                    state.current.kind.to_string(),
                    state.current.span,
                ))
            }
        })
    }
}
