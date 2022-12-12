use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::Expression;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum Argument {
    Positional {
        ellipsis: Option<Span>, // `...`
        value: Expression,      // `$var`
    },
    Named {
        name: SimpleIdentifier, // `foo`
        span: Span,             // `:`
        ellipsis: Option<Span>, // `...`
        value: Expression,      // `$var`
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ArgumentList {
    pub start: Span,              // `(`
    pub arguments: Vec<Argument>, // `$var`, `...$var`, `foo: $var`, `foo: ...$var`
    pub end: Span,                // `)`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ArgumentPlaceholder {
    pub start: Span,    // `(`
    pub ellipsis: Span, // `...`
    pub end: Span,      // `)`
}
