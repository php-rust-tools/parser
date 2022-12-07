use crate::expect_token;
use crate::expected_scope;
use crate::lexer::token::Span;
use crate::lexer::token::TokenKind;
use crate::parser::ast::classish::ClassishConstant;
use crate::parser::ast::enums::BackedEnumCase;
use crate::parser::ast::enums::BackedEnumMember;
use crate::parser::ast::enums::UnitEnumCase;
use crate::parser::ast::enums::UnitEnumMember;
use crate::parser::ast::identifiers::Identifier;
use crate::parser::ast::modifiers::ConstantModifierGroup;
use crate::parser::ast::modifiers::VisibilityModifier;
use crate::parser::ast::Statement;
use crate::parser::ast::TraitAdaptation;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::internal::data_type;
use crate::parser::internal::identifiers;
use crate::parser::internal::modifiers;
use crate::parser::internal::precedences::Precedence;
use crate::parser::internal::utils;
use crate::parser::state::Scope;
use crate::parser::state::State;
use crate::parser::Parser;
use crate::peek_token;

impl Parser {
    pub(in crate::parser) fn interface_statement(
        &self,
        state: &mut State,
    ) -> ParseResult<Statement> {
        let has_attributes = self.gather_attributes(state)?;
        let start = state.current.span;
        let modifiers = modifiers::collect(state)?;

        // if we have attributes, don't check const, we need a method.
        if has_attributes || state.current.kind == TokenKind::Function {
            Ok(Statement::Method(self.method(
                state,
                modifiers::interface_method_group(modifiers)?,
                start,
            )?))
        } else {
            Ok(Statement::ClassishConstant(self.constant(
                state,
                modifiers::interface_constant_group(modifiers)?,
                start,
            )?))
        }
    }

    pub(in crate::parser) fn unit_enum_member(
        &self,
        state: &mut State,
    ) -> ParseResult<UnitEnumMember> {
        let enum_name = expected_scope!([
            Scope::Enum(enum_name, _) => enum_name,
        ], state);

        let has_attributes = self.gather_attributes(state)?;

        if !has_attributes && state.current.kind == TokenKind::Case {
            let start = state.current.span;
            state.next();

            let name = identifiers::ident(state)?;

            if state.current.kind == TokenKind::Equals {
                return Err(ParseError::CaseValueForUnitEnum(
                    name.to_string(),
                    state.named(&enum_name),
                    state.current.span,
                ));
            }

            let end = utils::skip_semicolon(state)?;

            return Ok(UnitEnumMember::Case(UnitEnumCase { start, end, name }));
        }

        let start = state.current.span;
        let modifiers = modifiers::collect(state)?;

        // if we have attributes, don't check const, we need a method.
        if has_attributes || state.current.kind == TokenKind::Function {
            Ok(UnitEnumMember::Method(self.method(
                state,
                modifiers::enum_method_group(modifiers)?,
                start,
            )?))
        } else {
            Ok(UnitEnumMember::Constant(self.constant(
                state,
                modifiers::constant_group(modifiers)?,
                start,
            )?))
        }
    }

    pub(in crate::parser) fn backed_enum_member(
        &self,
        state: &mut State,
    ) -> ParseResult<BackedEnumMember> {
        let enum_name = expected_scope!([
            Scope::Enum(enum_name, _) => enum_name,
        ], state);

        let has_attributes = self.gather_attributes(state)?;

        if !has_attributes && state.current.kind == TokenKind::Case {
            let start = state.current.span;
            state.next();

            let name = identifiers::ident(state)?;

            if state.current.kind == TokenKind::SemiColon {
                return Err(ParseError::MissingCaseValueForBackedEnum(
                    name.to_string(),
                    state.named(&enum_name),
                    state.current.span,
                ));
            }

            utils::skip(state, TokenKind::Equals)?;

            let value = self.expression(state, Precedence::Lowest)?;

            let end = utils::skip_semicolon(state)?;

            return Ok(BackedEnumMember::Case(BackedEnumCase {
                start,
                end,
                name,
                value,
            }));
        }

        let start = state.current.span;
        let modifiers = modifiers::collect(state)?;

        // if we have attributes, don't check const, we need a method.
        if has_attributes || state.current.kind == TokenKind::Function {
            Ok(BackedEnumMember::Method(self.method(
                state,
                modifiers::enum_method_group(modifiers)?,
                start,
            )?))
        } else {
            Ok(BackedEnumMember::Constant(self.constant(
                state,
                modifiers::constant_group(modifiers)?,
                start,
            )?))
        }
    }

    pub(in crate::parser) fn class_like_statement(
        &self,
        state: &mut State,
    ) -> ParseResult<Statement> {
        let has_attributes = self.gather_attributes(state)?;

        let start = state.current.span;
        let modifiers = modifiers::collect(state)?;

        if !has_attributes {
            if state.current.kind == TokenKind::Use {
                return self.parse_classish_uses(state);
            }

            if state.current.kind == TokenKind::Const {
                return Ok(Statement::ClassishConstant(self.constant(
                    state,
                    modifiers::constant_group(modifiers)?,
                    start,
                )?));
            }
        }

        if state.current.kind == TokenKind::Function {
            return Ok(Statement::Method(self.method(
                state,
                modifiers::method_group(modifiers)?,
                start,
            )?));
        }

        // e.g: public static
        let modifiers = modifiers::property_group(modifiers)?;
        // e.g: string
        let ty = data_type::optional_data_type(state)?;
        // e.g: $name
        let var = identifiers::var(state)?;

        let mut value = None;
        // e.g: = "foo";
        if state.current.kind == TokenKind::Equals {
            state.next();
            value = Some(self.expression(state, Precedence::Lowest)?);
        }

        let class_name: String = expected_scope!([
            Scope::Trait(name) | Scope::Class(name, _, _) => state.named(&name),
            Scope::AnonymousClass(_) => state.named("class@anonymous"),
        ], state);

        if modifiers.has_readonly() {
            if modifiers.has_static() {
                return Err(ParseError::StaticPropertyUsingReadonlyModifier(
                    class_name,
                    var.to_string(),
                    state.current.span,
                ));
            }

            if value.is_some() {
                return Err(ParseError::ReadonlyPropertyHasDefaultValue(
                    class_name,
                    var.to_string(),
                    state.current.span,
                ));
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
                if modifiers.has_readonly() {
                    return Err(ParseError::MissingTypeForReadonlyProperty(
                        class_name,
                        var.to_string(),
                        state.current.span,
                    ));
                }
            }
        }

        utils::skip_semicolon(state)?;

        Ok(Statement::Property {
            var,
            value,
            r#type: ty,
            modifiers,
            attributes: state.get_attributes(),
        })
    }

    fn parse_classish_uses(&self, state: &mut State) -> ParseResult<Statement> {
        state.next();

        let mut traits = Vec::new();

        while state.current.kind != TokenKind::SemiColon
            && state.current.kind != TokenKind::LeftBrace
        {
            let t = identifiers::full_name(state)?;
            traits.push(t);

            if state.current.kind == TokenKind::Comma {
                if state.peek.kind == TokenKind::SemiColon {
                    // will fail with unexpected token `,`
                    // as `use` doesn't allow for trailing commas.
                    utils::skip_semicolon(state)?;
                } else if state.peek.kind == TokenKind::LeftBrace {
                    // will fail with unexpected token `{`
                    // as `use` doesn't allow for trailing commas.
                    utils::skip_left_brace(state)?;
                } else {
                    state.next();
                }
            } else {
                break;
            }
        }

        let mut adaptations = Vec::new();
        if state.current.kind == TokenKind::LeftBrace {
            utils::skip_left_brace(state)?;

            while state.current.kind != TokenKind::RightBrace {
                let (r#trait, method): (Option<Identifier>, Identifier) = match state.peek.kind {
                    TokenKind::DoubleColon => {
                        let r#trait = identifiers::full_name(state)?;
                        state.next();
                        let method = identifiers::ident(state)?;
                        (Some(r#trait), method)
                    }
                    _ => (None, identifiers::ident(state)?),
                };

                expect_token!([
                    TokenKind::As => {
                        match state.current.kind {
                            TokenKind::Public | TokenKind::Protected | TokenKind::Private => {
                                let visibility = peek_token!([
                                    TokenKind::Public => VisibilityModifier::Public {
                                        start: state.current.span,
                                        end: state.peek.span
                                    },
                                    TokenKind::Protected => VisibilityModifier::Protected {
                                        start: state.current.span,
                                        end: state.peek.span
                                    },
                                    TokenKind::Private => VisibilityModifier::Private {
                                        start: state.current.span,
                                        end: state.peek.span
                                    },
                                ], state, ["`private`", "`protected`", "`public`"]);
                                state.next();

                                if state.current.kind == TokenKind::SemiColon {
                                    adaptations.push(TraitAdaptation::Visibility {
                                        r#trait,
                                        method,
                                        visibility,
                                    });
                                } else {
                                    let alias: Identifier = identifiers::name(state)?;
                                    adaptations.push(TraitAdaptation::Alias {
                                        r#trait,
                                        method,
                                        alias,
                                        visibility: Some(visibility),
                                    });
                                }
                            }
                            _ => {
                                let alias: Identifier = identifiers::name(state)?;
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
                        insteadof.push(identifiers::full_name(state)?);

                        if state.current.kind == TokenKind::Comma {
                            if state.peek.kind == TokenKind::SemiColon {
                                // will fail with unexpected token `,`
                                // as `insteadof` doesn't allow for trailing commas.
                                utils::skip_semicolon(state)?;
                            }

                            state.next();

                            while state.current.kind != TokenKind::SemiColon {
                                insteadof.push(identifiers::full_name(state)?);

                                if state.current.kind == TokenKind::Comma {
                                    if state.peek.kind == TokenKind::SemiColon {
                                        // will fail with unexpected token `,`
                                        // as `insteadof` doesn't allow for trailing commas.
                                        utils::skip_semicolon(state)?;
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

                utils::skip_semicolon(state)?;
            }

            utils::skip_right_brace(state)?;
        } else {
            utils::skip_semicolon(state)?;
        }

        Ok(Statement::TraitUse {
            traits,
            adaptations,
        })
    }

    fn constant(
        &self,
        state: &mut State,
        modifiers: ConstantModifierGroup,
        start: Span,
    ) -> ParseResult<ClassishConstant> {
        state.next();

        let name = identifiers::ident(state)?;

        utils::skip(state, TokenKind::Equals)?;

        let value = self.expression(state, Precedence::Lowest)?;

        let end = utils::skip_semicolon(state)?;

        Ok(ClassishConstant {
            start,
            end,
            name,
            value,
            modifiers,
        })
    }
}
