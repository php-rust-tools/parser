use crate::lexer::token::Span;
use crate::parser::ast::attributes::AttributeGroup;
use crate::parser::ast::constant::ClassishConstant;
use crate::parser::ast::functions::Method;
use crate::parser::ast::identifiers::Identifier;

#[derive(Debug, Clone, PartialEq)]
pub enum InterfaceMember {
    Constant(ClassishConstant),
    Method(Method),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Interface {
    pub start: Span,
    pub end: Span,
    pub name: Identifier,
    pub attributes: Vec<AttributeGroup>,
    pub extends: Vec<Identifier>,
    pub members: Vec<InterfaceMember>,
}
