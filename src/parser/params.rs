use crate::{
    ast::{Arg, ParamList, PropertyFlag},
    Expression, Param, ParseError,
};
use crate::TokenKind;

use super::{precedence::Precedence, ParseResult, Parser};

impl Parser {
    pub(crate) fn param_list(&mut self) -> Result<ParamList, ParseError> {
        let mut params = ParamList::new();

        while !self.is_eof() && self.current.kind != TokenKind::RightParen {
            let mut param_type = None;

            let flag: Option<PropertyFlag> = if matches!(
                self.current.kind,
                TokenKind::Public | TokenKind::Protected | TokenKind::Private
            ) {
                let flag = self.current.kind.clone().into();
                self.next();
                Some(flag)
            } else {
                None
            };

            // 1. If we don't see a variable, we should expect a type-string.
            if !matches!(
                self.current.kind,
                TokenKind::Variable(_) | TokenKind::Ellipsis | TokenKind::Ampersand
            ) || self.config.force_type_strings
            {
                // 1a. Try to parse the type.
                param_type = Some(self.type_string()?);
            }

            let mut variadic = false;
            let mut by_ref = false;

            match self.current.kind {
                TokenKind::Ellipsis => {
                    self.next();
                    variadic = true;
                }
                TokenKind::Ampersand => {
                    self.next();
                    by_ref = true;
                }
                _ => {}
            };

            // 2. Then expect a variable.
            let var = expect!(self, TokenKind::Variable(v), v, "expected variable");

            let mut default = None;
            if self.current.kind == TokenKind::Equals {
                self.next();
                default = Some(self.expression(Precedence::Lowest)?);
            }

            params.push(Param {
                name: Expression::Variable { name: var },
                r#type: param_type,
                variadic,
                default,
                flag,
                by_ref,
            });

            self.optional_comma()?;
        }

        Ok(params)
    }

    pub(crate) fn args_list(&mut self) -> ParseResult<Vec<Arg>> {
        let mut args = Vec::new();

        while !self.is_eof() && self.current.kind != TokenKind::RightParen {
            let mut name = None;
            let mut unpack = false;
            if matches!(self.current.kind, TokenKind::Identifier(_))
                && self.peek.kind == TokenKind::Colon
            {
                name = Some(self.ident_maybe_reserved()?);
                self.next();
            } else if self.current.kind == TokenKind::Ellipsis {
                self.next();
                unpack = true;
            }

            if unpack && self.current.kind == TokenKind::RightParen {
                args.push(Arg {
                    name: None,
                    unpack: false,
                    value: Expression::VariadicPlaceholder,
                });

                break;
            }

            let value = self.expression(Precedence::Lowest)?;

            args.push(Arg {
                name,
                unpack,
                value,
            });

            self.optional_comma()?;
        }

        Ok(args)
    }
}
