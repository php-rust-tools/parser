use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Visibility {
    Public,
    Protected,
    Private,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum VisibilityModifier {
    Public(Span),
    Protected(Span),
    Private(Span),
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum PromotedPropertyModifier {
    Public(Span),
    Protected(Span),
    Private(Span),
    Readonly(Span),
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[repr(transparent)]
pub struct PromotedPropertyModifierGroup {
    pub modifiers: Vec<PromotedPropertyModifier>,
}

impl PromotedPropertyModifierGroup {
    pub fn is_empty(&self) -> bool {
        self.modifiers.is_empty()
    }

    pub fn has_readonly(&self) -> bool {
        for modifier in &self.modifiers {
            if matches!(modifier, PromotedPropertyModifier::Readonly { .. }) {
                return true;
            }
        }

        false
    }

    pub fn visibility(&self) -> Visibility {
        for modifier in &self.modifiers {
            if matches!(modifier, PromotedPropertyModifier::Private { .. }) {
                return Visibility::Private;
            }

            if matches!(modifier, PromotedPropertyModifier::Protected { .. }) {
                return Visibility::Protected;
            }
        }

        Visibility::Public
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum PropertyModifier {
    Public(Span),
    Protected(Span),
    Private(Span),
    Static(Span),
    Readonly(Span),
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[repr(transparent)]
pub struct PropertyModifierGroup {
    pub modifiers: Vec<PropertyModifier>,
}

impl PropertyModifierGroup {
    pub fn is_empty(&self) -> bool {
        self.modifiers.is_empty()
    }

    pub fn has_readonly(&self) -> bool {
        for modifier in &self.modifiers {
            if matches!(modifier, PropertyModifier::Readonly { .. }) {
                return true;
            }
        }

        false
    }

    pub fn has_static(&self) -> bool {
        for modifier in &self.modifiers {
            if matches!(modifier, PropertyModifier::Static { .. }) {
                return true;
            }
        }

        false
    }

    pub fn visibility(&self) -> Visibility {
        for modifier in &self.modifiers {
            if matches!(modifier, PropertyModifier::Private { .. }) {
                return Visibility::Private;
            }

            if matches!(modifier, PropertyModifier::Protected { .. }) {
                return Visibility::Protected;
            }
        }

        Visibility::Public
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum MethodModifier {
    Final(Span),
    Static(Span),
    Abstract(Span),
    Public(Span),
    Protected(Span),
    Private(Span),
}

impl MethodModifier {
    pub fn span(&self) -> Span {
        match self {
            MethodModifier::Final(span) => *span,
            MethodModifier::Static(span) => *span,
            MethodModifier::Abstract(span) => *span,
            MethodModifier::Public(span) => *span,
            MethodModifier::Protected(span) => *span,
            MethodModifier::Private(span) => *span,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[repr(transparent)]
pub struct MethodModifierGroup {
    pub modifiers: Vec<MethodModifier>,
}

impl MethodModifierGroup {
    pub fn is_empty(&self) -> bool {
        self.modifiers.is_empty()
    }

    pub fn has_final(&self) -> bool {
        for modifier in &self.modifiers {
            if matches!(modifier, MethodModifier::Final { .. }) {
                return true;
            }
        }

        false
    }

    pub fn has_static(&self) -> bool {
        for modifier in &self.modifiers {
            if matches!(modifier, MethodModifier::Static { .. }) {
                return true;
            }
        }

        false
    }

    pub fn has_abstract(&self) -> bool {
        for modifier in &self.modifiers {
            if matches!(modifier, MethodModifier::Abstract { .. }) {
                return true;
            }
        }

        false
    }

    pub fn get_abstract(&self) -> Option<&MethodModifier> {
        for modifier in &self.modifiers {
            if matches!(modifier, MethodModifier::Abstract { .. }) {
                return Some(modifier);
            }
        }

        None
    }

    pub fn visibility(&self) -> Visibility {
        for modifier in &self.modifiers {
            if matches!(modifier, MethodModifier::Private { .. }) {
                return Visibility::Private;
            }

            if matches!(modifier, MethodModifier::Protected { .. }) {
                return Visibility::Protected;
            }
        }

        Visibility::Public
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum ClassModifier {
    Final(Span),
    Abstract(Span),
    Readonly(Span),
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[repr(transparent)]
pub struct ClassModifierGroup {
    pub modifiers: Vec<ClassModifier>,
}

impl ClassModifierGroup {
    pub fn is_empty(&self) -> bool {
        self.modifiers.is_empty()
    }

    pub fn has_final(&self) -> bool {
        for modifier in &self.modifiers {
            if matches!(modifier, ClassModifier::Final { .. }) {
                return true;
            }
        }

        false
    }

    pub fn has_readonly(&self) -> bool {
        for modifier in &self.modifiers {
            if matches!(modifier, ClassModifier::Readonly { .. }) {
                return true;
            }
        }

        false
    }

    pub fn has_abstract(&self) -> bool {
        for modifier in &self.modifiers {
            if matches!(modifier, ClassModifier::Abstract { .. }) {
                return true;
            }
        }

        false
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum ConstantModifier {
    Final(Span),
    Public(Span),
    Protected(Span),
    Private(Span),
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[repr(transparent)]
pub struct ConstantModifierGroup {
    pub modifiers: Vec<ConstantModifier>,
}

impl ConstantModifierGroup {
    pub fn is_empty(&self) -> bool {
        self.modifiers.is_empty()
    }

    pub fn has_final(&self) -> bool {
        for modifier in &self.modifiers {
            if matches!(modifier, ConstantModifier::Final { .. }) {
                return true;
            }
        }

        false
    }

    pub fn visibility(&self) -> Visibility {
        for modifier in &self.modifiers {
            if matches!(modifier, ConstantModifier::Private { .. }) {
                return Visibility::Private;
            }

            if matches!(modifier, ConstantModifier::Protected { .. }) {
                return Visibility::Protected;
            }
        }

        Visibility::Public
    }
}
