use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::lexer::byte_string::ByteString;
use crate::lexer::token::Span;

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct Identifier {
    pub start: Span,
    pub name: ByteString,
    pub end: Span,
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
