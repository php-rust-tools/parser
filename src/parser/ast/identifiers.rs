use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::lexer::byte_string::ByteString;
use crate::lexer::token::Span;
use crate::parser::ast::Expression;

#[derive(Debug, PartialEq, Clone)]
pub enum Identifier {
    SimpleIdentifier(SimpleIdentifier),
    DynamicIdentifier(DynamicIdentifier),
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct SimpleIdentifier {
    pub span: Span,
    pub name: ByteString,
}

impl Display for SimpleIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct DynamicIdentifier {
    pub start: Span,
    pub expr: Box<Expression>,
    pub end: Span,
}
