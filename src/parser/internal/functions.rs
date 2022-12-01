use crate::expect_token;
use crate::expected_scope;
use crate::lexer::token::TokenKind;
use crate::parser::ast::ClassFlag;
use crate::parser::ast::ClosureUse;
use crate::parser::ast::Expression;
use crate::parser::ast::MethodFlag;
use crate::parser::ast::Statement;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::internal::precedence::Precedence;
use crate::parser::state::Scope;
use crate::parser::state::State;
use crate::parser::Parser;
use crate::scoped;

impl Parser {
    pub(in crate::parser) fn anonymous_function(
        &self,
        state: &mut State,
    ) -> ParseResult<Expression> {
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

        scoped!(state, Scope::AnonymousFunction(is_static), {
            self.lparen(state)?;

            let params = self.param_list(state)?;

            self.rparen(state)?;

            let mut uses = vec![];
            if state.current.kind == TokenKind::Use {
                state.next();

                self.lparen(state)?;

                while state.current.kind != TokenKind::RightParen {
                    let mut by_ref = false;
                    if state.current.kind == TokenKind::Ampersand {
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

                    self.optional_comma(state)?;
                }

                self.rparen(state)?;
            }

            let mut return_type = None;
            if state.current.kind == TokenKind::Colon {
                self.colon(state)?;

                return_type = Some(self.type_string(state)?);
            }

            self.lbrace(state)?;

            let body = self.block(state, &TokenKind::RightBrace)?;

            self.rbrace(state)?;

            Ok(Expression::Closure {
                params,
                uses,
                return_type,
                body,
                r#static: is_static,
                by_ref,
            })
        })
    }

    pub(in crate::parser) fn arrow_function(&self, state: &mut State) -> ParseResult<Expression> {
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

        scoped!(state, Scope::ArrowFunction(is_static), {
            self.lparen(state)?;

            let params = self.param_list(state)?;

            self.rparen(state)?;

            let mut return_type = None;
            if state.current.kind == TokenKind::Colon {
                self.colon(state)?;

                return_type = Some(self.type_string(state)?);
            }

            expect_token!([TokenKind::DoubleArrow], state, ["`=>`"]);

            let value = self.expression(state, Precedence::Lowest)?;

            Ok(Expression::ArrowFunction {
                params,
                return_type,
                expr: Box::new(value),
                by_ref,
                r#static: is_static,
            })
        })
    }

    pub(in crate::parser) fn function(&self, state: &mut State) -> ParseResult<Statement> {
        expect_token!([TokenKind::Function], state, ["`function`"]);

        let by_ref = if state.current.kind == TokenKind::Ampersand {
            state.next();
            true
        } else {
            false
        };

        let name = self.ident(state)?;

        scoped!(state, Scope::Function(name.clone()), {
            self.lparen(state)?;

            let params = self.param_list(state)?;

            self.rparen(state)?;

            let mut return_type = None;

            if state.current.kind == TokenKind::Colon {
                self.colon(state)?;

                return_type = Some(self.type_string(state)?);
            }

            self.lbrace(state)?;

            let body = self.block(state, &TokenKind::RightBrace)?;

            self.rbrace(state)?;

            Ok(Statement::Function {
                name: name.into(),
                params,
                body,
                return_type,
                by_ref,
            })
        })
    }

    pub(in crate::parser) fn method(
        &self,
        state: &mut State,
        flags: Vec<MethodFlag>,
    ) -> ParseResult<Statement> {
        expect_token!([TokenKind::Function], state, ["`function`"]);

        let by_ref = if state.current.kind == TokenKind::Ampersand {
            state.next();
            true
        } else {
            false
        };

        let name = self.ident_maybe_reserved(state)?;

        let has_body = expected_scope!([
            Scope::Class(_, cf) => {
                if !cf.contains(&ClassFlag::Abstract) && flags.contains(&MethodFlag::Abstract) {
                    return Err(ParseError::AbstractModifierOnNonAbstractClassMethod(
                        state.current.span,
                    ));
                }

                !flags.contains(&MethodFlag::Abstract)
            },
            Scope::Trait(_) => !flags.contains(&MethodFlag::Abstract),
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
            Scope::AnonymousClass => true,
        ], state);

        scoped!(state, Scope::Method(name.clone(), flags.clone()), {
            self.lparen(state)?;

            let params = self.param_list(state)?;

            self.rparen(state)?;

            let mut return_type = None;

            if state.current.kind == TokenKind::Colon {
                self.colon(state)?;

                return_type = Some(self.type_string(state)?);
            }

            if !has_body {
                self.semi(state)?;

                Ok(Statement::AbstractMethod {
                    name: name.into(),
                    params,
                    return_type,
                    flags: flags.to_vec(),
                    by_ref,
                })
            } else {
                self.lbrace(state)?;

                let body = self.block(state, &TokenKind::RightBrace)?;

                self.rbrace(state)?;

                Ok(Statement::Method {
                    name: name.into(),
                    params,
                    body,
                    return_type,
                    by_ref,
                    flags,
                })
            }
        })
    }
}
