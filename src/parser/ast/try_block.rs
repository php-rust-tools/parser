use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::node::Node;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::Block;

use super::variables::SimpleVariable;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum CatchType {
    Identifier(SimpleIdentifier),
    Union(Vec<SimpleIdentifier>),
}

impl Node for CatchType {
    fn children(&self) -> Vec<&dyn Node> {
        match self {
            CatchType::Identifier(identifier) => vec![identifier],
            CatchType::Union(identifiers) => identifiers.iter().map(|i| i as &dyn Node).collect(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TryStatement {
    pub start: Span,
    pub end: Span,
    pub body: Block,
    pub catches: Vec<CatchBlock>,
    pub finally: Option<FinallyBlock>,
}

impl Node for TryStatement {
    fn children(&self) -> Vec<&dyn Node> {
        let mut children: Vec<&dyn Node> = vec![&self.body];
        for catch in &self.catches {
            children.push(catch);
        }
        if let Some(finally) = &self.finally {
            children.push(finally);
        }
        children
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CatchBlock {
    pub start: Span,
    pub end: Span,
    pub types: CatchType,
    pub var: Option<SimpleVariable>,
    pub body: Block,
}

impl Node for CatchBlock {
    fn children(&self) -> Vec<&dyn Node> {
        let mut children = vec![&self.types as &dyn Node];
        if let Some(var) = &self.var {
            children.push(var as &dyn Node);
        }
        children.push(&self.body as &dyn Node);
        children
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct FinallyBlock {
    pub start: Span,
    pub end: Span,
    pub body: Block,
}

impl Node for FinallyBlock {
    fn children(&self) -> Vec<&dyn Node> {
        vec![&self.body as &dyn Node]
    }
}
