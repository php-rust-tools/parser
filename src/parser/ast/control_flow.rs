use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::parser::ast::Ending;
use crate::parser::ast::Expression;
use crate::parser::ast::Statement;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct IfStatement {
    pub r#if: Span,              // `if`
    pub left_parenthesis: Span,  // `(`
    pub condition: Expression,   // *expression*
    pub right_parenthesis: Span, // `)`
    pub body: IfStatementBody,   // `{ ... }`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum IfStatementBody {
    Statement {
        statement: Box<Statement>,       // `*statement*`
        elseifs: Vec<IfStatementElseIf>, // `elseif (*expression*) *statement*`
        r#else: Option<IfStatementElse>, // `else *statement*`
    },
    Block {
        colon: Span,                          // `:`
        statements: Vec<Statement>,           // `*statements*`
        elseifs: Vec<IfStatementElseIfBlock>, // `elseif (*expression*): *statements*`
        r#else: Option<IfStatementElseBlock>, // `else: *statements*`
        endif: Span,                          // `endif`
        ending: Ending,                       // `;` or `?>`
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct IfStatementElseIf {
    pub elseif: Span,              // `elseif`
    pub left_parenthesis: Span,    // `(`
    pub condition: Expression,     // `( *expression* )`
    pub right_parenthesis: Span,   // `)`
    pub statement: Box<Statement>, // `*statement*`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct IfStatementElse {
    pub r#else: Span,              // `else`
    pub statement: Box<Statement>, // `*statement*`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct IfStatementElseIfBlock {
    pub elseif: Span,               // `elseif`
    pub left_parenthesis: Span,     // `(`
    pub condition: Expression,      // `( *expression* )`
    pub right_parenthesis: Span,    // `)`
    pub colon: Span,                // `:`
    pub statements: Vec<Statement>, // `*statements*`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct IfStatementElseBlock {
    pub r#else: Span,               // `else`
    pub colon: Span,                // `:`
    pub statements: Vec<Statement>, // `*statements*`
}
