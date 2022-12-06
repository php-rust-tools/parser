use crate::lexer::token::Span;
use crate::parser::ast::Expression;

#[derive(Debug, PartialEq, Clone)]
pub struct Attribute {
    pub start: Span,
    pub end: Span,
    pub expression: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct AttributeGroup {
    pub start: Span,
    pub end: Span,
    pub members: Vec<Attribute>,
}
