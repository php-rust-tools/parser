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
use crate::parser::ast::modifiers::VisibilityModifier;
use crate::parser::ast::properties::Property;
use crate::parser::ast::properties::VariableProperty;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum TraitMember {
    Constant(ClassishConstant),
    TraitUsage(TraitUsage),
    Property(Property),
    VariableProperty(VariableProperty),
    AbstractMethod(AbstractMethod),
    AbstractConstructor(AbstractConstructor),
    ConcreteMethod(ConcreteMethod),
    ConcreteConstructor(ConcreteConstructor),
}

impl Node for TraitMember {
    fn children(&self) -> Vec<&dyn Node> {
        match self {
            TraitMember::Constant(constant) => vec![constant],
            TraitMember::TraitUsage(usage) => vec![usage],
            TraitMember::Property(property) => vec![property],
            TraitMember::VariableProperty(property) => vec![property],
            TraitMember::AbstractMethod(method) => vec![method],
            TraitMember::AbstractConstructor(constructor) => vec![constructor],
            TraitMember::ConcreteMethod(method) => vec![method],
            TraitMember::ConcreteConstructor(constructor) => vec![constructor],
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TraitBody {
    pub left_brace: Span,
    pub members: Vec<TraitMember>,
    pub right_brace: Span,
}

impl Node for TraitBody {
    fn children(&self) -> Vec<&dyn Node> {
        self.members.iter().map(|member| member as &dyn Node).collect()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TraitStatement {
    pub r#trait: Span,
    pub name: SimpleIdentifier,
    pub attributes: Vec<AttributeGroup>,
    pub body: TraitBody,
}

impl Node for TraitStatement {
    fn children(&self) -> Vec<&dyn Node> {
        vec![&self.name, &self.body]
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TraitUsage {
    pub r#use: Span,
    pub traits: Vec<SimpleIdentifier>,
    pub adaptations: Vec<TraitUsageAdaptation>,
}

impl Node for TraitUsage {
    fn children(&self) -> Vec<&dyn Node> {
        self.traits.iter().map(|t| t as &dyn Node).collect()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum TraitUsageAdaptation {
    Alias {
        r#trait: Option<SimpleIdentifier>,
        method: SimpleIdentifier,
        alias: SimpleIdentifier,
        visibility: Option<VisibilityModifier>,
    },
    Visibility {
        r#trait: Option<SimpleIdentifier>,
        method: SimpleIdentifier,
        visibility: VisibilityModifier,
    },
    Precedence {
        r#trait: Option<SimpleIdentifier>,
        method: SimpleIdentifier,
        insteadof: Vec<SimpleIdentifier>,
    },
}
