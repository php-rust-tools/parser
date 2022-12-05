use crate::lexer::token::TokenKind;
use crate::parser::ast::attribute::Attribute;
use crate::parser::ast::attribute::AttributeGroup;
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

        let start = state.current.span;
        let mut members = vec![];

        state.next();

        while state.current.kind != TokenKind::RightBracket {
            let span = state.current.span;
            let expression = self.expression(state, Precedence::Lowest)?;

            members.push(Attribute { span, expression });

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
