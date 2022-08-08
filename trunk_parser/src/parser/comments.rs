use trunk_lexer::{TokenKind, Token};

use crate::Parser;

impl Parser {
    pub(crate) fn skip_comments(&mut self) {
        while matches!(self.current.kind, TokenKind::Comment(_) | TokenKind::DocComment(_)) {
            self.next();
        }
    }

    pub(crate) fn gather_comments(&mut self) {
        while matches!(self.current.kind, TokenKind::Comment(_) | TokenKind::DocComment(_)) {
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