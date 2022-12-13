use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::Statement;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct UnbracedNamespace {
    pub start: Span,
    pub end: Span,
    pub name: SimpleIdentifier,
    pub statements: Vec<Statement>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BracedNamespace {
    pub span: Span,
    pub name: Option<SimpleIdentifier>,
    pub body: BracedNamespaceBody,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BracedNamespaceBody {
    pub start: Span,
    pub end: Span,
    pub statements: Vec<Statement>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum Namespace {
    Unbraced(UnbracedNamespace),
    Braced(BracedNamespace),
}
