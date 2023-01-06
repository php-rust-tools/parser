use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::parser::ast::attributes::AttributeGroup;
use crate::parser::ast::constant::ClassishConstant;
use crate::parser::ast::functions::AbstractConstructor;
use crate::parser::ast::functions::AbstractMethod;
use crate::parser::ast::functions::ConcreteConstructor;
use crate::parser::ast::functions::ConcreteMethod;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::modifiers::ClassModifierGroup;
use crate::parser::ast::properties::Property;
use crate::parser::ast::properties::VariableProperty;
use crate::parser::ast::traits::TraitUsage;
use crate::parser::ast::utils::CommaSeparated;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ClassBody {
    pub left_brace: Span, // `{`
    pub members: Vec<ClassMember>,
    pub right_brace: Span, // `}`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ClassStatement {
    pub attributes: Vec<AttributeGroup>, // `#[Qux]`
    #[serde(flatten)]
    pub modifiers: ClassModifierGroup, // `abstract`, `final`
    pub class: Span,                     // `class`
    pub name: SimpleIdentifier,          // `Foo`
    pub extends: Option<ClassExtends>,   // `extends Foo`
    pub implements: Option<ClassImplements>, // `implements Bar, Baz`
    pub body: ClassBody,                 // `{ ... }`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AnonymousClassBody {
    pub left_brace: Span, // `{`
    pub members: Vec<AnonymousClassMember>,
    pub right_brace: Span, // `}`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AnonymousClass {
    pub attributes: Vec<AttributeGroup>,     // `#[Qux]`
    pub class: Span,                         // `class`
    pub extends: Option<ClassExtends>,       // `extends Foo`
    pub implements: Option<ClassImplements>, // `implements Baz, Baz`
    pub body: AnonymousClassBody,            // `{ ... }`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ClassExtends {
    pub extends: Span,            // `extends`
    pub parent: SimpleIdentifier, // `Foo`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ClassImplements {
    pub implements: Span,                             // `implements`
    pub interfaces: CommaSeparated<SimpleIdentifier>, // `Bar, Baz`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum ClassMember {
    Constant(ClassishConstant),
    TraitUsage(TraitUsage),
    Property(Property),
    VariableProperty(VariableProperty),
    AbstractMethod(AbstractMethod),
    AbstractConstructor(AbstractConstructor),
    ConcreteMethod(ConcreteMethod),
    ConcreteConstructor(ConcreteConstructor),
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum AnonymousClassMember {
    Constant(ClassishConstant),
    TraitUsage(TraitUsage),
    Property(Property),
    VariableProperty(VariableProperty),
    ConcreteMethod(ConcreteMethod),
    ConcreteConstructor(ConcreteConstructor),
}
