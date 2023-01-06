use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::node::Node;
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

impl Node for UnbracedNamespace {
    fn children(&self) -> Vec<&dyn Node> {
        let mut children = vec![&self.name as &dyn Node];
        children.extend(
            self.statements
                .iter()
                .map(|s| s as &dyn Node)
                .collect::<Vec<&dyn Node>>(),
        );
        children
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BracedNamespace {
    pub namespace: Span,                // `namespace`
    pub name: Option<SimpleIdentifier>, // `Foo`
    pub body: BracedNamespaceBody,      // `{ *statements* }`
}

impl Node for BracedNamespace {
    fn children(&self) -> Vec<&dyn Node> {
        let mut children = vec![];
        if let Some(name) = &self.name {
            children.push(name as &dyn Node);
        }
        children.push(&self.body as &dyn Node);
        children
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BracedNamespaceBody {
    pub start: Span,                // `{`
    pub end: Span,                  // `}`
    pub statements: Vec<Statement>, // `*statements*`
}

impl Node for BracedNamespaceBody {
    fn children(&self) -> Vec<&dyn Node> {
        self.statements.iter().map(|s| s as &dyn Node).collect()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum NamespaceStatement {
    Unbraced(UnbracedNamespace), // `namespace Foo; *statements*`
    Braced(BracedNamespace),     // `namespace Foo { *statements* }`
}

impl Node for NamespaceStatement {
    fn children(&self) -> Vec<&dyn Node> {
        match self {
            NamespaceStatement::Unbraced(namespace) => vec![namespace],
            NamespaceStatement::Braced(namespace) => vec![namespace],
        }
    }
}
