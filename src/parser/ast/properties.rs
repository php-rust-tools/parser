use crate::parser::ast::attributes::AttributeGroup;
use crate::parser::ast::modifiers::PropertyModifierGroup;
use crate::parser::ast::variables::Variable;
use crate::parser::ast::Expression;
use crate::parser::ast::Type;

#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    pub attributes: Vec<AttributeGroup>,
    pub r#type: Option<Type>,
    pub modifiers: PropertyModifierGroup,
    pub entries: Vec<PropertyEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyEntry {
    pub variable: Variable,
    pub value: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableProperty {
    pub attributes: Vec<AttributeGroup>,
    pub r#type: Option<Type>,
    pub entries: Vec<VariablePropertyEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariablePropertyEntry {
    pub variable: Variable,
    pub value: Option<Expression>,
}
