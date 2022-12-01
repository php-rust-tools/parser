use crate::lexer::token::Token;
use crate::lexer::token::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub(in crate::parser) fn skip_comments(&mut self) {
        while matches!(
            self.current.kind,
            TokenKind::Comment(_) | TokenKind::DocComment(_)
        ) {
            self.next();
        }
    }

    pub(in crate::parser) fn gather_comments(&mut self) {
        while matches!(
            self.current.kind,
            TokenKind::Comment(_) | TokenKind::DocComment(_)
        ) {
            self.comments.push(self.current.clone());
            self.next();
        }
    }

    pub(in crate::parser) fn clear_comments(&mut self) -> Vec<Token> {
        let c = self.comments.clone();
        self.comments = vec![];
        c
    }
}
