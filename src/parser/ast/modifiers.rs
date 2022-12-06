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
    pub flags: Vec<PromotedPropertyModifier>,
}

impl PromotedPropertyModifierGroup {
    pub fn is_empty(&self) -> bool {
        self.flags.is_empty()
    }

    pub fn has_readonly(&self) -> bool {
        for flag in &self.flags {
            if matches!(flag, PromotedPropertyModifier::Readonly { .. }) {
                return true;
            }
        }

        false
    }

    pub fn visibility(&self) -> Visibility {
        for flag in &self.flags {
            if matches!(flag, PromotedPropertyModifier::Private { .. }) {
                return Visibility::Private;
            }

            if matches!(flag, PromotedPropertyModifier::Protected { .. }) {
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
    pub flags: Vec<PropertyModifier>,
}

impl PropertyModifierGroup {
    pub fn is_empty(&self) -> bool {
        self.flags.is_empty()
    }

    pub fn has_readonly(&self) -> bool {
        for flag in &self.flags {
            if matches!(flag, PropertyModifier::Readonly { .. }) {
                return true;
            }
        }

        false
    }

    pub fn has_static(&self) -> bool {
        for flag in &self.flags {
            if matches!(flag, PropertyModifier::Static { .. }) {
                return true;
            }
        }

        false
    }

    pub fn visibility(&self) -> Visibility {
        for flag in &self.flags {
            if matches!(flag, PropertyModifier::Private { .. }) {
                return Visibility::Private;
            }

            if matches!(flag, PropertyModifier::Protected { .. }) {
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
    pub flags: Vec<MethodModifier>,
}

impl MethodModifierGroup {
    pub fn is_empty(&self) -> bool {
        self.flags.is_empty()
    }

    pub fn has_final(&self) -> bool {
        for flag in &self.flags {
            if matches!(flag, MethodModifier::Final { .. }) {
                return true;
            }
        }

        false
    }

    pub fn has_static(&self) -> bool {
        for flag in &self.flags {
            if matches!(flag, MethodModifier::Static { .. }) {
                return true;
            }
        }

        false
    }

    pub fn has_abstract(&self) -> bool {
        for flag in &self.flags {
            if matches!(flag, MethodModifier::Abstract { .. }) {
                return true;
            }
        }

        false
    }

    pub fn visibility(&self) -> Visibility {
        for flag in &self.flags {
            if matches!(flag, MethodModifier::Private { .. }) {
                return Visibility::Private;
            }

            if matches!(flag, MethodModifier::Protected { .. }) {
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
    pub flags: Vec<ClassModifier>,
}

impl ClassModifierGroup {
    pub fn is_empty(&self) -> bool {
        self.flags.is_empty()
    }

    pub fn has_final(&self) -> bool {
        for flag in &self.flags {
            if matches!(flag, ClassModifier::Final { .. }) {
                return true;
            }
        }

        false
    }

    pub fn has_readonly(&self) -> bool {
        for flag in &self.flags {
            if matches!(flag, ClassModifier::Readonly { .. }) {
                return true;
            }
        }

        false
    }

    pub fn has_abstract(&self) -> bool {
        for flag in &self.flags {
            if matches!(flag, ClassModifier::Abstract { .. }) {
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
    pub flags: Vec<ConstantModifier>,
}

impl ConstantModifierGroup {
    pub fn is_empty(&self) -> bool {
        self.flags.is_empty()
    }

    pub fn has_final(&self) -> bool {
        for flag in &self.flags {
            if matches!(flag, ConstantModifier::Final { .. }) {
                return true;
            }
        }

        false
    }

    pub fn visibility(&self) -> Visibility {
        for flag in &self.flags {
            if matches!(flag, ConstantModifier::Private { .. }) {
                return Visibility::Private;
            }

            if matches!(flag, ConstantModifier::Protected { .. }) {
                return Visibility::Protected;
            }
        }

        Visibility::Public
    }
}
