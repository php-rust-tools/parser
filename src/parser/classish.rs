use crate::lexer::token::TokenKind;
use crate::parser::ast::BackedEnumType;
use crate::parser::ast::Block;
use crate::parser::ast::ClassFlag;
use crate::parser::ast::Expression;
use crate::parser::ast::Identifier;
use crate::parser::ast::Statement;
use crate::parser::error::ParseResult;
use crate::parser::Parser;

use crate::expect_token;
use crate::expected_token_err;

impl Parser {
    pub(in crate::parser) fn class_definition(&mut self) -> ParseResult<Statement> {
        let flags: Vec<ClassFlag> = self.class_flags()?.iter().map(|f| f.into()).collect();

        expect_token!([TokenKind::Class], self, ["`class`"]);

        let name = self.ident()?;
        let mut extends: Option<Identifier> = None;

        if self.current.kind == TokenKind::Extends {
            self.next();
            extends = Some(self.full_name()?.into());
        }

        let implements = if self.current.kind == TokenKind::Implements {
            self.next();

            self.at_least_one_comma_separated::<Identifier>(&|parser| {
                Ok(parser.full_name()?.into())
            })?
        } else {
            Vec::new()
        };

        self.lbrace()?;

        let mut body = Vec::new();
        while self.current.kind != TokenKind::RightBrace {
            self.gather_comments();

            if self.current.kind == TokenKind::RightBrace {
                self.clear_comments();
                break;
            }

            body.push(self.class_statement(flags.clone())?);
        }
        self.rbrace()?;

        Ok(Statement::Class {
            name: name.into(),
            extends,
            implements,
            body,
            flags,
        })
    }

    pub(in crate::parser) fn interface_definition(&mut self) -> ParseResult<Statement> {
        expect_token!([TokenKind::Interface], self, ["`interface`"]);

        let name = self.ident()?;

        let extends = if self.current.kind == TokenKind::Extends {
            self.next();

            self.at_least_one_comma_separated::<Identifier>(&|parser| {
                Ok(parser.full_name()?.into())
            })?
        } else {
            Vec::new()
        };

        self.lbrace()?;

        let mut body = Vec::new();
        while self.current.kind != TokenKind::RightBrace && !self.is_eof() {
            self.gather_comments();

            if self.current.kind == TokenKind::RightBrace {
                self.clear_comments();
                break;
            }

            body.push(self.interface_statement()?);
        }
        self.rbrace()?;

        Ok(Statement::Interface {
            name: name.into(),
            extends,
            body,
        })
    }

    pub(in crate::parser) fn trait_definition(&mut self) -> ParseResult<Statement> {
        expect_token!([TokenKind::Trait], self, ["`trait`"]);

        let name = self.ident()?;

        self.lbrace()?;

        let mut body = Vec::new();
        while self.current.kind != TokenKind::RightBrace && !self.is_eof() {
            self.gather_comments();

            if self.current.kind == TokenKind::RightBrace {
                self.clear_comments();
                break;
            }

            body.push(self.trait_statement()?);
        }
        self.rbrace()?;

        Ok(Statement::Trait {
            name: name.into(),
            body,
        })
    }

    pub(in crate::parser) fn anonymous_class_definition(&mut self) -> ParseResult<Expression> {
        self.next();

        expect_token!([TokenKind::Class], self, ["`class`"]);

        let mut args = vec![];

        if self.current.kind == TokenKind::LeftParen {
            self.lparen()?;

            args = self.args_list()?;

            self.rparen()?;
        }

        let mut extends: Option<Identifier> = None;

        if self.current.kind == TokenKind::Extends {
            self.next();
            extends = Some(self.full_name()?.into());
        }

        let mut implements = Vec::new();
        if self.current.kind == TokenKind::Implements {
            self.next();

            while self.current.kind != TokenKind::LeftBrace {
                self.optional_comma()?;

                implements.push(self.full_name()?.into());
            }
        }

        self.lbrace()?;

        let mut body = Vec::new();
        while self.current.kind != TokenKind::RightBrace && !self.is_eof() {
            body.push(self.anonymous_class_statement()?);
        }

        self.rbrace()?;

        Ok(Expression::New {
            target: Box::new(Expression::AnonymousClass {
                extends,
                implements,
                body,
            }),
            args,
        })
    }

    pub(in crate::parser) fn enum_definition(&mut self) -> ParseResult<Statement> {
        self.next();

        let name = self.ident()?;

        let backed_type: Option<BackedEnumType> = if self.current.kind == TokenKind::Colon {
            self.colon()?;

            match self.current.kind.clone() {
                TokenKind::Identifier(s) if s == b"string" || s == b"int" => {
                    self.next();

                    Some(match &s[..] {
                        b"string" => BackedEnumType::String,
                        b"int" => BackedEnumType::Int,
                        _ => unreachable!(),
                    })
                }
                _ => {
                    return expected_token_err!(["`string`", "`int`"], self);
                }
            }
        } else {
            None
        };

        let mut implements = Vec::new();
        if self.current.kind == TokenKind::Implements {
            self.next();

            while self.current.kind != TokenKind::LeftBrace {
                implements.push(self.full_name()?.into());

                self.optional_comma()?;
            }
        }

        self.lbrace()?;

        let mut body = Block::new();
        while self.current.kind != TokenKind::RightBrace {
            self.skip_comments();
            body.push(self.enum_statement(backed_type.is_some())?);
        }

        self.rbrace()?;

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
        &mut self,
        func: &(dyn Fn(&mut Parser) -> ParseResult<T>),
    ) -> ParseResult<Vec<T>> {
        let mut result: Vec<T> = vec![];
        loop {
            result.push(func(self)?);
            if self.current.kind != TokenKind::Comma {
                break;
            }

            self.next();
        }

        Ok(result)
    }
}
