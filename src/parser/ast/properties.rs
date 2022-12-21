use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::parser::ast::attributes::AttributeGroup;
use crate::parser::ast::data_type::Type;
use crate::parser::ast::modifiers::PropertyModifierGroup;
use crate::parser::ast::variables::SimpleVariable;
use crate::parser::ast::Expression;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Property {
    pub attributes: Vec<AttributeGroup>,
    #[serde(flatten)]
    pub modifiers: PropertyModifierGroup,
    pub r#type: Option<Type>,
    pub entries: Vec<PropertyEntry>,
    pub end: Span,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct VariableProperty {
    pub attributes: Vec<AttributeGroup>,
    pub r#type: Option<Type>,
    pub entries: Vec<PropertyEntry>,
    pub end: Span,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum PropertyEntry {
    Uninitialized {
        variable: SimpleVariable,
    },
    Initialized {
        variable: SimpleVariable,
        equals: Span,
        value: Expression,
    },
}
