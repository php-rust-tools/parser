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
pub enum Scope {
    Namespace(ByteString),
    BracedNamespace(Option<ByteString>),

    Interface(ByteString),
    Class(ByteString, Vec<ClassFlag>),
    Trait(ByteString),
    Enum(ByteString),
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
}

impl State {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut iter = tokens.into_iter();

        Self {
            stack: VecDeque::new(),
            current: iter.next().unwrap_or_default(),
            peek: iter.next().unwrap_or_default(),
            iter,
            comments: vec![],
        }
    }

    pub fn named(&self, name: &ByteString) -> String {
        let mut namespace = None;
        for scope in &self.stack {
            match scope {
                Scope::Namespace(n) => {
                    namespace = Some(n.to_string());

                    break;
                }
                Scope::BracedNamespace(n) => {
                    namespace = n.as_ref().map(|s| s.to_string());

                    break;
                }
                _ => {}
            }
        }

        match namespace {
            Some(v) => format!("{}\\{}", v, name),
            None => name.to_string(),
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

    pub fn enter(&mut self, state: Scope) {
        self.stack.push_back(state);
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
