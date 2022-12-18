use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::byte_string::ByteString;
use crate::lexer::token::Span;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum Literal {
    String(LiteralString),
    Integer(LiteralInteger),
    Float(LiteralFloat),
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct LiteralString {
    pub value: ByteString,
    pub span: Span,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct LiteralInteger {
    pub value: ByteString,
    pub span: Span,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct LiteralFloat {
    pub value: ByteString,
    pub span: Span,
}
