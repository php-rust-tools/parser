use trunk_lexer::{TokenKind, Token};

use crate::Parser;

impl Parser {
    pub(crate) fn skip_comments(&mut self) {
        while let TokenKind::Comment(_) = self.current.kind {
            self.next();
        }
    }

    pub(crate) fn gather_comments(&mut self) {
        while let TokenKind::Comment(_) = self.current.kind {
            self.comments.push(self.current.clone());
            self.next();
        }
    }

    pub(crate) fn clear_comments(&mut self) -> Vec<Token> {
        let c = self.comments.clone();
        self.comments = vec![];
        c
    }
}