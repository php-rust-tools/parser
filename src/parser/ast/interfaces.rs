use crate::lexer::token::Span;
use crate::parser::ast::attributes::AttributeGroup;
use crate::parser::ast::constant::ClassishConstant;
use crate::parser::ast::functions::Method;
use crate::parser::ast::identifiers::SimpleIdentifier;

#[derive(Debug, Clone, PartialEq)]
pub enum InterfaceMember {
    Constant(ClassishConstant),
    Method(Method),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Interface {
    pub start: Span,
    pub end: Span,
    pub attributes: Vec<AttributeGroup>,
    pub name: SimpleIdentifier,
    pub extends: Option<InterfaceExtends>,
    pub members: Vec<InterfaceMember>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InterfaceExtends {
    pub span: Span,
    pub parents: Vec<SimpleIdentifier>,
}
