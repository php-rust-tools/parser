use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::node::Node;
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

impl Node for ClassBody {
    fn children(&self) -> Vec<&dyn Node> {
        self.members
            .iter()
            .map(|member| member as &dyn Node)
            .collect()
    }
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

impl Node for ClassStatement {
    fn children(&self) -> Vec<&dyn Node> {
        let mut children: Vec<&dyn Node> = vec![&self.name];
        if let Some(extends) = &self.extends {
            children.push(extends);
        }
        if let Some(implements) = &self.implements {
            children.push(implements);
        }
        children.push(&self.body);
        children
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AnonymousClassBody {
    pub left_brace: Span, // `{`
    pub members: Vec<AnonymousClassMember>,
    pub right_brace: Span, // `}`
}

impl Node for AnonymousClassBody {
    fn children(&self) -> Vec<&dyn Node> {
        self.members
            .iter()
            .map(|member| member as &dyn Node)
            .collect()
    }
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

impl Node for AnonymousClass {
    fn children(&self) -> Vec<&dyn Node> {
        let mut children: Vec<&dyn Node> = vec![];
        if let Some(extends) = &self.extends {
            children.push(extends);
        }
        if let Some(implements) = &self.implements {
            children.push(implements);
        }
        children.push(&self.body);
        children
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ClassExtends {
    pub extends: Span,            // `extends`
    pub parent: SimpleIdentifier, // `Foo`
}

impl Node for ClassExtends {
    fn children(&self) -> Vec<&dyn Node> {
        vec![&self.parent]
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ClassImplements {
    pub implements: Span,                             // `implements`
    pub interfaces: CommaSeparated<SimpleIdentifier>, // `Bar, Baz`
}

impl Node for ClassImplements {
    fn children(&self) -> Vec<&dyn Node> {
        self.interfaces.children()
    }
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

impl Node for ClassMember {
    fn children(&self) -> Vec<&dyn Node> {
        match self {
            ClassMember::Constant(constant) => vec![constant],
            ClassMember::TraitUsage(usage) => vec![usage],
            ClassMember::Property(property) => vec![property],
            ClassMember::VariableProperty(property) => vec![property],
            ClassMember::AbstractMethod(method) => vec![method],
            ClassMember::AbstractConstructor(method) => vec![method],
            ClassMember::ConcreteMethod(method) => vec![method],
            ClassMember::ConcreteConstructor(method) => vec![method],
        }
    }
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

impl Node for AnonymousClassMember {
    fn children(&self) -> Vec<&dyn Node> {
        match self {
            AnonymousClassMember::Constant(constant) => vec![constant],
            AnonymousClassMember::TraitUsage(usage) => vec![usage],
            AnonymousClassMember::Property(property) => vec![property],
            AnonymousClassMember::VariableProperty(property) => vec![property],
            AnonymousClassMember::ConcreteMethod(method) => vec![method],
            AnonymousClassMember::ConcreteConstructor(method) => vec![method],
        }
    }
}
