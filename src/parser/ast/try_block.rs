use serde::{Deserialize, Serialize};

use crate::lexer::token::Span;
use crate::parser::ast::identifiers::Identifier;
use crate::parser::ast::Block;
use crate::parser::ast::Expression;

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub enum CatchType {
    Identifier(Identifier),
    Union(Vec<Identifier>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TryBlock {
    pub start: Span,
    pub end: Span,
    pub body: Block,
    pub catches: Vec<CatchBlock>,
    pub finally: Option<FinallyBlock>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatchBlock {
    pub start: Span,
    pub end: Span,
    pub types: CatchType,
    pub var: Option<Expression>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FinallyBlock {
    pub start: Span,
    pub end: Span,
    pub body: Block,
}
