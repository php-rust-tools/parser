use crate::lexer::token::TokenKind;
use crate::parser::ast::identifiers::Identifier;
use crate::parser::ast::variables::Variable;
use crate::parser::error::ParseResult;
use crate::parser::state::State;

use crate::peek_token;

/// Expect an unqualified identifier such as Foo or Bar.
pub fn ident(state: &mut State) -> ParseResult<Identifier> {
    let name = peek_token!([
            TokenKind::Identifier(identifier) => {
                identifier.clone()
            },
        ], state, "an identifier");

    let start = state.current.span;
    state.next();
    let end = state.current.span;

    Ok(Identifier { start, name, end })
}

/// Expect an unqualified or qualified identifier such as Foo, Bar or Foo\Bar.
pub fn name(state: &mut State) -> ParseResult<Identifier> {
    let name = peek_token!([
        TokenKind::Identifier(name) | TokenKind::QualifiedIdentifier(name) => {
            name.clone()
        },
    ], state, "an identifier");

    let start = state.current.span;
    state.next();
    let end = state.current.span;

    Ok(Identifier { start, name, end })
}

/// Expect an optional unqualified or qualified identifier such as Foo, Bar or Foo\Bar.
pub fn optional_name(state: &mut State) -> Option<Identifier> {
    let ident = match &state.current.kind {
        TokenKind::Identifier(name) | TokenKind::QualifiedIdentifier(name) => Some(Identifier {
            start: state.current.span,
            name: name.clone(),
            end: state.peek.span,
        }),
        _ => None,
    };

    if ident.is_some() {
        state.next();
    }

    ident
}

/// Expect an unqualified, qualified or fully qualified identifier such as Foo, Foo\Bar or \Foo\Bar.
pub fn full_name(state: &mut State) -> ParseResult<Identifier> {
    let name = peek_token!([
            TokenKind::Identifier(name) | TokenKind::QualifiedIdentifier(name) | TokenKind::FullyQualifiedIdentifier(name) => {
                name.clone()
            },
        ], state, "an identifier");

    let start = state.current.span;
    state.next();
    let end = state.current.span;

    Ok(Identifier { start, name, end })
}

pub fn var(state: &mut State) -> ParseResult<Variable> {
    let name = peek_token!([
            TokenKind::Variable(v) => v.clone(),
        ], state, "a variable");

    let start = state.current.span;
    state.next();
    let end = state.current.span;

    Ok(Variable { start, name, end })
}

pub fn ident_maybe_reserved(state: &mut State) -> ParseResult<Identifier> {
    match state.current.kind {
        _ if is_reserved_ident(&state.current.kind) => {
            let name = state.current.kind.to_string().into();

            let start = state.current.span;
            state.next();
            let end = state.current.span;

            Ok(Identifier { start, name, end })
        }
        _ => ident(state),
    }
}

pub fn is_reserved_ident(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Static
            | TokenKind::Parent
            | TokenKind::Self_
            | TokenKind::Abstract
            | TokenKind::Final
            | TokenKind::For
            | TokenKind::Private
            | TokenKind::Protected
            | TokenKind::Public
            | TokenKind::Include
            | TokenKind::IncludeOnce
            | TokenKind::Eval
            | TokenKind::Require
            | TokenKind::RequireOnce
            | TokenKind::LogicalOr
            | TokenKind::LogicalXor
            | TokenKind::LogicalAnd
            | TokenKind::Instanceof
            | TokenKind::New
            | TokenKind::Clone
            | TokenKind::Exit
            | TokenKind::If
            | TokenKind::ElseIf
            | TokenKind::Else
            | TokenKind::EndIf
            | TokenKind::Echo
            | TokenKind::Do
            | TokenKind::While
            | TokenKind::EndWhile
            | TokenKind::EndFor
            | TokenKind::Foreach
            | TokenKind::EndForeach
            | TokenKind::Declare
            | TokenKind::EndDeclare
            | TokenKind::As
            | TokenKind::Try
            | TokenKind::Catch
            | TokenKind::Finally
            | TokenKind::Throw
            | TokenKind::Use
            | TokenKind::Insteadof
            | TokenKind::Global
            | TokenKind::Var
            | TokenKind::Unset
            | TokenKind::Isset
            | TokenKind::Empty
            | TokenKind::Continue
            | TokenKind::Goto
            | TokenKind::Function
            | TokenKind::Const
            | TokenKind::Return
            | TokenKind::Print
            | TokenKind::Yield
            | TokenKind::List
            | TokenKind::Switch
            | TokenKind::EndSwitch
            | TokenKind::Case
            | TokenKind::Default
            | TokenKind::Break
            | TokenKind::Array
            | TokenKind::Callable
            | TokenKind::Extends
            | TokenKind::Implements
            | TokenKind::Namespace
            | TokenKind::Trait
            | TokenKind::Interface
            | TokenKind::Class
            | TokenKind::ClassConstant
            | TokenKind::TraitConstant
            | TokenKind::FunctionConstant
            | TokenKind::MethodConstant
            | TokenKind::LineConstant
            | TokenKind::FileConstant
            | TokenKind::DirConstant
            | TokenKind::NamespaceConstant
            | TokenKind::HaltCompiler
            | TokenKind::Fn
            | TokenKind::Match
            | TokenKind::Enum
            | TokenKind::From
    )
}
