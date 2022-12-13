use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::Block;
use crate::parser::ast::Expression;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum CatchType {
    Identifier(SimpleIdentifier),
    Union(Vec<SimpleIdentifier>),
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TryBlock {
    pub start: Span,
    pub end: Span,
    pub body: Block,
    pub catches: Vec<CatchBlock>,
    pub finally: Option<FinallyBlock>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CatchBlock {
    pub start: Span,
    pub end: Span,
    pub types: CatchType,
    pub var: Option<Expression>,
    pub body: Block,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct FinallyBlock {
    pub start: Span,
    pub end: Span,
    pub body: Block,
}
