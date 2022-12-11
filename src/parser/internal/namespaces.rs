use crate::lexer::token::TokenKind;
use crate::parser;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::Block;
use crate::parser::ast::Statement;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::internal::identifiers;
use crate::parser::internal::utils;
use crate::parser::state::NamespaceType;
use crate::parser::state::Scope;
use crate::parser::state::State;
use crate::scoped;

pub fn namespace(state: &mut State) -> ParseResult<Statement> {
    state.stream.next();

    let name = identifiers::optional_name(state);

    if let Some(name) = &name {
        if state.stream.current().kind != TokenKind::LeftBrace {
            if let Some(NamespaceType::Braced) = state.namespace_type() {
                return Err(ParseError::MixingBracedAndUnBracedNamespaceDeclarations(
                    state.stream.current().span,
                ));
            }

            return unbraced_namespace(state, name.clone());
        }
    }

    match state.namespace_type() {
        Some(NamespaceType::Unbraced) => Err(
            ParseError::MixingBracedAndUnBracedNamespaceDeclarations(state.stream.current().span),
        ),
        Some(NamespaceType::Braced) if state.namespace().is_some() => Err(
            ParseError::NestedNamespaceDeclarations(state.stream.current().span),
        ),
        _ => braced_namespace(state, name),
    }
}

fn unbraced_namespace(state: &mut State, name: SimpleIdentifier) -> ParseResult<Statement> {
    let body = scoped!(state, Scope::Namespace(name.clone()), {
        let mut body = Block::new();
        // since this is an unbraced namespace, as soon as we encouter another
        // `namespace` token as a top level statement, this namespace scope ends.
        // otherwise we will end up with nested namespace statements.
        while state.stream.current().kind != TokenKind::Namespace && !state.stream.is_eof() {
            body.push(parser::top_level_statement(state)?);
        }

        body
    });

    Ok(Statement::Namespace { name, body })
}

fn braced_namespace(state: &mut State, name: Option<SimpleIdentifier>) -> ParseResult<Statement> {
    utils::skip_left_brace(state)?;

    let body = scoped!(state, Scope::BracedNamespace(name.clone()), {
        let mut body = Block::new();
        while state.stream.current().kind != TokenKind::RightBrace && !state.stream.is_eof() {
            body.push(parser::top_level_statement(state)?);
        }

        body
    });

    utils::skip_right_brace(state)?;

    Ok(Statement::BracedNamespace { name, body })
}
