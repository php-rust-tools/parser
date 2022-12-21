use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::literals::Literal;
use crate::parser::ast::Expression;
use crate::parser::ast::Statement;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DeclareEntry {
    pub key: SimpleIdentifier, // `strict_types`
    pub equals: Span,          // `=`
    pub value: Literal,        // `1`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DeclareEntryGroup {
    pub left_parenthesis: Span,     // `(`
    pub right_parenthesis: Span,    // `)`
    pub entries: Vec<DeclareEntry>, // `strict_types = 1`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeclareBody {
    // declaration is terminated with `;`
    Noop {
        semicolon: Span, // `;`
    },
    // declaration is followed by a `{` and terminated with `}` after multiple statements.
    Braced {
        left_brace: Span,           // `{`
        statements: Vec<Statement>, // `*statements*`
        right_brace: Span,          // `}`
    },
    // declaration is terminated with `;` after a single expression.
    Expression {
        expression: Expression, // `*expression*`
        semicolon: Span,        // `;`
    },
    // declaration is followed by a `:` and terminated with `enddeclare` and `;` after multiple statements.
    Block {
        colon: Span,                // `:`
        statements: Vec<Statement>, // `*statements*`
        end: (Span, Span),          // `enddeclare` + `;`
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Declare {
    pub declare: Span,              // `declare`
    pub entries: DeclareEntryGroup, // `(strict_types = 1)`
    pub body: DeclareBody,          // `;`
}
