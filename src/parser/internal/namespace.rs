use crate::lexer::token::TokenKind;
use crate::parser::ast::Block;
use crate::parser::ast::Statement;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::state::NamespaceType;
use crate::parser::state::Scope;
use crate::parser::state::State;
use crate::parser::Parser;
use crate::prelude::ByteString;
use crate::scoped;

impl Parser {
    pub(in crate::parser) fn namespace(&self, state: &mut State) -> ParseResult<Statement> {
        state.next();

        let name = self.optional_name(state);

        if let Some(name) = &name {
            if state.current.kind != TokenKind::LeftBrace {
                match state.namespace_type() {
                    Some(NamespaceType::Braced) => {
                        return Err(ParseError::MixingBracedAndUnBracedNamespaceDeclarations(
                            state.current.span,
                        ));
                    }
                    Some(NamespaceType::Unbraced) => {
                        // exit the current namespace scope.
                        // we don't need to check if the current scope is a namespace
                        // because we know it is, it is not possible for it to be anything else.
                        // as using `namespace` anywhere aside from a top-level stmt would result
                        // in a parse error.
                        state.exit();
                    }
                    _ => {}
                }

                return self.unbraced_namespace(state, name.clone());
            }
        }

        match state.namespace_type() {
            Some(NamespaceType::Unbraced) => Err(
                ParseError::MixingBracedAndUnBracedNamespaceDeclarations(state.current.span),
            ),
            Some(NamespaceType::Braced) if state.namespace().is_some() => {
                Err(ParseError::NestedNamespaceDeclarations(state.current.span))
            }
            _ => self.braced_namespace(state, name),
        }
    }

    fn unbraced_namespace(&self, state: &mut State, name: ByteString) -> ParseResult<Statement> {
        let body = scoped!(state, Scope::Namespace(name.clone()), {
            let mut body = Block::new();
            while !state.is_eof() {
                body.push(self.top_level_statement(state)?);
            }

            Ok(body)
        })?;

        Ok(Statement::Namespace { name, body })
    }

    fn braced_namespace(
        &self,
        state: &mut State,
        name: Option<ByteString>,
    ) -> ParseResult<Statement> {
        self.lbrace(state)?;

        let body = scoped!(state, Scope::BracedNamespace(name.clone()), {
            let mut body = Block::new();
            while state.current.kind != TokenKind::RightBrace && !state.is_eof() {
                body.push(self.top_level_statement(state)?);
            }

            Ok(body)
        })?;

        self.rbrace(state)?;

        Ok(Statement::BracedNamespace { name, body })
    }
}
