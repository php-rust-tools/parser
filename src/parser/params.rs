use crate::expect_token;
use crate::parser::error::ParseError;
use crate::TokenKind;
use crate::{
    ast::{Arg, ParamList, PropertyFlag},
    Expression, Param,
};

use super::{precedence::Precedence, ParseResult, Parser};

#[derive(Debug)]
pub enum ParamPosition {
    Function,
    Method(String),
    AbstractMethod(String),
}

impl Parser {
    pub(crate) fn param_list(&mut self, position: ParamPosition) -> Result<ParamList, ParseError> {
        let mut params = ParamList::new();

        while !self.is_eof() && self.current.kind != TokenKind::RightParen {
            let mut param_type = None;

            let flags: Vec<PropertyFlag> = self
                .promoted_property_flags()?
                .iter()
                .map(|f| f.into())
                .collect();

            if !flags.is_empty() {
                match position {
                    ParamPosition::Method(name) if name != "__construct" => {
                        return Err(ParseError::PromotedPropertyOutsideConstructor(
                            self.current.span,
                        ));
                    }
                    ParamPosition::AbstractMethod(name) => {
                        if name == "__construct" {
                            return Err(ParseError::PromotedPropertyOnAbstractConstructor(
                                self.current.span,
                            ));
                        } else {
                            return Err(ParseError::PromotedPropertyOutsideConstructor(
                                self.current.span,
                            ));
                        }
                    }
                    _ => {}
                }
            }

            // If this is a readonly promoted property, or we don't see a variable
            if self.config.force_type_strings
                || flags.contains(&PropertyFlag::Readonly)
                || !matches!(
                    self.current.kind,
                    TokenKind::Variable(_) | TokenKind::Ellipsis | TokenKind::Ampersand
                )
            {
                // Try to parse the type.
                param_type = Some(self.type_string()?);
            }

            let mut variadic = false;
            let mut by_ref = false;

            if matches!(self.current.kind, TokenKind::Ampersand) {
                self.next();
                by_ref = true;
            }

            if matches!(self.current.kind, TokenKind::Ellipsis) {
                self.next();
                if !flags.is_empty() {
                    return Err(ParseError::VariadicPromotedProperty(self.current.span));
                }

                variadic = true;
            }

            // 2. Then expect a variable.
            let var = expect_token!([
                TokenKind::Variable(v) => v
            ], self, "a varaible");

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
                flags,
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
