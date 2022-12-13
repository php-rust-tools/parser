use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::parser::ast::arguments::ArgumentList;
use crate::parser::ast::identifiers::SimpleIdentifier;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Attribute {
    pub start: Span,
    pub end: Span,
    pub name: SimpleIdentifier,
    pub arguments: Option<ArgumentList>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AttributeGroup {
    pub start: Span,
    pub end: Span,
    pub members: Vec<Attribute>,
}
