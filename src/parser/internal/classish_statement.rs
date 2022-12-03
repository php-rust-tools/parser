use crate::expected_scope;
use crate::lexer::token::TokenKind;
use crate::parser::ast::Identifier;
use crate::parser::ast::MethodFlag;
use crate::parser::ast::PropertyFlag;
use crate::parser::ast::Statement;
use crate::parser::ast::TraitAdaptation;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::internal::precedence::Precedence;
use crate::parser::state::Scope;
use crate::parser::state::State;
use crate::parser::Parser;

use crate::expect_token;
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

    pub(in crate::parser) fn enum_statement(&self, state: &mut State) -> ParseResult<Statement> {
        let (enum_name, backed) = expected_scope!([
            Scope::Enum(enum_name, backed) => (enum_name, backed),
        ], state);

        if state.current.kind == TokenKind::Case {
            state.next();

            let name = self.ident(state)?;

            if backed {
                if state.current.kind == TokenKind::SemiColon {
                    return Err(ParseError::MissingCaseValueForBackedEnum(
                        name.to_string(),
                        state.named(&enum_name),
                        state.current.span,
                    ));
                }

                expect_token!([TokenKind::Equals], state, "`=`");

                let value = self.expression(state, Precedence::Lowest)?;
                self.semi(state)?;

                return Ok(Statement::BackedEnumCase {
                    name: name.into(),
                    value,
                });
            } else {
                if state.current.kind == TokenKind::Equals {
                    return Err(ParseError::CaseValueForUnitEnum(
                        name.to_string(),
                        state.named(&enum_name),
                        state.current.span,
                    ));
                }

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

        if state.current.kind == TokenKind::Const {
            return self.parse_classish_const(state, member_flags);
        }

        if state.current.kind == TokenKind::Function {
            return self.method(
                state,
                member_flags.iter().map(|t| t.clone().into()).collect(),
            );
        }

        let ty = self.get_optional_type(state)?;

        expect_token!([
            TokenKind::Variable(var) => {
                let flags: Vec<PropertyFlag> = member_flags.into_iter().map(|f| f.into()).collect();
                let mut value = None;

                if state.current.kind == TokenKind::Equals {
                    state.next();
                    value = Some(self.expression(state, Precedence::Lowest)?);
                }

                let class_name: String = expected_scope!([
                    Scope::Trait(name) | Scope::Class(name, _, _) => state.named(&name),
                    Scope::AnonymousClass(_) => state.named(&"class@anonymous".into()),
                ], state);

                if flags.contains(&PropertyFlag::Readonly) {
                    if flags.contains(&PropertyFlag::Static) {
                        return Err(ParseError::StaticPropertyUsingReadonlyModifier(class_name, var.to_string(), state.current.span));
                    }

                    if value.is_some() {
                        return Err(ParseError::ReadonlyPropertyHasDefaultValue(class_name, var.to_string(), state.current.span));
                    }
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
                        if flags.contains(&PropertyFlag::Readonly) {
                            return Err(ParseError::MissingTypeForReadonlyProperty(
                                class_name,
                                var.to_string(),
                                state.current.span,
                            ));
                        }
                    }
                }

                self.semi(state)?;

                Ok(Statement::Property {
                    var,
                    value,
                    r#type: ty,
                    flags,
                })
            }
        ], state, ["a varaible"])
    }

    fn parse_classish_var(&self, state: &mut State) -> ParseResult<Statement> {
        state.next();

        let ty = self.get_optional_type(state)?;
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
            r#type: ty,
        })
    }

    fn parse_classish_uses(&self, state: &mut State) -> ParseResult<Statement> {
        state.next();

        let mut traits = Vec::new();

        while state.current.kind != TokenKind::SemiColon
            && state.current.kind != TokenKind::LeftBrace
        {
            let t = self.full_name(state)?;
            traits.push(t.into());

            if state.current.kind == TokenKind::Comma {
                if state.peek.kind == TokenKind::SemiColon {
                    // will fail with unexpected token `,`
                    // as `use` doesn't allow for trailing commas.
                    self.semi(state)?;
                } else if state.peek.kind == TokenKind::LeftBrace {
                    // will fail with unexpected token `{`
                    // as `use` doesn't allow for trailing commas.
                    self.lbrace(state)?;
                } else {
                    state.next();
                }
            } else {
                break;
            }
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

                expect_token!([
                    TokenKind::As => {
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
                    },
                    TokenKind::Insteadof => {
                        let mut insteadof = Vec::new();
                        insteadof.push(self.full_name(state)?.into());

                        if state.current.kind == TokenKind::Comma {
                            if state.peek.kind == TokenKind::SemiColon {
                                // will fail with unexpected token `,`
                                // as `insteadof` doesn't allow for trailing commas.
                                self.semi(state)?;
                            }

                            state.next();

                            while state.current.kind != TokenKind::SemiColon {
                                insteadof.push(self.full_name(state)?.into());

                                if state.current.kind == TokenKind::Comma {
                                    if state.peek.kind == TokenKind::SemiColon {
                                        // will fail with unexpected token `,`
                                        // as `insteadof` doesn't allow for trailing commas.
                                        self.semi(state)?;
                                    } else {
                                        state.next();
                                    }
                                } else {
                                    break;
                                }
                            }
                        }

                        adaptations.push(TraitAdaptation::Precedence {
                            r#trait,
                            method,
                            insteadof,
                        });
                    }
                ], state, ["`as`", "`insteadof`"]);

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
