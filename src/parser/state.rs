use std::collections::VecDeque;
use std::vec::IntoIter;

use crate::lexer::byte_string::ByteString;
use crate::lexer::token::Token;
use crate::lexer::token::TokenKind;
use crate::parser::ast::ClassFlag;
use crate::parser::ast::MethodFlag;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NamespaceType {
    Braced,
    Unbraced,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Scope {
    Namespace(ByteString),
    BracedNamespace(Option<ByteString>),

    Interface(ByteString),
    Class(ByteString, Vec<ClassFlag>),
    Trait(ByteString),
    Enum(ByteString, bool),
    AnonymousClass,

    Function(ByteString),
    Method(ByteString, Vec<MethodFlag>),
    AnonymousFunction(bool),
    ArrowFunction(bool),
}

#[derive(Debug, Clone)]
pub struct State {
    pub stack: VecDeque<Scope>,
    pub current: Token,
    pub peek: Token,
    pub iter: IntoIter<Token>,
    pub comments: Vec<Token>,
    pub namespace_type: Option<NamespaceType>,
}

impl State {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut iter = tokens.into_iter();

        Self {
            stack: VecDeque::with_capacity(3),
            current: iter.next().unwrap_or_default(),
            peek: iter.next().unwrap_or_default(),
            iter,
            comments: vec![],
            namespace_type: None,
        }
    }

    /// Return the namespace type used in the current state
    ///
    /// The namespace type is retrieve from the last entered
    /// namespace scope.
    ///
    /// Note: even when a namespace scope is exited, the namespace type
    /// is retained, until the next namespace scope is entered.
    pub fn namespace_type(&self) -> Option<NamespaceType> {
        self.namespace_type.clone()
    }

    pub fn namespace(&self) -> Option<&Scope> {
        for scope in &self.stack {
            match scope {
                Scope::Namespace(_) | Scope::BracedNamespace(_) => {
                    return Some(scope);
                }
                _ => {}
            }
        }

        None
    }

    pub fn named(&self, name: &ByteString) -> String {
        match self.namespace() {
            Some(Scope::Namespace(n)) | Some(Scope::BracedNamespace(Some(n))) => {
                format!("{}\\{}", n, name)
            }
            _ => name.to_string(),
        }
    }

    pub fn scope(&self) -> ParseResult<&Scope> {
        self.stack
            .back()
            .ok_or(ParseError::UnpredictableState(self.current.span))
    }

    pub fn parent(&self) -> ParseResult<&Scope> {
        self.stack
            .get(self.stack.len() - 2)
            .ok_or(ParseError::UnpredictableState(self.current.span))
    }

    pub fn enter(&mut self, scope: Scope) {
        match &scope {
            Scope::Namespace(_) => {
                self.namespace_type = Some(NamespaceType::Unbraced);
            }
            Scope::BracedNamespace(_) => {
                self.namespace_type = Some(NamespaceType::Braced);
            }
            _ => {}
        }

        self.stack.push_back(scope);
    }

    pub fn exit(&mut self) {
        self.stack.pop_back();
    }

    pub fn skip_comments(&mut self) {
        while matches!(
            self.current.kind,
            TokenKind::Comment(_) | TokenKind::DocComment(_)
        ) {
            self.next();
        }
    }

    pub fn gather_comments(&mut self) {
        while matches!(
            self.current.kind,
            TokenKind::Comment(_) | TokenKind::DocComment(_)
        ) {
            self.comments.push(self.current.clone());
            self.next();
        }
    }

    pub fn clear_comments(&mut self) -> Vec<Token> {
        let c = self.comments.clone();
        self.comments = vec![];
        c
    }

    pub fn is_eof(&mut self) -> bool {
        self.current.kind == TokenKind::Eof
    }

    pub fn next(&mut self) {
        self.current = self.peek.clone();
        self.peek = self.iter.next().unwrap_or_default()
    }
}
