use crate::lexer::token::Span;
use crate::parser::ast::identifiers::Identifier;
use crate::parser::ast::Type;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TemplateVariance {
    Covariance(Span),
    Contravariance(Span),
    Invaraint,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TemplateTypeConstraint {
    SubType(Span, Type),
    SuperType(Span, Type),
    Equal(Span, Type),
    None,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Template {
    pub name: Identifier,
    pub variance: TemplateVariance,
    pub constraint: TemplateTypeConstraint,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TemplateGroup {
    pub start: Span,
    pub end: Span,
    pub members: Vec<Template>,
}
