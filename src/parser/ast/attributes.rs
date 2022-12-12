use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::parser::ast::Expression;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Attribute {
    pub start: Span,
    pub end: Span,
    // TODO(azjezz): name + arg list
    pub expression: Expression,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AttributeGroup {
    pub start: Span,
    pub end: Span,
    pub members: Vec<Attribute>,
}
