use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::parser::ast::comments::CommentGroup;
use crate::parser::ast::identifiers::SimpleIdentifier;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct GotoLabel {
    pub comments: CommentGroup,
    pub label: SimpleIdentifier, // `foo`
    pub colon: Span,             // `:`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct GotoStatement {
    pub comments: CommentGroup,
    pub keyword: Span,           // `goto`
    pub label: SimpleIdentifier, // `foo`
    pub semicolon: Span,         // `;`
}
