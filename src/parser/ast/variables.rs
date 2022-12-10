use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::lexer::byte_string::ByteString;
use crate::lexer::token::Span;
use crate::parser::ast::Expression;

#[derive(Debug, PartialEq, Clone)]
pub enum Variable {
    SimpleVariable(SimpleVariable),
    VariableVariable(VariableVariable),
    BracedVariableVariable(BracedVariableVariable),
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct SimpleVariable {
    pub span: Span,
    pub name: ByteString,
}

#[derive(Debug, PartialEq, Clone)]
pub struct VariableVariable {
    pub span: Span,
    pub variable: Box<Variable>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BracedVariableVariable {
    pub start: Span,
    pub variable: Box<Expression>,
    pub end: Span,
}

impl Display for SimpleVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", self.name)
    }
}
