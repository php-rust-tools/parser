use crate::lexer::token::TokenKind;
use crate::parser::ast::identifier::Identifier;
use crate::parser::ast::BackedEnumType;
use crate::parser::ast::Block;
use crate::parser::ast::ClassFlag;
use crate::parser::ast::Expression;
use crate::parser::ast::Statement;
use crate::parser::error::ParseResult;
use crate::parser::state::Scope;
use crate::parser::state::State;
use crate::parser::Parser;

use crate::expect_token;
use crate::scoped;

impl Parser {
    pub(in crate::parser) fn class_definition(&self, state: &mut State) -> ParseResult<Statement> {
        let flags: Vec<ClassFlag> = self.class_flags(state)?.iter().map(|f| f.into()).collect();

        expect_token!([TokenKind::Class], state, ["`class`"]);

        let name = self.ident(state)?;

        let mut has_parent = false;
        let mut extends: Option<Identifier> = None;

        if state.current.kind == TokenKind::Extends {
            state.next();
            extends = Some(self.full_name(state)?);
            has_parent = true;
        }

        scoped!(
            state,
            Scope::Class(name.clone(), flags.clone(), has_parent),
            {
                let implements = if state.current.kind == TokenKind::Implements {
                    state.next();

                    self.at_least_one_comma_separated::<Identifier>(state, &|parser, state| {
                        parser.full_name(state)
                    })?
                } else {
                    Vec::new()
                };

                self.lbrace(state)?;

                let attributes = state.get_attributes();

                let mut body = Vec::new();
                while state.current.kind != TokenKind::RightBrace {
                    state.gather_comments();

                    if state.current.kind == TokenKind::RightBrace {
                        state.clear_comments();
                        break;
                    }

                    body.push(self.class_like_statement(state)?);
                }
                self.rbrace(state)?;

                Ok(Statement::Class {
                    name,
                    attributes,
                    extends,
                    implements,
                    body,
                    flags,
                })
            }
        )
    }

    pub(in crate::parser) fn interface_definition(
        &self,
        state: &mut State,
    ) -> ParseResult<Statement> {
        expect_token!([TokenKind::Interface], state, ["`interface`"]);
        let name = self.ident(state)?;

        scoped!(state, Scope::Interface(name.clone()), {
            let extends = if state.current.kind == TokenKind::Extends {
                state.next();

                self.at_least_one_comma_separated::<Identifier>(state, &|parser, state| {
                    parser.full_name(state)
                })?
            } else {
                Vec::new()
            };

            self.lbrace(state)?;

            let attributes = state.get_attributes();

            let mut body = Vec::new();
            while state.current.kind != TokenKind::RightBrace && !state.is_eof() {
                state.gather_comments();

                if state.current.kind == TokenKind::RightBrace {
                    state.clear_comments();
                    break;
                }

                body.push(self.interface_statement(state)?);
            }
            self.rbrace(state)?;

            Ok(Statement::Interface {
                name,
                attributes,
                extends,
                body,
            })
        })
    }

    pub(in crate::parser) fn trait_definition(&self, state: &mut State) -> ParseResult<Statement> {
        expect_token!([TokenKind::Trait], state, ["`trait`"]);

        let name = self.ident(state)?;

        scoped!(state, Scope::Trait(name.clone()), {
            self.lbrace(state)?;

            let attributes = state.get_attributes();

            let mut body = Vec::new();
            while state.current.kind != TokenKind::RightBrace && !state.is_eof() {
                state.gather_comments();

                if state.current.kind == TokenKind::RightBrace {
                    state.clear_comments();
                    break;
                }

                body.push(self.class_like_statement(state)?);
            }
            self.rbrace(state)?;

            Ok(Statement::Trait {
                name,
                attributes,
                body,
            })
        })
    }

    pub(in crate::parser) fn anonymous_class_definition(
        &self,
        state: &mut State,
    ) -> ParseResult<Expression> {
        expect_token!([TokenKind::New], state, ["`new`"]);

        self.gather_attributes(state)?;

        expect_token!([TokenKind::Class], state, ["`class`"]);

        let mut args = vec![];

        if state.current.kind == TokenKind::LeftParen {
            args = self.args_list(state)?;
        }

        let mut has_parent = false;
        let mut extends: Option<Identifier> = None;

        if state.current.kind == TokenKind::Extends {
            state.next();
            extends = Some(self.full_name(state)?);
            has_parent = true;
        }

        scoped!(state, Scope::AnonymousClass(has_parent), {
            let mut implements = Vec::new();
            if state.current.kind == TokenKind::Implements {
                state.next();

                while state.current.kind != TokenKind::LeftBrace {
                    implements.push(self.full_name(state)?);

                    if state.current.kind == TokenKind::Comma {
                        state.next();
                    } else {
                        break;
                    }
                }
            }

            self.lbrace(state)?;

            let attributes = state.get_attributes();

            let mut body = Vec::new();
            while state.current.kind != TokenKind::RightBrace && !state.is_eof() {
                body.push(self.class_like_statement(state)?);
            }

            self.rbrace(state)?;

            Ok(Expression::New {
                target: Box::new(Expression::AnonymousClass {
                    attributes,
                    extends,
                    implements,
                    body,
                }),
                args,
            })
        })
    }

    pub(in crate::parser) fn enum_definition(&self, state: &mut State) -> ParseResult<Statement> {
        expect_token!([TokenKind::Enum], state, ["`enum`"]);

        let name = self.ident(state)?;

        let backed_type: Option<BackedEnumType> = if state.current.kind == TokenKind::Colon {
            self.colon(state)?;

            expect_token!([
                TokenKind::Identifier(s) if s == b"string" || s == b"int" => {
                    Some(match &s[..] {
                        b"string" => BackedEnumType::String,
                        b"int" => BackedEnumType::Int,
                        _ => unreachable!(),
                    })
                },
            ], state, ["`string`", "`int`",])
        } else {
            None
        };

        scoped!(state, Scope::Enum(name.clone(), backed_type.is_some()), {
            let mut implements = Vec::new();
            if state.current.kind == TokenKind::Implements {
                state.next();

                while state.current.kind != TokenKind::LeftBrace {
                    implements.push(self.full_name(state)?);

                    if state.current.kind == TokenKind::Comma {
                        state.next();
                    } else {
                        break;
                    }
                }
            }

            let attributes = state.get_attributes();

            self.lbrace(state)?;

            let mut body = Block::new();
            while state.current.kind != TokenKind::RightBrace {
                state.skip_comments();
                body.push(self.enum_statement(state)?);
            }

            self.rbrace(state)?;

            match backed_type {
                Some(backed_type) => Ok(Statement::BackedEnum {
                    name,
                    attributes,
                    backed_type,
                    implements,
                    body,
                }),
                None => Ok(Statement::UnitEnum {
                    name,
                    attributes,
                    implements,
                    body,
                }),
            }
        })
    }

    fn at_least_one_comma_separated<T>(
        &self,
        state: &mut State,
        func: &(dyn Fn(&Parser, &mut State) -> ParseResult<T>),
    ) -> ParseResult<Vec<T>> {
        let mut result: Vec<T> = vec![];
        loop {
            result.push(func(self, state)?);
            if state.current.kind != TokenKind::Comma {
                break;
            }

            state.next();
        }

        Ok(result)
    }
}
