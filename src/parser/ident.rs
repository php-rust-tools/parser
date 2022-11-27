use super::{ParseError, ParseResult};
use crate::Parser;
use crate::{ByteString, TokenKind};

impl Parser {
    /// Expect an unqualified identifier such as Foo or Bar.
    pub(crate) fn ident(&mut self) -> ParseResult<ByteString> {
        Ok(expect!(
            self,
            TokenKind::Identifier(i),
            i,
            "expected identifier"
        ))
    }

    /// Expect an unqualified or qualified identifier such as Foo, Bar or Foo\Bar.
    pub(crate) fn name(&mut self) -> ParseResult<ByteString> {
        Ok(expect!(
            self,
            TokenKind::Identifier(i) | TokenKind::QualifiedIdentifier(i),
            i,
            "expected identifier"
        ))
    }

    /// Expect an unqualified, qualified or fully qualified identifier such as Foo, Foo\Bar or \Foo\Bar.
    pub(crate) fn full_name(&mut self) -> ParseResult<ByteString> {
        Ok(expect!(
            self,
            TokenKind::Identifier(i)
                | TokenKind::QualifiedIdentifier(i)
                | TokenKind::FullyQualifiedIdentifier(i),
            i,
            "expected identifier"
        ))
    }

    pub(crate) fn var(&mut self) -> ParseResult<ByteString> {
        Ok(expect!(
            self,
            TokenKind::Variable(v),
            v,
            "expected variable name"
        ))
    }

    pub(crate) fn full_name_maybe_type_keyword(&mut self) -> ParseResult<ByteString> {
        match self.current.kind {
            TokenKind::Array | TokenKind::Callable => {
                let r = Ok(self.current.kind.to_string().into());
                self.next();
                r
            }
            _ => self.full_name(),
        }
    }

    pub(crate) fn type_with_static(&mut self) -> ParseResult<ByteString> {
        Ok(match self.current.kind {
            TokenKind::Static | TokenKind::Null | TokenKind::True | TokenKind::False => {
                let str = self.current.kind.to_string();
                self.next();
                str.into()
            }
            _ => self.full_name_maybe_type_keyword()?,
        })
    }

    pub(crate) fn ident_maybe_reserved(&mut self) -> ParseResult<ByteString> {
        match self.current.kind {
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
            | TokenKind::For
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
            | TokenKind::Enum => {
                let string = self.current.kind.to_string().into();
                self.next();
                Ok(string)
            }
            _ => self.ident(),
        }
    }
}
