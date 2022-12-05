use std::fmt::Display;

use crate::lexer::byte_string::ByteString;
use crate::lexer::token::Span;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Variable {
    pub start: Span,
    pub name: ByteString,
    pub end: Span,
}

impl Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", self.name)
    }
}
