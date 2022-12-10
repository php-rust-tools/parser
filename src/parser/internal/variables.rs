use crate::expected_token_err;
use crate::lexer::token::TokenKind;
use crate::parser::ast::variables::BracedVariableVariable;
use crate::parser::ast::variables::SimpleVariable;
use crate::parser::ast::variables::Variable;
use crate::parser::ast::variables::VariableVariable;
use crate::parser::error::ParseResult;
use crate::parser::expressions;
use crate::parser::internal::utils;
use crate::parser::state::State;

pub fn simple_variable(state: &mut State) -> ParseResult<SimpleVariable> {
    if let TokenKind::Variable(name) = state.current.kind.clone() {
        let span = state.current.span;
        state.next();

        return Ok(SimpleVariable { span, name });
    }

    expected_token_err!("a variable", state)
}

pub fn dynamic_variable(state: &mut State) -> ParseResult<Variable> {
    match state.current.kind.clone() {
        TokenKind::Variable(name) => {
            let span = state.current.span;
            state.next();

            Ok(Variable::SimpleVariable(SimpleVariable { span, name }))
        }
        TokenKind::DollarLeftBrace => {
            let start = state.current.span;
            state.next();

            let expr = expressions::lowest_precedence(state)?;

            let end = utils::skip_right_brace(state)?;

            Ok(Variable::BracedVariableVariable(BracedVariableVariable {
                start,
                variable: Box::new(expr),
                end,
            }))
        }
        // todo(azjezz): figure out why the lexer does this.
        TokenKind::Dollar if state.peek.kind == TokenKind::LeftBrace => {
            let start = state.current.span;
            state.next();
            state.next();

            let expr = expressions::lowest_precedence(state)?;

            let end = utils::skip_right_brace(state)?;

            Ok(Variable::BracedVariableVariable(BracedVariableVariable {
                start,
                variable: Box::new(expr),
                end,
            }))
        }
        TokenKind::Dollar => {
            let span = state.current.span;
            state.next();

            let variable = dynamic_variable(state)?;

            Ok(Variable::VariableVariable(VariableVariable {
                span,
                variable: Box::new(variable),
            }))
        }
        _ => {
            expected_token_err!("a variable", state)
        }
    }
}
