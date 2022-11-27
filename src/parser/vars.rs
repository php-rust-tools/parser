use super::{ParseError, ParseResult, Precedence};
use crate::TokenKind;
use crate::{Expression, Parser};

impl Parser {
    pub(crate) fn dynamic_variable(&mut self) -> ParseResult<Expression> {
        self.next();

        Ok(match &self.current.kind {
            TokenKind::LeftBrace => {
                self.next();

                let name = self.expression(Precedence::Lowest)?;

                self.rbrace()?;

                Expression::DynamicVariable {
                    name: Box::new(name),
                }
            }
            TokenKind::Variable(variable) => {
                let variable = variable.clone();

                self.next();

                Expression::DynamicVariable {
                    name: Box::new(Expression::Variable { name: variable }),
                }
            }
            _ => {
                return Err(ParseError::UnexpectedToken(
                    self.current.kind.to_string(),
                    self.current.span,
                ))
            }
        })
    }
}
