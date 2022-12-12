use crate::lexer::token::Span;
use crate::lexer::token::TokenKind;
use crate::parser;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::namespaces::BracedNamespace;
use crate::parser::ast::namespaces::BracedNamespaceBody;
use crate::parser::ast::namespaces::Namespace;
use crate::parser::ast::namespaces::UnbracedNamespace;
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
    let start = utils::skip(state, TokenKind::Namespace)?;
    let name = identifiers::optional_name(state);

    if let Some(name) = &name {
        if state.stream.current().kind != TokenKind::LeftBrace {
            if let Some(NamespaceType::Braced) = state.namespace_type() {
                return Err(ParseError::MixingBracedAndUnBracedNamespaceDeclarations(
                    state.stream.current().span,
                ));
            }

            return unbraced_namespace(state, start, name.clone());
        }
    }

    match state.namespace_type() {
        Some(NamespaceType::Unbraced) => Err(
            ParseError::MixingBracedAndUnBracedNamespaceDeclarations(state.stream.current().span),
        ),
        Some(NamespaceType::Braced) if state.namespace().is_some() => Err(
            ParseError::NestedNamespaceDeclarations(state.stream.current().span),
        ),
        _ => braced_namespace(state, start, name),
    }
}

fn unbraced_namespace(
    state: &mut State,
    start: Span,
    name: SimpleIdentifier,
) -> ParseResult<Statement> {
    let end = utils::skip_semicolon(state)?;

    let statements = scoped!(state, Scope::Namespace(name.clone()), {
        let mut statements = Block::new();
        // since this is an unbraced namespace, as soon as we encouter another
        // `namespace` token as a top level statement, this namespace scope ends.
        // otherwise we will end up with nested namespace statements.
        while state.stream.current().kind != TokenKind::Namespace && !state.stream.is_eof() {
            statements.push(parser::top_level_statement(state)?);
        }

        statements
    });

    Ok(Statement::Namespace(Namespace::Unbraced(
        UnbracedNamespace {
            start,
            end,
            name,
            statements,
        },
    )))
}

fn braced_namespace(
    state: &mut State,
    span: Span,
    name: Option<SimpleIdentifier>,
) -> ParseResult<Statement> {
    let body = scoped!(state, Scope::BracedNamespace(name.clone()), {
        let start = utils::skip_left_brace(state)?;

        let mut statements = Block::new();
        while state.stream.current().kind != TokenKind::RightBrace && !state.stream.is_eof() {
            statements.push(parser::top_level_statement(state)?);
        }

        let end = utils::skip_right_brace(state)?;

        BracedNamespaceBody {
            start,
            end,
            statements,
        }
    });

    Ok(Statement::Namespace(Namespace::Braced(BracedNamespace {
        span,
        name,
        body,
    })))
}
