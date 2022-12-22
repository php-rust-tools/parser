use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Parenthesized<T> {
    pub left_parenthesis: Span, // `(`
    pub inner: T,
    pub right_parenthesis: Span, // `)`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Braced<T> {
    pub left_brace: Span, // `{`
    pub inner: T,
    pub right_brace: Span, // `}`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SemicolonTerminated<T> {
    pub inner: T,
    pub semicolon: Span, // `;`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CommaSeparated<T> {
    pub inner: Vec<T>,
    pub commas: Vec<Span>, // `,`
}
