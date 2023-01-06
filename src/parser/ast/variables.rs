use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use std::fmt::Display;

use crate::lexer::byte_string::ByteString;
use crate::lexer::token::Span;
use crate::node::Node;
use crate::parser::ast::Expression;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum Variable {
    SimpleVariable(SimpleVariable),
    VariableVariable(VariableVariable),
    BracedVariableVariable(BracedVariableVariable),
}

impl Node for Variable {
    fn children(&self) -> Vec<&dyn Node> {
        match self {
            Variable::SimpleVariable(variable) => variable.children(),
            Variable::VariableVariable(variable) => variable.children(),
            Variable::BracedVariableVariable(variable) => variable.children(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SimpleVariable {
    pub span: Span,
    pub name: ByteString,
}

impl Node for SimpleVariable {
    //
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct VariableVariable {
    pub span: Span,
    pub variable: Box<Variable>,
}

impl Node for VariableVariable {
    fn children(&self) -> Vec<&dyn Node> {
        vec![&self.variable]
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BracedVariableVariable {
    pub start: Span,
    pub variable: Box<Expression>,
    pub end: Span,
}

impl Node for BracedVariableVariable {
    fn children(&self) -> Vec<&dyn Node> {
        vec![&self.variable]
    }
}

impl Display for SimpleVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
