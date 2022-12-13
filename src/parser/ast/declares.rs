use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::Expression;
use crate::parser::ast::Statement;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DeclareEntry {
    pub key: SimpleIdentifier,
    pub span: Span,
    pub value: Expression,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DeclareEntryGroup {
    pub start: Span,
    pub end: Span,
    pub entries: Vec<DeclareEntry>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeclareBody {
    // declaration is terminated with `;`
    Noop {
        span: Span,
    },
    // declaration is followed by a `{` and terminated with `}` after multiple statements.
    Braced {
        start: Span,
        statements: Vec<Statement>,
        end: Span,
    },
    // declaration is terminated with `;` after a single expression.
    Expression {
        expression: Expression,
        end: Span,
    },
    // declaration is followed by a `:` and terminated with `enddeclare` and `;` after multiple statements.
    Block {
        start: Span,
        statements: Vec<Statement>,
        end: (Span, Span),
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Declare {
    pub span: Span,
    pub entries: DeclareEntryGroup,
    pub body: DeclareBody,
}
