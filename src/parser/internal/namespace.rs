use crate::lexer::token::TokenKind;
use crate::parser::ast::identifier::Identifier;
use crate::parser::ast::Block;
use crate::parser::ast::Statement;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::state::NamespaceType;
use crate::parser::state::Scope;
use crate::parser::state::State;
use crate::parser::Parser;
use crate::scoped;

impl Parser {
    pub(in crate::parser) fn namespace(&self, state: &mut State) -> ParseResult<Statement> {
        state.next();

        let name = self.optional_name(state);

        if let Some(name) = &name {
            if state.current.kind != TokenKind::LeftBrace {
                if let Some(NamespaceType::Braced) = state.namespace_type() {
                    return Err(ParseError::MixingBracedAndUnBracedNamespaceDeclarations(
                        state.current.span,
                    ));
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

    fn unbraced_namespace(&self, state: &mut State, name: Identifier) -> ParseResult<Statement> {
        let body = scoped!(state, Scope::Namespace(name.clone()), {
            let mut body = Block::new();
            // since this is an unbraced namespace, as soon as we encouter another
            // `namespace` token as a top level statement, this namespace scope ends.
            // otherwise we will end up with nested namespace statements.
            while state.current.kind != TokenKind::Namespace && !state.is_eof() {
                body.push(self.top_level_statement(state)?);
            }

            body
        });

        Ok(Statement::Namespace { name, body })
    }

    fn braced_namespace(
        &self,
        state: &mut State,
        name: Option<Identifier>,
    ) -> ParseResult<Statement> {
        self.lbrace(state)?;

        let body = scoped!(state, Scope::BracedNamespace(name.clone()), {
            let mut body = Block::new();
            while state.current.kind != TokenKind::RightBrace && !state.is_eof() {
                body.push(self.top_level_statement(state)?);
            }

            body
        });

        self.rbrace(state)?;

        Ok(Statement::BracedNamespace { name, body })
    }
}
