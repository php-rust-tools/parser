use crate::lexer::token::TokenKind;
use crate::parser::ast::attributes::Attribute;
use crate::parser::ast::attributes::AttributeGroup;
use crate::parser::error::ParseResult;
use crate::parser::internal::precedences::Precedence;
use crate::parser::state::State;
use crate::parser::Parser;

impl Parser {
    pub(in crate::parser) fn gather_attributes(&self, state: &mut State) -> ParseResult<bool> {
        state.gather_comments();

        if state.current.kind != TokenKind::Attribute {
            return Ok(false);
        }

        let start = state.current.span;
        let mut members = vec![];

        state.next();

        while state.current.kind != TokenKind::RightBracket {
            let start = state.current.span;
            let expression = self.expression(state, Precedence::Lowest)?;
            let end = state.current.span;

            members.push(Attribute {
                start,
                expression,
                end,
            });

            if state.current.kind == TokenKind::Comma {
                state.next();
            } else {
                break;
            }
        }

        let end = state.current.span;
        self.rbracket(state)?;

        state.attribute(AttributeGroup {
            start,
            members,
            end,
        });

        // recursive, looking for multiple attribute brackets after each other.
        self.gather_attributes(state).map(|_| true)
    }
}
