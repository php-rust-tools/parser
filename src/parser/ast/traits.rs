use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::parser::ast::attributes::AttributeGroup;
use crate::parser::ast::constant::ClassishConstant;
use crate::parser::ast::functions::Method;
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
    Method(Method),
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TraitBody {
    pub start: Span,
    pub end: Span,
    pub members: Vec<TraitMember>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Trait {
    pub span: Span,
    pub name: SimpleIdentifier,
    pub attributes: Vec<AttributeGroup>,
    pub body: TraitBody,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TraitUsage {
    pub span: Span,
    pub traits: Vec<SimpleIdentifier>,
    pub adaptations: Vec<TraitUsageAdaptation>,
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
