use std::vec::IntoIter;

use crate::lexer::token::Token;
use crate::lexer::token::TokenKind;

#[derive(Debug, Clone)]
pub struct State {
    pub current: Token,
    pub peek: Token,
    pub iter: IntoIter<Token>,
    pub comments: Vec<Token>,
}

impl State {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut iter = tokens.into_iter();

        Self {
            current: iter.next().unwrap_or_default(),
            peek: iter.next().unwrap_or_default(),
            iter,
            comments: vec![],
        }
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
