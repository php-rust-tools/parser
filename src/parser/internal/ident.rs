use crate::lexer::byte_string::ByteString;
use crate::lexer::token::TokenKind;
use crate::parser::error::ParseResult;
use crate::parser::state::State;
use crate::parser::Parser;

use crate::expect_token;

impl Parser {
    /// Expect an unqualified identifier such as Foo or Bar.
    pub(in crate::parser) fn ident(&self, state: &mut State) -> ParseResult<ByteString> {
        Ok(expect_token!([
            TokenKind::Identifier(identifier) => identifier,
        ], state, "an identifier"))
    }

    /// Expect an unqualified or qualified identifier such as Foo, Bar or Foo\Bar.
    pub(in crate::parser) fn name(&self, state: &mut State) -> ParseResult<ByteString> {
        Ok(expect_token!([
            TokenKind::Identifier(identifier) => identifier,
            TokenKind::QualifiedIdentifier(qualified) => qualified,
        ], state, "an identifier"))
    }

    /// Expect an unqualified, qualified or fully qualified identifier such as Foo, Foo\Bar or \Foo\Bar.
    pub(in crate::parser) fn full_name(&self, state: &mut State) -> ParseResult<ByteString> {
        Ok(expect_token!([
            TokenKind::Identifier(identifier) => identifier,
            TokenKind::QualifiedIdentifier(qualified) => qualified,
            TokenKind::FullyQualifiedIdentifier(fully_qualified) => fully_qualified,
        ], state, "an identifier"))
    }

    pub(in crate::parser) fn var(&self, state: &mut State) -> ParseResult<ByteString> {
        Ok(expect_token!([
            TokenKind::Variable(v) => v,
        ], state, "a variable"))
    }

    pub(in crate::parser) fn ident_maybe_reserved(
        &self,
        state: &mut State,
    ) -> ParseResult<ByteString> {
        match state.current.kind {
            _ if is_reserved_ident(&state.current.kind) => {
                let string = state.current.kind.to_string().into();
                state.next();
                Ok(string)
            }
            _ => self.ident(state),
        }
    }
}

pub fn is_reserved_ident(kind: &TokenKind) -> bool {
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
