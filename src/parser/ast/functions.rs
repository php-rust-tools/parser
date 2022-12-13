use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::parser::ast::attributes::AttributeGroup;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::modifiers::MethodModifierGroup;
use crate::parser::ast::modifiers::PromotedPropertyModifierGroup;
use crate::parser::ast::variables::SimpleVariable;
use crate::parser::ast::Block;
use crate::parser::ast::Expression;
use crate::parser::ast::Type;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct FunctionParameter {
    pub start: Span,
    pub end: Span,
    pub name: SimpleVariable,
    pub attributes: Vec<AttributeGroup>,
    pub r#type: Option<Type>,
    pub variadic: bool,
    pub default: Option<Expression>,
    pub by_ref: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct FunctionParameterList {
    pub start: Span,
    pub end: Span,
    pub members: Vec<FunctionParameter>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Function {
    pub start: Span,
    pub end: Span,
    pub name: SimpleIdentifier,
    pub attributes: Vec<AttributeGroup>,
    pub parameters: FunctionParameterList,
    pub return_type: Option<Type>,
    pub by_ref: bool,
    pub body: Block,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ClosureUse {
    pub var: Expression,
    pub by_ref: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Closure {
    pub start: Span,
    pub end: Span,
    pub attributes: Vec<AttributeGroup>,
    pub parameters: FunctionParameterList,
    pub return_ty: Option<Type>,
    pub uses: Vec<ClosureUse>,
    pub by_ref: bool,
    pub body: Block,
    pub r#static: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ArrowFunction {
    pub start: Span,
    pub end: Span,
    pub attributes: Vec<AttributeGroup>,
    pub parameters: FunctionParameterList,
    pub return_type: Option<Type>,
    pub by_ref: bool,
    pub body: Box<Expression>,
    pub r#static: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MethodParameter {
    pub start: Span,
    pub end: Span,
    pub name: SimpleVariable,
    pub attributes: Vec<AttributeGroup>,
    pub r#type: Option<Type>,
    pub variadic: bool,
    pub default: Option<Expression>,
    #[serde(flatten)]
    pub modifiers: PromotedPropertyModifierGroup,
    pub by_ref: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MethodParameterList {
    pub start: Span,
    pub end: Span,
    pub members: Vec<MethodParameter>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Method {
    pub start: Span,
    pub end: Span,
    pub name: SimpleIdentifier,
    pub attributes: Vec<AttributeGroup>,
    pub parameters: MethodParameterList,
    pub body: Option<Block>,
    #[serde(flatten)]
    pub modifiers: MethodModifierGroup,
    pub return_type: Option<Type>,
    pub by_ref: bool,
}
