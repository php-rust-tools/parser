use crate::lexer::token::TokenKind;
use crate::parser::ast::ClassFlag;
use crate::parser::ast::Identifier;
use crate::parser::ast::MethodFlag;
use crate::parser::ast::Statement;
use crate::parser::ast::TraitAdaptation;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::precedence::Precedence;
use crate::parser::Parser;

use crate::expect_token;
use crate::expected_token_err;
use crate::peek_token;

#[derive(Debug)]
pub enum ClassishDefinitionType {
    Class(Vec<ClassFlag>),
    AnonymousClass,
    Trait,
    Interface,
    Enum,
}

impl Parser {
    pub(in crate::parser) fn class_statement(
        &mut self,
        flags: Vec<ClassFlag>,
    ) -> ParseResult<Statement> {
        self.complete_class_statement(ClassishDefinitionType::Class(flags))
    }

    pub(in crate::parser) fn interface_statement(&mut self) -> ParseResult<Statement> {
        if self.current.kind == TokenKind::Const {
            return self.parse_classish_const(vec![]);
        }

        if self.current.kind == TokenKind::Function {
            return self.method(ClassishDefinitionType::Interface, vec![]);
        }

        let member_flags = self.interface_members_flags()?;

        peek_token!([
            TokenKind::Const => self.parse_classish_const(member_flags),
            TokenKind::Function => self.method(
                ClassishDefinitionType::Interface,
                member_flags.iter().map(|t| t.clone().into()).collect(),
            )
        ], self, ["`const`", "`function`"])
    }

    pub(in crate::parser) fn trait_statement(&mut self) -> ParseResult<Statement> {
        self.complete_class_statement(ClassishDefinitionType::Trait)
    }

    pub(in crate::parser) fn anonymous_class_statement(&mut self) -> ParseResult<Statement> {
        self.complete_class_statement(ClassishDefinitionType::AnonymousClass)
    }

    pub(in crate::parser) fn enum_statement(&mut self, backed: bool) -> ParseResult<Statement> {
        if self.current.kind == TokenKind::Case {
            self.next();

            let name = self.ident()?;

            if backed {
                expect_token!([TokenKind::Equals], self, "`=`");

                let value = self.expression(Precedence::Lowest)?;
                self.semi()?;

                return Ok(Statement::BackedEnumCase {
                    name: name.into(),
                    value,
                });
            } else {
                self.semi()?;

                return Ok(Statement::UnitEnumCase { name: name.into() });
            }
        }

        if self.current.kind == TokenKind::Const {
            return self.parse_classish_const(vec![]);
        }

        if self.current.kind == TokenKind::Function {
            return self.method(ClassishDefinitionType::Enum, vec![]);
        }

        let member_flags = self.enum_members_flags()?;

        peek_token!([
            TokenKind::Const => self.parse_classish_const(member_flags),
            TokenKind::Function => self.method(
                ClassishDefinitionType::Enum,
                member_flags.iter().map(|t| t.clone().into()).collect(),
            )
        ], self, ["`const`", "`function`"])
    }

    fn complete_class_statement(
        &mut self,
        class_type: ClassishDefinitionType,
    ) -> ParseResult<Statement> {
        if self.current.kind == TokenKind::Use {
            return self.parse_classish_uses();
        }

        if self.current.kind == TokenKind::Var {
            return self.parse_classish_var();
        }

        if self.current.kind == TokenKind::Const {
            return self.parse_classish_const(vec![]);
        }

        if self.current.kind == TokenKind::Function {
            return self.method(class_type, vec![]);
        }

        let member_flags = self.class_members_flags()?;

        match &self.current.kind {
            TokenKind::Const => self.parse_classish_const(member_flags),
            TokenKind::Function => self.method(
                class_type,
                member_flags.iter().map(|t| t.clone().into()).collect(),
            ),
            // TODO
            TokenKind::Variable(_) => {
                let var = self.var()?;
                let mut value = None;

                if self.current.kind == TokenKind::Equals {
                    self.next();
                    value = Some(self.expression(Precedence::Lowest)?);
                }

                self.semi()?;

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
                let prop_type = self.type_string()?;
                let var = self.var()?;
                let mut value = None;

                if self.current.kind == TokenKind::Equals {
                    self.next();
                    value = Some(self.expression(Precedence::Lowest)?);
                }

                // TODO: Support comma-separated property declarations.
                //       nikic/php-parser does this with a single Property statement
                //       that is capable of holding multiple property declarations.
                self.semi()?;

                Ok(Statement::Property {
                    var,
                    value,
                    r#type: Some(prop_type),
                    flags: member_flags.into_iter().map(|f| f.into()).collect(),
                })
            }
            _ => expected_token_err!(
                ["`const`", "`function`", "an identifier", "a varaible"],
                self
            ),
        }
    }

    fn parse_classish_var(&mut self) -> ParseResult<Statement> {
        self.next();

        let mut var_type = None;

        if !matches!(self.current.kind, TokenKind::Variable(_)) || self.config.force_type_strings {
            var_type = Some(self.type_string()?);
        }

        let var = self.var()?;
        let mut value = None;

        if self.current.kind == TokenKind::Equals {
            self.next();

            value = Some(self.expression(Precedence::Lowest)?);
        }

        self.semi()?;

        Ok(Statement::Var {
            var,
            value,
            r#type: var_type,
        })
    }

    fn parse_classish_uses(&mut self) -> ParseResult<Statement> {
        self.next();

        let mut traits = Vec::new();

        while self.current.kind != TokenKind::SemiColon && self.current.kind != TokenKind::LeftBrace
        {
            self.optional_comma()?;

            let t = self.full_name()?;
            traits.push(t.into());
        }

        let mut adaptations = Vec::new();
        if self.current.kind == TokenKind::LeftBrace {
            self.lbrace()?;

            while self.current.kind != TokenKind::RightBrace {
                let (r#trait, method): (Option<Identifier>, Identifier) = match self.peek.kind {
                    TokenKind::DoubleColon => {
                        let r#trait = self.full_name()?;
                        self.next();
                        let method = self.ident()?;
                        (Some(r#trait.into()), method.into())
                    }
                    _ => (None, self.ident()?.into()),
                };

                match self.current.kind {
                    TokenKind::As => {
                        self.next();

                        match self.current.kind {
                            TokenKind::Public | TokenKind::Protected | TokenKind::Private => {
                                let visibility: MethodFlag = self.current.kind.clone().into();
                                self.next();

                                if self.current.kind == TokenKind::SemiColon {
                                    adaptations.push(TraitAdaptation::Visibility {
                                        r#trait,
                                        method,
                                        visibility,
                                    });
                                } else {
                                    let alias: Identifier = self.name()?.into();
                                    adaptations.push(TraitAdaptation::Alias {
                                        r#trait,
                                        method,
                                        alias,
                                        visibility: Some(visibility),
                                    });
                                }
                            }
                            _ => {
                                let alias: Identifier = self.name()?.into();
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
                        self.next();

                        let mut insteadof = Vec::new();
                        insteadof.push(self.full_name()?.into());
                        while self.current.kind != TokenKind::SemiColon {
                            self.optional_comma()?;
                            insteadof.push(self.full_name()?.into());
                        }

                        adaptations.push(TraitAdaptation::Precedence {
                            r#trait,
                            method,
                            insteadof,
                        });
                    }
                    _ => {
                        return Err(ParseError::UnexpectedToken(
                            self.current.kind.to_string(),
                            self.current.span,
                        ))
                    }
                };

                self.semi()?;
            }

            self.rbrace()?;
        } else {
            self.semi()?;
        }

        Ok(Statement::TraitUse {
            traits,
            adaptations,
        })
    }

    fn parse_classish_const(&mut self, const_flags: Vec<TokenKind>) -> ParseResult<Statement> {
        if const_flags.contains(&TokenKind::Static) {
            return Err(ParseError::StaticModifierOnConstant(self.current.span));
        }

        if const_flags.contains(&TokenKind::Readonly) {
            return Err(ParseError::ReadonlyModifierOnConstant(self.current.span));
        }

        if const_flags.contains(&TokenKind::Final) && const_flags.contains(&TokenKind::Private) {
            return Err(ParseError::FinalModifierOnPrivateConstant(
                self.current.span,
            ));
        }

        self.next();

        let name = self.ident()?;

        expect_token!([TokenKind::Equals], self, "`=`");

        let value = self.expression(Precedence::Lowest)?;

        self.semi()?;

        Ok(Statement::ClassishConstant {
            name: name.into(),
            value,
            flags: const_flags.into_iter().map(|f| f.into()).collect(),
        })
    }
}
