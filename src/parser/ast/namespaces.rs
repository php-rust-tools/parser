use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::Statement;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct UnbracedNamespace {
    pub start: Span,                // `namespace`
    pub name: SimpleIdentifier,     // `Foo`
    pub end: Span,                  // `;`
    pub statements: Vec<Statement>, // `*statements*`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BracedNamespace {
    pub namespace: Span,                // `namespace`
    pub name: Option<SimpleIdentifier>, // `Foo`
    pub body: BracedNamespaceBody,      // `{ *statements* }`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BracedNamespaceBody {
    pub start: Span,                // `{`
    pub end: Span,                  // `}`
    pub statements: Vec<Statement>, // `*statements*`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum Namespace {
    Unbraced(UnbracedNamespace), // `namespace Foo; *statements*`
    Braced(BracedNamespace),     // `namespace Foo { *statements* }`
}
