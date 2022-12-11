use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::parser::ast::attributes::AttributeGroup;
use crate::parser::ast::constant::ClassishConstant;
use crate::parser::ast::functions::Method;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::properties::Property;
use crate::parser::ast::properties::VariableProperty;
use crate::parser::ast::traits::TraitUsage;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Class {
    pub start: Span,
    pub end: Span,
    pub name: SimpleIdentifier,
    pub extends: Option<ClassExtends>,
    pub implements: Option<ClassImplements>,
    pub attributes: Vec<AttributeGroup>,
    pub members: Vec<ClassMember>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AnonymousClass {
    pub start: Span,
    pub end: Span,
    pub extends: Option<ClassExtends>,
    pub implements: Option<ClassImplements>,
    pub attributes: Vec<AttributeGroup>,
    pub members: Vec<ClassMember>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ClassExtends {
    pub span: Span,
    pub parent: SimpleIdentifier,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ClassImplements {
    pub span: Span,
    pub interfaces: Vec<SimpleIdentifier>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClassMember {
    Constant(ClassishConstant),
    TraitUsage(TraitUsage),
    Property(Property),
    VariableProperty(VariableProperty),
    Method(Method),
}
