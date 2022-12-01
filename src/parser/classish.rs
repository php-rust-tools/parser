use crate::lexer::token::TokenKind;
use crate::parser::ast::BackedEnumType;
use crate::parser::ast::Block;
use crate::parser::ast::ClassFlag;
use crate::parser::ast::Expression;
use crate::parser::ast::Identifier;
use crate::parser::ast::Statement;
use crate::parser::error::ParseResult;
use crate::parser::state::State;
use crate::parser::Parser;

use crate::expect_token;
use crate::expected_token_err;

impl Parser {
    pub(in crate::parser) fn class_definition(&self, state: &mut State) -> ParseResult<Statement> {
        let flags: Vec<ClassFlag> = self.class_flags(state)?.iter().map(|f| f.into()).collect();

        expect_token!([TokenKind::Class], state, ["`class`"]);

        let name = self.ident(state)?;
        let mut extends: Option<Identifier> = None;

        if state.current.kind == TokenKind::Extends {
            state.next();
            extends = Some(self.full_name(state)?.into());
        }

        let implements = if state.current.kind == TokenKind::Implements {
            state.next();

            self.at_least_one_comma_separated::<Identifier>(state, &|parser, state| {
                Ok(parser.full_name(state)?.into())
            })?
        } else {
            Vec::new()
        };

        self.lbrace(state)?;

        let mut body = Vec::new();
        while state.current.kind != TokenKind::RightBrace {
            state.gather_comments();

            if state.current.kind == TokenKind::RightBrace {
                state.clear_comments();
                break;
            }

            body.push(self.class_statement(state, flags.clone())?);
        }
        self.rbrace(state)?;

        Ok(Statement::Class {
            name: name.into(),
            extends,
            implements,
            body,
            flags,
        })
    }

    pub(in crate::parser) fn interface_definition(
        &self,
        state: &mut State,
    ) -> ParseResult<Statement> {
        expect_token!([TokenKind::Interface], state, ["`interface`"]);

        let name = self.ident(state)?;

        let extends = if state.current.kind == TokenKind::Extends {
            state.next();

            self.at_least_one_comma_separated::<Identifier>(state, &|parser, state| {
                Ok(parser.full_name(state)?.into())
            })?
        } else {
            Vec::new()
        };

        self.lbrace(state)?;

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
            name: name.into(),
            extends,
            body,
        })
    }

    pub(in crate::parser) fn trait_definition(&self, state: &mut State) -> ParseResult<Statement> {
        expect_token!([TokenKind::Trait], state, ["`trait`"]);

        let name = self.ident(state)?;

        self.lbrace(state)?;

        let mut body = Vec::new();
        while state.current.kind != TokenKind::RightBrace && !state.is_eof() {
            state.gather_comments();

            if state.current.kind == TokenKind::RightBrace {
                state.clear_comments();
                break;
            }

            body.push(self.trait_statement(state)?);
        }
        self.rbrace(state)?;

        Ok(Statement::Trait {
            name: name.into(),
            body,
        })
    }

    pub(in crate::parser) fn anonymous_class_definition(
        &self,
        state: &mut State,
    ) -> ParseResult<Expression> {
        state.next();

        expect_token!([TokenKind::Class], state, ["`class`"]);

        let mut args = vec![];

        if state.current.kind == TokenKind::LeftParen {
            self.lparen(state)?;

            args = self.args_list(state)?;

            self.rparen(state)?;
        }

        let mut extends: Option<Identifier> = None;

        if state.current.kind == TokenKind::Extends {
            state.next();
            extends = Some(self.full_name(state)?.into());
        }

        let mut implements = Vec::new();
        if state.current.kind == TokenKind::Implements {
            state.next();

            while state.current.kind != TokenKind::LeftBrace {
                self.optional_comma(state)?;

                implements.push(self.full_name(state)?.into());
            }
        }

        self.lbrace(state)?;

        let mut body = Vec::new();
        while state.current.kind != TokenKind::RightBrace && !state.is_eof() {
            body.push(self.anonymous_class_statement(state)?);
        }

        self.rbrace(state)?;

        Ok(Expression::New {
            target: Box::new(Expression::AnonymousClass {
                extends,
                implements,
                body,
            }),
            args,
        })
    }

    pub(in crate::parser) fn enum_definition(&self, state: &mut State) -> ParseResult<Statement> {
        state.next();

        let name = self.ident(state)?;

        let backed_type: Option<BackedEnumType> = if state.current.kind == TokenKind::Colon {
            self.colon(state)?;

            match state.current.kind.clone() {
                TokenKind::Identifier(s) if s == b"string" || s == b"int" => {
                    state.next();

                    Some(match &s[..] {
                        b"string" => BackedEnumType::String,
                        b"int" => BackedEnumType::Int,
                        _ => unreachable!(),
                    })
                }
                _ => {
                    return expected_token_err!(["`string`", "`int`"], state);
                }
            }
        } else {
            None
        };

        let mut implements = Vec::new();
        if state.current.kind == TokenKind::Implements {
            state.next();

            while state.current.kind != TokenKind::LeftBrace {
                implements.push(self.full_name(state)?.into());

                self.optional_comma(state)?;
            }
        }

        self.lbrace(state)?;

        let mut body = Block::new();
        while state.current.kind != TokenKind::RightBrace {
            state.skip_comments();
            body.push(self.enum_statement(state, backed_type.is_some())?);
        }

        self.rbrace(state)?;

        match backed_type {
            Some(backed_type) => Ok(Statement::BackedEnum {
                name: name.into(),
                backed_type,
                implements,
                body,
            }),
            None => Ok(Statement::UnitEnum {
                name: name.into(),
                implements,
                body,
            }),
        }
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
