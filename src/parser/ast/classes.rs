use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::parser::ast::attributes::AttributeGroup;
use crate::parser::ast::constant::ClassishConstant;
use crate::parser::ast::functions::Method;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::modifiers::ClassModifierGroup;
use crate::parser::ast::properties::Property;
use crate::parser::ast::properties::VariableProperty;
use crate::parser::ast::traits::TraitUsage;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ClassBody {
    pub start: Span,
    pub end: Span,
    pub members: Vec<ClassMember>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Class {
    pub span: Span,
    pub name: SimpleIdentifier,
    #[serde(flatten)]
    pub modifiers: ClassModifierGroup,
    pub extends: Option<ClassExtends>,
    pub implements: Option<ClassImplements>,
    pub attributes: Vec<AttributeGroup>,
    pub body: ClassBody,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AnonymousClass {
    pub attributes: Vec<AttributeGroup>,     // `#[Foo]`
    pub span: Span,                          // `class`
    pub extends: Option<ClassExtends>,       // `extends Foo, Bar`
    pub implements: Option<ClassImplements>, // `implements Foo, Bar`
    pub body: ClassBody,                     // `{ ... }`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ClassExtends {
    pub span: Span,
    pub parent: SimpleIdentifier,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ClassImplements {
    pub span: Span,
    pub interfaces: Vec<SimpleIdentifier>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum ClassMember {
    Constant(ClassishConstant),
    TraitUsage(TraitUsage),
    Property(Property),
    VariableProperty(VariableProperty),
    Method(Method),
}
