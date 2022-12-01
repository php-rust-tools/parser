use crate::lexer::token::TokenKind;
use crate::parser::ast::Identifier;
use crate::parser::ast::MethodFlag;
use crate::parser::ast::Statement;
use crate::parser::ast::TraitAdaptation;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::internal::precedence::Precedence;
use crate::parser::state::State;
use crate::parser::Parser;

use crate::expect_token;
use crate::expected_token_err;
use crate::peek_token;

impl Parser {
    pub(in crate::parser) fn interface_statement(
        &self,
        state: &mut State,
    ) -> ParseResult<Statement> {
        if state.current.kind == TokenKind::Const {
            return self.parse_classish_const(state, vec![]);
        }

        if state.current.kind == TokenKind::Function {
            return self.method(state, vec![]);
        }

        let member_flags = self.interface_members_flags(state)?;

        peek_token!([
            TokenKind::Const => self.parse_classish_const(state, member_flags),
            TokenKind::Function => self.method(
                state,
                member_flags.iter().map(|t| t.clone().into()).collect(),
            )
        ], state, ["`const`", "`function`"])
    }

    pub(in crate::parser) fn enum_statement(
        &self,
        state: &mut State,
        backed: bool,
    ) -> ParseResult<Statement> {
        if state.current.kind == TokenKind::Case {
            state.next();

            let name = self.ident(state)?;

            if backed {
                expect_token!([TokenKind::Equals], state, "`=`");

                let value = self.expression(state, Precedence::Lowest)?;
                self.semi(state)?;

                return Ok(Statement::BackedEnumCase {
                    name: name.into(),
                    value,
                });
            } else {
                self.semi(state)?;

                return Ok(Statement::UnitEnumCase { name: name.into() });
            }
        }

        if state.current.kind == TokenKind::Const {
            return self.parse_classish_const(state, vec![]);
        }

        if state.current.kind == TokenKind::Function {
            return self.method(state, vec![]);
        }

        let member_flags = self.enum_members_flags(state)?;

        peek_token!([
            TokenKind::Const => self.parse_classish_const(state, member_flags),
            TokenKind::Function => self.method(
                state,
                member_flags.iter().map(|t| t.clone().into()).collect(),
            )
        ], state, ["`const`", "`function`"])
    }

    pub(in crate::parser) fn class_like_statement(
        &self,
        state: &mut State,
    ) -> ParseResult<Statement> {
        if state.current.kind == TokenKind::Use {
            return self.parse_classish_uses(state);
        }

        if state.current.kind == TokenKind::Var {
            return self.parse_classish_var(state);
        }

        if state.current.kind == TokenKind::Const {
            return self.parse_classish_const(state, vec![]);
        }

        if state.current.kind == TokenKind::Function {
            return self.method(state, vec![]);
        }

        let member_flags = self.class_members_flags(state)?;

        match &state.current.kind {
            TokenKind::Const => self.parse_classish_const(state, member_flags),
            TokenKind::Function => self.method(
                state,
                member_flags.iter().map(|t| t.clone().into()).collect(),
            ),
            // TODO
            TokenKind::Variable(_) => {
                let var = self.var(state)?;
                let mut value = None;

                if state.current.kind == TokenKind::Equals {
                    state.next();
                    value = Some(self.expression(state, Precedence::Lowest)?);
                }

                self.semi(state)?;

                Ok(Statement::Property {
                    var,
                    value,
                    r#type: None,
                    flags: member_flags.into_iter().map(|f| f.into()).collect(),
                })
            }
            TokenKind::Question
            | TokenKind::Identifier(_)
            | TokenKind::QualifiedIdentifier(_)
            | TokenKind::FullyQualifiedIdentifier(_)
            | TokenKind::Array
            | TokenKind::Null => {
                let prop_type = self.type_string(state)?;
                let var = self.var(state)?;
                let mut value = None;

                if state.current.kind == TokenKind::Equals {
                    state.next();
                    value = Some(self.expression(state, Precedence::Lowest)?);
                }

                // TODO: Support comma-separated property declarations.
                //       nikic/php-parser does this with a single Property statement
                //       that is capable of holding multiple property declarations.
                self.semi(state)?;

                Ok(Statement::Property {
                    var,
                    value,
                    r#type: Some(prop_type),
                    flags: member_flags.into_iter().map(|f| f.into()).collect(),
                })
            }
            _ => expected_token_err!(
                ["`const`", "`function`", "an identifier", "a varaible"],
                state
            ),
        }
    }

    fn parse_classish_var(&self, state: &mut State) -> ParseResult<Statement> {
        state.next();

        let mut var_type = None;

        if !matches!(state.current.kind, TokenKind::Variable(_)) {
            var_type = Some(self.type_string(state)?);
        }

        let var = self.var(state)?;
        let mut value = None;

        if state.current.kind == TokenKind::Equals {
            state.next();

            value = Some(self.expression(state, Precedence::Lowest)?);
        }

        self.semi(state)?;

        Ok(Statement::Var {
            var,
            value,
            r#type: var_type,
        })
    }

    fn parse_classish_uses(&self, state: &mut State) -> ParseResult<Statement> {
        state.next();

        let mut traits = Vec::new();

        while state.current.kind != TokenKind::SemiColon
            && state.current.kind != TokenKind::LeftBrace
        {
            self.optional_comma(state)?;

            let t = self.full_name(state)?;
            traits.push(t.into());
        }

        let mut adaptations = Vec::new();
        if state.current.kind == TokenKind::LeftBrace {
            self.lbrace(state)?;

            while state.current.kind != TokenKind::RightBrace {
                let (r#trait, method): (Option<Identifier>, Identifier) = match state.peek.kind {
                    TokenKind::DoubleColon => {
                        let r#trait = self.full_name(state)?;
                        state.next();
                        let method = self.ident(state)?;
                        (Some(r#trait.into()), method.into())
                    }
                    _ => (None, self.ident(state)?.into()),
                };

                match state.current.kind {
                    TokenKind::As => {
                        state.next();

                        match state.current.kind {
                            TokenKind::Public | TokenKind::Protected | TokenKind::Private => {
                                let visibility: MethodFlag = state.current.kind.clone().into();
                                state.next();

                                if state.current.kind == TokenKind::SemiColon {
                                    adaptations.push(TraitAdaptation::Visibility {
                                        r#trait,
                                        method,
                                        visibility,
                                    });
                                } else {
                                    let alias: Identifier = self.name(state)?.into();
                                    adaptations.push(TraitAdaptation::Alias {
                                        r#trait,
                                        method,
                                        alias,
                                        visibility: Some(visibility),
                                    });
                                }
                            }
                            _ => {
                                let alias: Identifier = self.name(state)?.into();
                                adaptations.push(TraitAdaptation::Alias {
                                    r#trait,
                                    method,
                                    alias,
                                    visibility: None,
                                });
                            }
                        }
                    }
                    TokenKind::Insteadof => {
                        state.next();

                        let mut insteadof = Vec::new();
                        insteadof.push(self.full_name(state)?.into());
                        while state.current.kind != TokenKind::SemiColon {
                            self.optional_comma(state)?;
                            insteadof.push(self.full_name(state)?.into());
                        }

                        adaptations.push(TraitAdaptation::Precedence {
                            r#trait,
                            method,
                            insteadof,
                        });
                    }
                    _ => {
                        return Err(ParseError::UnexpectedToken(
                            state.current.kind.to_string(),
                            state.current.span,
                        ))
                    }
                };

                self.semi(state)?;
            }

            self.rbrace(state)?;
        } else {
            self.semi(state)?;
        }

        Ok(Statement::TraitUse {
            traits,
            adaptations,
        })
    }

    fn parse_classish_const(
        &self,
        state: &mut State,
        const_flags: Vec<TokenKind>,
    ) -> ParseResult<Statement> {
        if const_flags.contains(&TokenKind::Static) {
            return Err(ParseError::StaticModifierOnConstant(state.current.span));
        }

        if const_flags.contains(&TokenKind::Readonly) {
            return Err(ParseError::ReadonlyModifierOnConstant(state.current.span));
        }

        if const_flags.contains(&TokenKind::Final) && const_flags.contains(&TokenKind::Private) {
            return Err(ParseError::FinalModifierOnPrivateConstant(
                state.current.span,
            ));
        }

        state.next();

        let name = self.ident(state)?;

        expect_token!([TokenKind::Equals], state, "`=`");

        let value = self.expression(state, Precedence::Lowest)?;

        self.semi(state)?;

        Ok(Statement::ClassishConstant {
            name: name.into(),
            value,
            flags: const_flags.into_iter().map(|f| f.into()).collect(),
        })
    }
}
