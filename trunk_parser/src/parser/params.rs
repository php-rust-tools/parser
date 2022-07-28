use crate::{ast::ParamList, ParseError, Param, Expression};
use trunk_lexer::TokenKind;

use super::Parser;

impl Parser {
    pub(crate) fn param_list(&mut self) -> Result<ParamList, ParseError> {
        let mut params = ParamList::new();

        while ! self.is_eof() && self.current.kind != TokenKind::RightParen {
            let mut param_type = None;

            // 1. If we don't see a variable, we should expect a type-string.
            if ! matches!(self.current.kind, TokenKind::Variable(_) | TokenKind::Ellipsis) {
                // 1a. Try to parse the type.
                param_type = Some(self.type_string()?);
            }

            let variadic = if self.current.kind == TokenKind::Ellipsis {
                self.next();
                true
            } else { false };

            // 2. Then expect a variable.
            let var = expect!(self, TokenKind::Variable(v), v, "expected variable");

            let mut default = None;
            if self.current.kind == TokenKind::Equals {
                self.next();
                default = Some(self.expression(0)?);
            }

            // TODO: Support variable types and default values.
            params.push(Param {
                name: Expression::Variable(var),
                r#type: param_type,
                variadic,
                default
            });
            
            if self.current.kind == TokenKind::Comma {
                self.next();
            }
        }

        Ok(params)
    }
}