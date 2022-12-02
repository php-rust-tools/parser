use crate::lexer::token::TokenKind;
use crate::parser::ast::Expression;
use crate::parser::error::ParseResult;
use crate::parser::internal::precedence::Precedence;
use crate::parser::state::State;
use crate::parser::Parser;
use crate::peek_token;

impl Parser {
    pub(in crate::parser) fn dynamic_variable(&self, state: &mut State) -> ParseResult<Expression> {
        state.next();

        let expr = peek_token!([
            TokenKind::LeftBrace => {
                state.next();

                let name = self.expression(state, Precedence::Lowest)?;

                self.rbrace(state)?;

                Expression::DynamicVariable {
                    name: Box::new(name),
                }
            },
            TokenKind::Variable(variable) => {
                let variable = variable;

                state.next();

                Expression::DynamicVariable {
                    name: Box::new(Expression::Variable { name: variable }),
                }
            }
        ], state, ["`{`", "a variable"]);

        Ok(expr)
    }
}
