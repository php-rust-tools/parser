use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::node::Node;
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

impl Node for Argument {
    fn children(&mut self) -> Vec<&mut dyn Node> {
        match self {
            Argument::Positional { value, .. } => vec![value],
            Argument::Named { name, value, .. } => vec![name, value],
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ArgumentList {
    pub comments: CommentGroup,
    pub left_parenthesis: Span,   // `(`
    pub arguments: Vec<Argument>, // `$var`, `...$var`, `foo: $var`, `foo: ...$var`
    pub right_parenthesis: Span,  // `)`
}

impl Node for ArgumentList {
    fn children(&mut self) -> Vec<&mut dyn Node> {
        self.arguments
            .iter_mut()
            .map(|a| a as &mut dyn Node)
            .collect()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SingleArgument {
    pub comments: CommentGroup,
    pub left_parenthesis: Span,  // `(`
    pub argument: Argument,      // `$var`
    pub right_parenthesis: Span, // `)`
}

impl Node for SingleArgument {
    fn children(&mut self) -> Vec<&mut dyn Node> {
        vec![&mut self.argument]
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ArgumentPlaceholder {
    pub comments: CommentGroup,
    pub left_parenthesis: Span,  // `(`
    pub ellipsis: Span,          // `...`
    pub right_parenthesis: Span, // `)`
}
