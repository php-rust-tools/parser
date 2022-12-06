use crate::lexer::token::TokenKind;
use crate::parser::ast::functions::FunctionParameter;
use crate::parser::ast::functions::FunctionParameterList;
use crate::parser::ast::functions::MethodParameter;
use crate::parser::ast::functions::MethodParameterList;
use crate::parser::ast::Arg;
use crate::parser::ast::Expression;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::internal::precedences::Precedence;
use crate::parser::state::Scope;
use crate::parser::state::State;
use crate::parser::Parser;

use super::identifiers::is_reserved_ident;

impl Parser {
    pub(in crate::parser) fn function_parameter_list(
        &self,
        state: &mut State,
    ) -> Result<FunctionParameterList, ParseError> {
        let mut members = Vec::new();

        let list_start = state.current.span;
        self.lparen(state)?;

        state.skip_comments();

        while !state.is_eof() && state.current.kind != TokenKind::RightParen {
            let start = state.current.span;

            self.gather_attributes(state)?;

            let ty = self.get_optional_type(state)?;

            let mut variadic = false;
            let mut by_ref = false;

            if state.current.kind == TokenKind::Ampersand {
                state.next();
                by_ref = true;
            }

            if state.current.kind == TokenKind::Ellipsis {
                state.next();

                variadic = true;
            }

            // 2. Then expect a variable.
            let var = self.var(state)?;

            let mut default = None;
            if state.current.kind == TokenKind::Equals {
                state.next();
                default = Some(self.expression(state, Precedence::Lowest)?);
            }

            let end = state.current.span;

            members.push(FunctionParameter {
                start,
                end,
                name: var,
                attributes: state.get_attributes(),
                r#type: ty,
                variadic,
                default,
                by_ref,
            });

            state.skip_comments();

            if state.current.kind == TokenKind::Comma {
                state.next();
            } else {
                break;
            }
        }

        self.rparen(state)?;

        let list_end = state.current.span;

        Ok(FunctionParameterList {
            start: list_start,
            end: list_end,
            members,
        })
    }

    /// TODO(azjezz): split this into `method_parameter_list` and `abstract_method_parameter_list`?
    ///               abstract method parameter list won't have a promoted property, so some of the logic
    ///               here can be avoided for performance.
    pub(in crate::parser) fn method_parameter_list(
        &self,
        state: &mut State,
    ) -> Result<MethodParameterList, ParseError> {
        let mut class_name = String::new();
        let construct: i8 = match state.scope()? {
            Scope::Method(name, modifiers) => {
                if name.to_string() != "__construct" {
                    0
                } else {
                    match state.parent()? {
                        // can only have abstract ctor
                        Scope::Interface(_) => 1,
                        // can only have concret ctor
                        Scope::AnonymousClass(_) => {
                            class_name = state.named("class@anonymous");

                            2
                        }
                        // can have either abstract or concret ctor,
                        // depens on method modifiers.
                        Scope::Class(name, _, _) | Scope::Trait(name) => {
                            if modifiers.has_abstract() {
                                1
                            } else {
                                class_name = state.named(name);

                                2
                            }
                        }
                        _ => unreachable!(),
                    }
                }
            }
            scope => unreachable!("shouldn't reach scope `{:?}`", scope),
        };

        let mut members = Vec::new();

        let list_start = state.current.span;
        self.lparen(state)?;

        state.skip_comments();

        while !state.is_eof() && state.current.kind != TokenKind::RightParen {
            let start = state.current.span;

            self.gather_attributes(state)?;

            let modifiers = self.get_promoted_property_modifier_group(self.modifiers(state)?)?;

            let ty = self.get_optional_type(state)?;

            let mut variadic = false;
            let mut by_ref = false;

            if matches!(state.current.kind, TokenKind::Ampersand) {
                state.next();
                by_ref = true;
            }

            if matches!(state.current.kind, TokenKind::Ellipsis) {
                state.next();
                if !modifiers.is_empty() {
                    return Err(ParseError::VariadicPromotedProperty(state.current.span));
                }

                variadic = true;
            }

            // 2. Then expect a variable.
            let var = self.var(state)?;

            if !modifiers.is_empty() {
                match construct {
                    0 => {
                        return Err(ParseError::PromotedPropertyOutsideConstructor(
                            state.current.span,
                        ));
                    }
                    1 => {
                        return Err(ParseError::PromotedPropertyOnAbstractConstructor(
                            state.current.span,
                        ));
                    }
                    _ => {}
                }

                match &ty {
                    Some(ty) => {
                        if ty.includes_callable() || ty.is_bottom() {
                            return Err(ParseError::ForbiddenTypeUsedInProperty(
                                class_name,
                                var.to_string(),
                                ty.clone(),
                                state.current.span,
                            ));
                        }
                    }
                    None => {
                        if modifiers.has_readonly() {
                            return Err(ParseError::MissingTypeForReadonlyProperty(
                                class_name,
                                var.to_string(),
                                state.current.span,
                            ));
                        }
                    }
                }
            }

            let mut default = None;
            if state.current.kind == TokenKind::Equals {
                state.next();
                default = Some(self.expression(state, Precedence::Lowest)?);
            }

            let end = state.current.span;

            members.push(MethodParameter {
                start,
                end,
                name: var,
                attributes: state.get_attributes(),
                r#type: ty,
                variadic,
                default,
                modifiers,
                by_ref,
            });

            state.skip_comments();

            if state.current.kind == TokenKind::Comma {
                state.next();
            } else {
                break;
            }
        }

        self.rparen(state)?;

        let list_end = state.current.span;

        Ok(MethodParameterList {
            start: list_start,
            end: list_end,
            members,
        })
    }

    pub(in crate::parser) fn args_list(&self, state: &mut State) -> ParseResult<Vec<Arg>> {
        self.lparen(state)?;
        state.skip_comments();

        let mut args = Vec::new();

        while !state.is_eof() && state.current.kind != TokenKind::RightParen {
            let mut name = None;
            let mut unpack = false;
            if (matches!(state.current.kind, TokenKind::Identifier(_))
                || is_reserved_ident(&state.current.kind))
                && state.peek.kind == TokenKind::Colon
            {
                name = Some(self.ident_maybe_reserved(state)?);
                state.next();
            } else if state.current.kind == TokenKind::Ellipsis {
                state.next();
                unpack = true;
            }

            if unpack && state.current.kind == TokenKind::RightParen {
                args.push(Arg {
                    name: None,
                    unpack: false,
                    value: Expression::VariadicPlaceholder,
                });

                break;
            }

            let value = self.expression(state, Precedence::Lowest)?;

            args.push(Arg {
                name,
                unpack,
                value,
            });

            if state.current.kind == TokenKind::Comma {
                state.next();
            } else {
                break;
            }
        }

        self.rparen(state)?;

        Ok(args)
    }
}
