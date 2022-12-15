use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::parser::ast::attributes::AttributeGroup;
use crate::parser::ast::comments::CommentGroup;
use crate::parser::ast::data_type::Type;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::modifiers::MethodModifierGroup;
use crate::parser::ast::modifiers::PromotedPropertyModifierGroup;
use crate::parser::ast::variables::SimpleVariable;
use crate::parser::ast::Block;
use crate::parser::ast::Expression;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct FunctionParameter {
    pub comments: CommentGroup,
    pub name: SimpleVariable,
    pub attributes: Vec<AttributeGroup>,
    pub r#type: Option<Type>,
    pub ellipsis: Option<Span>,
    pub default: Option<Expression>,
    pub ampersand: Option<Span>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct FunctionParameterList {
    pub comments: CommentGroup,
    pub left_parenthesis: Span,
    pub members: Vec<FunctionParameter>,
    pub right_parenthesis: Span,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Function {
    pub comments: CommentGroup,
    pub start: Span,
    pub end: Span,
    pub name: SimpleIdentifier,
    pub attributes: Vec<AttributeGroup>,
    pub parameters: FunctionParameterList,
    pub return_type: Option<Type>,
    pub ampersand: Option<Span>,
    pub body: Block,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ClosureUse {
    pub comments: CommentGroup,
    pub variable: SimpleVariable,
    pub ampersand: Option<Span>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Closure {
    pub comments: CommentGroup,
    pub start: Span,
    pub end: Span,
    pub attributes: Vec<AttributeGroup>,
    pub parameters: FunctionParameterList,
    pub return_ty: Option<Type>,
    pub uses: Vec<ClosureUse>,
    pub ampersand: Option<Span>,
    pub body: Block,
    pub r#static: Option<Span>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ArrowFunction {
    pub comments: CommentGroup,
    pub r#static: Option<Span>,
    pub ampersand: Option<Span>,
    pub r#fn: Span,
    pub attributes: Vec<AttributeGroup>,
    pub parameters: FunctionParameterList,
    pub return_type: Option<Type>,
    pub body: Box<Expression>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MethodParameter {
    pub attributes: Vec<AttributeGroup>,
    pub comments: CommentGroup,
    pub ampersand: Option<Span>,
    pub name: SimpleVariable,
    pub r#type: Option<Type>,
    pub ellipsis: Option<Span>,
    pub default: Option<Expression>,
    #[serde(flatten)]
    pub modifiers: PromotedPropertyModifierGroup,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MethodParameterList {
    pub comments: CommentGroup,
    pub start: Span,
    pub members: Vec<MethodParameter>,
    pub end: Span,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Method {
    pub comments: CommentGroup,
    pub start: Span,
    pub end: Span,
    pub name: SimpleIdentifier,
    pub attributes: Vec<AttributeGroup>,
    pub parameters: MethodParameterList,
    pub body: Option<Block>,
    #[serde(flatten)]
    pub modifiers: MethodModifierGroup,
    pub return_type: Option<Type>,
    pub ampersand: Option<Span>,
}
