use serde::Deserialize;
use serde::Serialize;

use crate::parser::ast::attributes::AttributeGroup;
use crate::parser::ast::modifiers::PropertyModifierGroup;
use crate::parser::ast::variables::SimpleVariable;
use crate::parser::ast::Expression;
use crate::parser::ast::Type;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Property {
    pub attributes: Vec<AttributeGroup>,
    pub r#type: Option<Type>,
    pub modifiers: PropertyModifierGroup,
    pub entries: Vec<PropertyEntry>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PropertyEntry {
    pub variable: SimpleVariable,
    pub value: Option<Expression>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct VariableProperty {
    pub attributes: Vec<AttributeGroup>,
    pub r#type: Option<Type>,
    pub entries: Vec<VariablePropertyEntry>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct VariablePropertyEntry {
    pub variable: SimpleVariable,
    pub value: Option<Expression>,
}
