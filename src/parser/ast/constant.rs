use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::parser::ast::attributes::AttributeGroup;
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
    pub start: Span,                 // `const`
    pub end: Span,                   // `;`
    pub entries: Vec<ConstantEntry>, // `FOO = 123`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ClassishConstant {
    pub start: Span,                     // `const`
    pub end: Span,                       // `;`
    pub attributes: Vec<AttributeGroup>, // `#[Foo]`
    #[serde(flatten)]
    pub modifiers: ConstantModifierGroup, // `public`
    pub entries: Vec<ConstantEntry>,     // `FOO = 123`
}
