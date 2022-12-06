use serde::{Deserialize, Serialize};

use crate::lexer::byte_string::ByteString;
use crate::lexer::token::Span;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub enum CommentFormat {
    SingleLine,
    MultiLine,
    HashMark,
    Document,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Comment {
    pub start: Span,
    pub end: Span,
    pub format: CommentFormat,
    pub content: ByteString,
}
