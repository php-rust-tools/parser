use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::parser::ast::attributes::AttributeGroup;
use crate::parser::ast::constant::ClassishConstant;
use crate::parser::ast::functions::Method;
use crate::parser::ast::identifiers::SimpleIdentifier;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InterfaceMember {
    Constant(ClassishConstant),
    Method(Method),
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Interface {
    pub start: Span,
    pub end: Span,
    pub attributes: Vec<AttributeGroup>,
    pub name: SimpleIdentifier,
    pub extends: Option<InterfaceExtends>,
    pub members: Vec<InterfaceMember>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct InterfaceExtends {
    pub span: Span,
    pub parents: Vec<SimpleIdentifier>,
}
