use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::lexer::byte_string::ByteString;
use crate::lexer::token::Span;
use crate::parser::ast::Expression;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum Identifier {
    SimpleIdentifier(SimpleIdentifier),
    DynamicIdentifier(DynamicIdentifier),
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SimpleIdentifier {
    pub span: Span,
    pub value: ByteString,
}

impl Display for SimpleIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DynamicIdentifier {
    pub start: Span,
    pub expr: Box<Expression>,
    pub end: Span,
}
