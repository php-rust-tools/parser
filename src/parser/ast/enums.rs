use serde::{Deserialize, Serialize};

use crate::lexer::token::Span;
use crate::parser::ast::attributes::AttributeGroup;
use crate::parser::ast::constant::ClassishConstant;
use crate::parser::ast::functions::Method;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::Expression;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct UnitEnumCase {
    pub start: Span,
    pub end: Span,
    pub name: SimpleIdentifier,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnitEnumMember {
    Case(UnitEnumCase),
    Method(Method),
    Constant(ClassishConstant),
}

#[derive(Debug, PartialEq, Clone)]
pub struct UnitEnum {
    pub start: Span,
    pub end: Span,
    pub name: SimpleIdentifier,
    pub attributes: Vec<AttributeGroup>,
    pub implements: Vec<SimpleIdentifier>,
    pub members: Vec<UnitEnumMember>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum BackedEnumType {
    String(Span),
    Int(Span),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackedEnumCase {
    pub start: Span,
    pub end: Span,
    pub name: SimpleIdentifier,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BackedEnumMember {
    Case(BackedEnumCase),
    Method(Method),
    Constant(ClassishConstant),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackedEnum {
    pub start: Span,
    pub end: Span,
    pub name: SimpleIdentifier,
    pub attributes: Vec<AttributeGroup>,
    pub implements: Vec<SimpleIdentifier>,
    pub backed_type: BackedEnumType,
    pub members: Vec<BackedEnumMember>,
}
