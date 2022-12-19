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
use crate::parser::ast::utils::CommaSeparated;
use crate::parser::ast::variables::SimpleVariable;
use crate::parser::ast::Expression;
use crate::parser::ast::Statement;

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
pub struct FunctionBody {
    pub comments: CommentGroup,
    pub left_brace: Span,
    pub statements: Vec<Statement>,
    pub right_brace: Span,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Function {
    pub comments: CommentGroup,
    pub attributes: Vec<AttributeGroup>,
    pub function: Span,
    pub ampersand: Option<Span>,
    pub name: SimpleIdentifier,
    pub parameters: FunctionParameterList,
    pub return_type: Option<Type>,
    pub body: FunctionBody,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ClosureUseVariable {
    pub comments: CommentGroup,
    pub ampersand: Option<Span>,
    pub variable: SimpleVariable,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ClosureUse {
    pub comments: CommentGroup,
    pub r#use: Span,
    pub left_parenthesis: Span,
    pub variables: CommaSeparated<ClosureUseVariable>,
    pub right_parenthesis: Span,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Closure {
    pub comments: CommentGroup,
    pub attributes: Vec<AttributeGroup>,
    pub r#static: Option<Span>,
    pub function: Span,
    pub ampersand: Option<Span>,
    pub parameters: FunctionParameterList,
    pub uses: Option<ClosureUse>,
    pub return_ty: Option<Type>,
    pub body: FunctionBody,
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
    pub double_arrow: Span,
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
    pub left_parenthesis: Span,
    pub members: Vec<MethodParameter>,
    pub right_parenthesis: Span,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Method {
    pub comments: CommentGroup,
    pub attributes: Vec<AttributeGroup>,
    #[serde(flatten)]
    pub modifiers: MethodModifierGroup,
    pub function: Span,
    pub ampersand: Option<Span>,
    pub name: SimpleIdentifier,
    pub parameters: MethodParameterList,
    pub return_type: Option<Type>,
    pub body: MethodBody,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MethodBody {
    Abstract(Span), // `;`
    Block {
        left_brace: Span, // `{`
        statements: Vec<Statement>,
        right_brace: Span, // `}`
    },
}
