use crate::expect_token;
use crate::expected_scope;
use crate::lexer::token::TokenKind;
use crate::parser::ast::functions::ArrowFunction;
use crate::parser::ast::functions::Closure;
use crate::parser::ast::functions::ClosureUse;
use crate::parser::ast::functions::Function;
use crate::parser::ast::functions::Method;
use crate::parser::ast::identifiers::Identifier;
use crate::parser::ast::modifiers::MethodModifierGroup;
use crate::parser::ast::Expression;
use crate::parser::ast::Statement;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::internal::precedences::Precedence;
use crate::parser::state::Scope;
use crate::parser::state::State;
use crate::parser::Parser;
use crate::scoped;

impl Parser {
    pub(in crate::parser) fn anonymous_function(
        &self,
        state: &mut State,
    ) -> ParseResult<Expression> {
        let start = state.current.span;

        let is_static = if state.current.kind == TokenKind::Static {
            state.next();

            true
        } else {
            false
        };

        expect_token!([TokenKind::Function], state, ["`function`"]);

        let by_ref = if state.current.kind == TokenKind::Ampersand {
            state.next();
            true
        } else {
            false
        };

        let attributes = state.get_attributes();
        let parameters = self.function_parameter_list(state)?;

        let mut uses = vec![];
        if state.current.kind == TokenKind::Use {
            state.next();

            self.lparen(state)?;

            while state.current.kind != TokenKind::RightParen {
                let mut by_ref = false;
                if state.current.kind == TokenKind::Ampersand {
                    state.next();

                    by_ref = true;
                }

                // TODO(azjezz): this shouldn't call expr, we should have a function
                // just for variables, so we don't have to go through the whole `match` in `expression(...)`
                let var = match self.expression(state, Precedence::Lowest)? {
                    s @ Expression::Variable { .. } => ClosureUse { var: s, by_ref },
                    _ => {
                        return Err(ParseError::UnexpectedToken(
                            "expected variable".into(),
                            state.current.span,
                        ))
                    }
                };

                uses.push(var);

                if state.current.kind == TokenKind::Comma {
                    state.next();
                } else {
                    break;
                }
            }

            self.rparen(state)?;
        }

        let mut return_ty = None;
        if state.current.kind == TokenKind::Colon {
            self.colon(state)?;

            return_ty = Some(self.get_type(state)?);
        }

        let body = scoped!(state, Scope::AnonymousFunction(is_static), {
            self.lbrace(state)?;

            let body = self.block(state, &TokenKind::RightBrace)?;

            self.rbrace(state)?;

            body
        });

        let end = state.current.span;

        Ok(Expression::Closure(Closure {
            start,
            end,
            attributes,
            parameters,
            uses,
            return_ty,
            body,
            r#static: is_static,
            by_ref,
        }))
    }

    pub(in crate::parser) fn arrow_function(&self, state: &mut State) -> ParseResult<Expression> {
        let start = state.current.span;

        let is_static = if state.current.kind == TokenKind::Static {
            state.next();

            true
        } else {
            false
        };

        expect_token!([TokenKind::Fn], state, ["`fn`"]);

        let by_ref = if state.current.kind == TokenKind::Ampersand {
            state.next();
            true
        } else {
            false
        };

        let attributes = state.get_attributes();
        let parameters = self.function_parameter_list(state)?;

        let mut return_type = None;
        if state.current.kind == TokenKind::Colon {
            self.colon(state)?;

            return_type = Some(self.get_type(state)?);
        }

        expect_token!([TokenKind::DoubleArrow], state, ["`=>`"]);

        let body = scoped!(state, Scope::ArrowFunction(is_static), {
            Box::new(self.expression(state, Precedence::Lowest)?)
        });

        let end = state.current.span;

        Ok(Expression::ArrowFunction(ArrowFunction {
            start,
            end,
            attributes,
            parameters,
            return_type,
            body,
            by_ref,
            r#static: is_static,
        }))
    }

    pub(in crate::parser) fn function(&self, state: &mut State) -> ParseResult<Statement> {
        let start = state.current.span;

        expect_token!([TokenKind::Function], state, ["`function`"]);

        let by_ref = if state.current.kind == TokenKind::Ampersand {
            state.next();
            true
        } else {
            false
        };

        let name = if state.current.kind == TokenKind::Null {
            let start = state.current.span;
            let name = "null".into();
            state.next();
            let end = state.current.span;

            Identifier { start, name, end }
        } else {
            self.ident(state)?
        };

        // get attributes before processing parameters, otherwise
        // parameters will steal attributes of this function.
        let attributes = state.get_attributes();

        let parameters = self.function_parameter_list(state)?;

        let mut return_type = None;

        if state.current.kind == TokenKind::Colon {
            self.colon(state)?;

            return_type = Some(self.get_type(state)?);
        }

        let body = scoped!(state, Scope::Function(name.clone()), {
            self.lbrace(state)?;

            let body = self.block(state, &TokenKind::RightBrace)?;

            self.rbrace(state)?;

            body
        });

        let end = state.current.span;

        Ok(Statement::Function(Function {
            start,
            end,
            name,
            attributes,
            parameters,
            return_type,
            body,
            by_ref,
        }))
    }

    pub(in crate::parser) fn method(
        &self,
        state: &mut State,
        flags: MethodModifierGroup,
    ) -> ParseResult<Method> {
        // TODO(azjezz): this is incorrect.
        // start span is actually the start span of the first flag.
        let start = state.current.span;

        expect_token!([TokenKind::Function], state, ["`function`"]);

        let by_ref = if state.current.kind == TokenKind::Ampersand {
            state.next();
            true
        } else {
            false
        };

        let name = self.ident_maybe_reserved(state)?;

        let has_body = expected_scope!([
            Scope::Class(_, flags_group, _) => {
                if !flags_group.has_abstract() && flags.has_abstract() {
                    return Err(ParseError::AbstractModifierOnNonAbstractClassMethod(
                        state.current.span,
                    ));
                }

                !flags.has_abstract()
            },
            Scope::Trait(_) => !flags.has_abstract(),
            Scope::Interface(_) => false,
            Scope::Enum(enum_name, _) => {
                if name.to_string() == "__construct" {
                    return Err(ParseError::ConstructorInEnum(
                        state.named(&enum_name),
                        state.current.span,
                    ));
                }

                true
            },
            Scope::AnonymousClass(_) => true,
        ], state);

        // get attributes before processing parameters, otherwise
        // parameters will steal attributes of this method.
        let attributes = state.get_attributes();

        let (parameters, body, return_type) =
            scoped!(state, Scope::Method(name.clone(), flags.clone()), {
                let parameters = self.method_parameter_list(state)?;

                let mut return_type = None;

                if state.current.kind == TokenKind::Colon {
                    self.colon(state)?;

                    return_type = Some(self.get_type(state)?);
                }

                if !has_body {
                    self.semi(state)?;

                    (parameters, None, return_type)
                } else {
                    self.lbrace(state)?;

                    let body = self.block(state, &TokenKind::RightBrace)?;

                    self.rbrace(state)?;

                    (parameters, Some(body), return_type)
                }
            });

        let end = state.current.span;

        Ok(Method {
            start,
            end,
            attributes,
            name,
            parameters,
            body,
            return_type,
            by_ref,
            flags,
        })
    }
}
