use crate::lexer::token::TokenKind;
use crate::parser;
use crate::parser::ast::Expression;
use crate::parser::error::ParseResult;
use crate::parser::internal::identifiers;
use crate::parser::internal::precedences::Precedence;
use crate::parser::internal::utils;
use crate::parser::state::State;
use crate::peek_token;

pub fn dynamic_variable(state: &mut State) -> ParseResult<Expression> {
    state.next();

    let expr = peek_token!([
        TokenKind::LeftBrace => {
            state.next();

            // TODO(azjezz): this is not an expression! it's a constant expression
            let name = parser::expression(state, Precedence::Lowest)?;

            utils::skip_right_brace(state)?;

            Expression::DynamicVariable {
                name: Box::new(name),
            }
        },
        TokenKind::Variable(_) => {
            Expression::DynamicVariable {
                name: Box::new(Expression::Variable(identifiers::var(state)?)),
            }
        }
    ], state, ["`{`", "a variable"]);

    Ok(expr)
}
