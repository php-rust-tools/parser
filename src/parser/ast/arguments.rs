use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::parser::ast::comments::CommentGroup;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::Expression;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum Argument {
    Positional {
        comments: CommentGroup,
        ellipsis: Option<Span>, // `...`
        value: Expression,      // `$var`
    },
    Named {
        comments: CommentGroup,
        name: SimpleIdentifier, // `foo`
        colon: Span,            // `:`
        ellipsis: Option<Span>, // `...`
        value: Expression,      // `$var`
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ArgumentList {
    pub comments: CommentGroup,
    pub left_parenthesis: Span,   // `(`
    pub arguments: Vec<Argument>, // `$var`, `...$var`, `foo: $var`, `foo: ...$var`
    pub right_parenthesis: Span,  // `)`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SingleArgument {
    pub comments: CommentGroup,
    pub left_parenthesis: Span,  // `(`
    pub argument: Argument,      // `$var`
    pub right_parenthesis: Span, // `)`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ArgumentPlaceholder {
    pub comments: CommentGroup,
    pub left_parenthesis: Span,  // `(`
    pub ellipsis: Span,          // `...`
    pub right_parenthesis: Span, // `)`
}
