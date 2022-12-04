use crate::lexer::token::TokenKind;
use crate::parser::ast::Attribute;
use crate::parser::error::ParseResult;
use crate::parser::internal::precedence::Precedence;
use crate::parser::state::State;
use crate::parser::Parser;

impl Parser {
    pub(in crate::parser) fn gather_attributes(&self, state: &mut State) -> ParseResult<bool> {
        state.gather_comments();

        if state.current.kind != TokenKind::Attribute {
            return Ok(false);
        }

        state.next();

        while state.current.kind != TokenKind::RightBracket {
            let span = state.current.span;
            let expression = self.expression(state, Precedence::Lowest)?;

            state.attribute(Attribute { span, expression });

            if state.current.kind == TokenKind::Comma {
                state.next();
            } else {
                break;
            }
        }

        self.rbracket(state)?;

        // recursive, looking for multiple attribute brackets after each other.
        self.gather_attributes(state).map(|_| true)
    }
}
