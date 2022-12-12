use crate::lexer::token::TokenKind;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::state::State;

use crate::peek_token;

pub fn identifier_of(state: &mut State, kinds: &[&str]) -> ParseResult<SimpleIdentifier> {
    let ident = identifier(state)?;

    let name = ident.value.to_string();

    if kinds.contains(&name.as_str()) {
        Ok(ident)
    } else {
        Err(ParseError::ExpectedIdentifier(
            kinds.iter().map(|s| s.to_string()).collect(),
            name,
            state.stream.current().span,
        ))
    }
}

/// Expect an unqualified identifier such as Foo or Bar for a class, interface, trait, or an enum name.
pub fn type_identifier(state: &mut State) -> ParseResult<SimpleIdentifier> {
    match state.stream.current().kind.clone() {
        TokenKind::Identifier(name) => {
            let span = state.stream.current().span;

            state.stream.next();

            Ok(SimpleIdentifier { span, value: name })
        }
        TokenKind::Enum | TokenKind::From => {
            let span = state.stream.current().span;
            let name = state.stream.current().kind.to_string().into();

            state.stream.next();

            Ok(SimpleIdentifier { span, value: name })
        }
        t if is_reserved_identifier(&t) => Err(ParseError::CannotUseReservedKeywordAsATypeName(
            state.stream.current().kind.to_string(),
            state.stream.current().span,
        )),
        _ => Err(ParseError::ExpectedToken(
            vec!["an identifier".to_owned()],
            Some(state.stream.current().kind.to_string()),
            state.stream.current().span,
        )),
    }
}

/// Expect an unqualified identifier such as foo or bar for a goto label name.
pub fn label_identifier(state: &mut State) -> ParseResult<SimpleIdentifier> {
    match state.stream.current().kind.clone() {
        TokenKind::Identifier(name) => {
            let span = state.stream.current().span;

            state.stream.next();

            Ok(SimpleIdentifier { span, value: name })
        }
        TokenKind::Enum | TokenKind::From => {
            let span = state.stream.current().span;
            let name = state.stream.current().kind.to_string().into();

            state.stream.next();

            Ok(SimpleIdentifier { span, value: name })
        }
        t if is_reserved_identifier(&t) => Err(ParseError::CannotUseReservedKeywordAsAGoToLabel(
            state.stream.current().kind.to_string(),
            state.stream.current().span,
        )),
        _ => Err(ParseError::ExpectedToken(
            vec!["an identifier".to_owned()],
            Some(state.stream.current().kind.to_string()),
            state.stream.current().span,
        )),
    }
}

/// Expect an unqualified identifier such as FOO or BAR for a constant name.
pub fn constant_identifier(state: &mut State) -> ParseResult<SimpleIdentifier> {
    match state.stream.current().kind.clone() {
        TokenKind::Identifier(name) => {
            let span = state.stream.current().span;

            state.stream.next();

            Ok(SimpleIdentifier { span, value: name })
        }
        TokenKind::Enum | TokenKind::From | TokenKind::Self_ | TokenKind::Parent => {
            let span = state.stream.current().span;
            let name = state.stream.current().kind.to_string().into();

            state.stream.next();

            Ok(SimpleIdentifier { span, value: name })
        }
        t if is_reserved_identifier(&t) => {
            Err(ParseError::CannotUseReservedKeywordAsAConstantName(
                state.stream.current().kind.to_string(),
                state.stream.current().span,
            ))
        }
        _ => Err(ParseError::ExpectedToken(
            vec!["an identifier".to_owned()],
            Some(state.stream.current().kind.to_string()),
            state.stream.current().span,
        )),
    }
}

/// Expect an unqualified identifier such as Foo or Bar.
pub fn identifier(state: &mut State) -> ParseResult<SimpleIdentifier> {
    if let TokenKind::Identifier(name) = state.stream.current().kind.clone() {
        let span = state.stream.current().span;

        state.stream.next();

        Ok(SimpleIdentifier { span, value: name })
    } else {
        Err(ParseError::ExpectedToken(
            vec!["an identifier".to_owned()],
            Some(state.stream.current().kind.to_string()),
            state.stream.current().span,
        ))
    }
}

/// Expect an unqualified or qualified identifier such as Foo, Bar or Foo\Bar.
pub fn name(state: &mut State) -> ParseResult<SimpleIdentifier> {
    let name = peek_token!([
        TokenKind::Identifier(name) | TokenKind::QualifiedIdentifier(name) => {
            name.clone()
        },
    ], state, "an identifier");

    let span = state.stream.current().span;
    state.stream.next();

    Ok(SimpleIdentifier { span, value: name })
}

/// Expect an optional unqualified or qualified identifier such as Foo, Bar or Foo\Bar.
pub fn optional_name(state: &mut State) -> Option<SimpleIdentifier> {
    let ident = match &state.stream.current().kind {
        TokenKind::Identifier(name) | TokenKind::QualifiedIdentifier(name) => {
            Some(SimpleIdentifier {
                span: state.stream.current().span,
                value: name.clone(),
            })
        }
        _ => None,
    };

    if ident.is_some() {
        state.stream.next();
    }

    ident
}

/// Expect an unqualified, qualified or fully qualified identifier such as Foo, Foo\Bar or \Foo\Bar.
pub fn full_name(state: &mut State) -> ParseResult<SimpleIdentifier> {
    match state.stream.current().kind.clone() {
        TokenKind::Identifier(name)
        | TokenKind::QualifiedIdentifier(name)
        | TokenKind::FullyQualifiedIdentifier(name) => {
            let span = state.stream.current().span;

            state.stream.next();

            Ok(SimpleIdentifier { span, value: name })
        }
        _ => Err(ParseError::ExpectedToken(
            vec!["an identifier".to_owned()],
            Some(state.stream.current().kind.to_string()),
            state.stream.current().span,
        )),
    }
}

/// Expect an unqualified, qualified or fully qualified identifier such as Foo, Foo\Bar or \Foo\Bar.
pub fn full_type_name(state: &mut State) -> ParseResult<SimpleIdentifier> {
    match state.stream.current().kind.clone() {
        TokenKind::Identifier(name)
        | TokenKind::QualifiedIdentifier(name)
        | TokenKind::FullyQualifiedIdentifier(name) => {
            let span = state.stream.current().span;

            state.stream.next();

            Ok(SimpleIdentifier { span, value: name })
        }
        TokenKind::Enum | TokenKind::From => {
            let span = state.stream.current().span;
            let name = state.stream.current().kind.to_string().into();

            state.stream.next();

            Ok(SimpleIdentifier { span, value: name })
        }
        t if is_reserved_identifier(&t) => Err(ParseError::CannotUseReservedKeywordAsATypeName(
            state.stream.current().kind.to_string(),
            state.stream.current().span,
        )),
        _ => Err(ParseError::ExpectedToken(
            vec!["an identifier".to_owned()],
            Some(state.stream.current().kind.to_string()),
            state.stream.current().span,
        )),
    }
}

/// Expect an unqualified, qualified or fully qualified identifier such as Foo, Foo\Bar or \Foo\Bar.
pub fn full_type_name_including_self(state: &mut State) -> ParseResult<SimpleIdentifier> {
    match state.stream.current().kind.clone() {
        TokenKind::Identifier(name)
        | TokenKind::QualifiedIdentifier(name)
        | TokenKind::FullyQualifiedIdentifier(name) => {
            let span = state.stream.current().span;

            state.stream.next();

            Ok(SimpleIdentifier { span, value: name })
        }
        TokenKind::Enum
        | TokenKind::From
        | TokenKind::Self_
        | TokenKind::Static
        | TokenKind::Parent => {
            let span = state.stream.current().span;
            let name = state.stream.current().kind.to_string().into();

            state.stream.next();

            Ok(SimpleIdentifier { span, value: name })
        }
        t if is_reserved_identifier(&t) => Err(ParseError::CannotUseReservedKeywordAsATypeName(
            state.stream.current().kind.to_string(),
            state.stream.current().span,
        )),
        _ => Err(ParseError::ExpectedToken(
            vec!["an identifier".to_owned()],
            Some(state.stream.current().kind.to_string()),
            state.stream.current().span,
        )),
    }
}

pub fn identifier_maybe_reserved(state: &mut State) -> ParseResult<SimpleIdentifier> {
    match state.stream.current().kind {
        _ if is_reserved_identifier(&state.stream.current().kind) => {
            let name = state.stream.current().kind.to_string().into();

            let span = state.stream.current().span;
            state.stream.next();

            Ok(SimpleIdentifier { span, value: name })
        }
        _ => identifier(state),
    }
}

pub fn identifier_maybe_soft_reserved(state: &mut State) -> ParseResult<SimpleIdentifier> {
    match state.stream.current().kind {
        _ if is_soft_reserved_identifier(&state.stream.current().kind) => {
            let name = state.stream.current().kind.to_string().into();
            let span = state.stream.current().span;
            state.stream.next();

            Ok(SimpleIdentifier { span, value: name })
        }
        _ => identifier(state),
    }
}

pub fn is_identifier_maybe_soft_reserved(kind: &TokenKind) -> bool {
    if let TokenKind::Identifier(_) = kind {
        return true;
    }

    is_soft_reserved_identifier(kind)
}

pub fn is_identifier_maybe_reserved(kind: &TokenKind) -> bool {
    if let TokenKind::Identifier(_) = kind {
        return true;
    }

    is_reserved_identifier(kind)
}

pub fn is_soft_reserved_identifier(kind: &TokenKind) -> bool {
    matches!(kind, |TokenKind::Parent| TokenKind::Self_
        | TokenKind::True
        | TokenKind::False
        | TokenKind::List
        | TokenKind::Null
        | TokenKind::Enum
        | TokenKind::From
        | TokenKind::Readonly)
}

pub fn is_reserved_identifier(kind: &TokenKind) -> bool {
    if is_soft_reserved_identifier(kind) {
        return true;
    }

    matches!(
        kind,
        TokenKind::Static
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
            | TokenKind::Die
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
    )
}
