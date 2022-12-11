use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::parser::ast::attributes::AttributeGroup;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::modifiers::ConstantModifierGroup;
use crate::parser::ast::Expression;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ConstantEntry {
    pub name: SimpleIdentifier,
    pub value: Expression,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Constant {
    pub start: Span,
    pub end: Span,
    pub entries: Vec<ConstantEntry>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ClassishConstant {
    pub start: Span,
    pub end: Span,
    pub attributes: Vec<AttributeGroup>,
    pub modifiers: ConstantModifierGroup,
    pub entries: Vec<ConstantEntry>,
}
