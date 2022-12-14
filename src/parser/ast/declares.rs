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
    pub key: SimpleIdentifier, // `strict_types`
    pub span: Span,            // `=`
    pub value: Expression,     // `1`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DeclareEntryGroup {
    pub start: Span,                // `(`
    pub end: Span,                  // `)`
    pub entries: Vec<DeclareEntry>, // `strict_types = 1`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeclareBody {
    // declaration is terminated with `;`
    Noop {
        span: Span, // `;`
    },
    // declaration is followed by a `{` and terminated with `}` after multiple statements.
    Braced {
        start: Span,                // `{`
        statements: Vec<Statement>, // `*statements*`
        end: Span,                  // `}`
    },
    // declaration is terminated with `;` after a single expression.
    Expression {
        expression: Expression, // `*expression*`
        end: Span,              // `;`
    },
    // declaration is followed by a `:` and terminated with `enddeclare` and `;` after multiple statements.
    Block {
        start: Span,                // `:`
        statements: Vec<Statement>, // `*statements*`
        end: (Span, Span),          // `enddeclare` + `;`
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Declare {
    pub span: Span,                 // `declare`
    pub entries: DeclareEntryGroup, // `(strict_types = 1)`
    pub body: DeclareBody,          // `;`
}
