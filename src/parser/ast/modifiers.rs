use serde::{Deserialize, Serialize};

use crate::lexer::token::Span;

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum Visibility {
    Public,
    Protected,
    Private,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum VisibilityModifier {
    Public { start: Span, end: Span },
    Protected { start: Span, end: Span },
    Private { start: Span, end: Span },
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum PromotedPropertyModifier {
    Public { start: Span, end: Span },
    Protected { start: Span, end: Span },
    Private { start: Span, end: Span },
    Readonly { start: Span, end: Span },
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum PropertyModifier {
    Public { start: Span, end: Span },
    Protected { start: Span, end: Span },
    Private { start: Span, end: Span },
    Static { start: Span, end: Span },
    Readonly { start: Span, end: Span },
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum MethodModifier {
    Final { start: Span, end: Span },
    Static { start: Span, end: Span },
    Abstract { start: Span, end: Span },
    Public { start: Span, end: Span },
    Protected { start: Span, end: Span },
    Private { start: Span, end: Span },
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum ClassModifier {
    Final { start: Span, end: Span },
    Abstract { start: Span, end: Span },
    Readonly { start: Span, end: Span },
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum ConstantModifier {
    Final { start: Span, end: Span },
    Public { start: Span, end: Span },
    Protected { start: Span, end: Span },
    Private { start: Span, end: Span },
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
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
