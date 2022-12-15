use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::parser::ast::attributes::AttributeGroup;
use crate::parser::ast::comments::CommentGroup;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::modifiers::ConstantModifierGroup;
use crate::parser::ast::Expression;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ConstantEntry {
    pub name: SimpleIdentifier, // `FOO`
    pub span: Span,             // `=`
    pub value: Expression,      // `123`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Constant {
    pub comments: CommentGroup,
    pub start: Span,                 // `const`
    pub entries: Vec<ConstantEntry>, // `FOO = 123`
    pub end: Span,                   // `;`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ClassishConstant {
    pub comments: CommentGroup,
    pub attributes: Vec<AttributeGroup>,  // `#[Foo]`
    pub modifiers: ConstantModifierGroup, // `public`
    pub start: Span,                      // `const`
    #[serde(flatten)]
    pub entries: Vec<ConstantEntry>, // `FOO = 123`
    pub end: Span,                        // `;`
}
