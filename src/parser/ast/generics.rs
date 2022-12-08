use crate::lexer::token::Span;
use crate::parser::ast::Type;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Generic {
    pub r#type: Type,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GenericGroup {
    pub start: Span,
    pub end: Span,
    pub members: Vec<Generic>,
}
