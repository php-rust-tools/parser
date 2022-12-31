use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::parser::ast::literals::LiteralInteger;
use crate::parser::ast::utils::CommaSeparated;
use crate::parser::ast::Ending;
use crate::parser::ast::Expression;
use crate::parser::ast::Statement;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ForeachStatement {
    pub foreach: Span,                      // `foreach`
    pub left_parenthesis: Span,             // `(`
    pub iterator: ForeachStatementIterator, // `( *expression* as & $var => $value )`
    pub right_parenthesis: Span,            // `)`
    pub body: ForeachStatementBody,         // `{ ... }`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ForeachStatementIterator {
    // `*expression* as &$var`
    Value {
        expression: Expression,  // `*expression*`
        r#as: Span,              // `as`
        ampersand: Option<Span>, // `&`
        value: Expression,       // `$var`
    },
    // `*expression* as &$key => $value`
    KeyAndValue {
        expression: Expression,  // `*expression*`
        r#as: Span,              // `as`
        ampersand: Option<Span>, // `&`
        key: Expression,         // `$key`
        double_arrow: Span,      // `=>`
        value: Expression,       // `$value`
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum ForeachStatementBody {
    Statement(Box<Statement>),
    Block {
        colon: Span,                // `:`
        statements: Vec<Statement>, // `*statements*`
        endforeach: Span,           // `endforeach`
        ending: Ending,             // `;` or `?>`
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ForStatement {
    pub r#for: Span,                    // `for`
    pub left_parenthesis: Span,         // `(`
    pub iterator: ForStatementIterator, // `*expression*; *expression*; *expression*`
    pub right_parenthesis: Span,        // `)`
    pub body: ForStatementBody,         // `{ ... }`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ForStatementIterator {
    pub initializations: CommaSeparated<Expression>, // `*expression*;`
    pub initializations_semicolon: Span,             // `;`
    pub conditions: CommaSeparated<Expression>,      // `*expression*;`
    pub conditions_semicolon: Span,                  // `;`
    pub r#loop: CommaSeparated<Expression>,          // `*expression*`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum ForStatementBody {
    Statement(Box<Statement>),
    Block {
        colon: Span,                // `:`
        statements: Vec<Statement>, // `*statements*`
        endfor: Span,               // `endfor`
        ending: Ending,             // `;` or `?>`
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DoWhileStatement {
    pub r#do: Span,              // `do`
    pub body: Box<Statement>,    // `{ ... }`
    pub r#while: Span,           // `while`
    pub left_parenthesis: Span,  // `(`
    pub condition: Expression,   // `( *expression* )`
    pub right_parenthesis: Span, // `)`
    pub semicolon: Span,         // `;`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct WhileStatement {
    pub r#while: Span,            // `while`
    pub left_parenthesis: Span,   // `(`
    pub condition: Expression,    // *expression*
    pub right_parenthesis: Span,  // `)`
    pub body: WhileStatementBody, // `{ ... }`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum WhileStatementBody {
    Statement(Box<Statement>),
    Block {
        colon: Span,                // `:`
        statements: Vec<Statement>, // `*statements*`
        endwhile: Span,             // `endwhile`
        ending: Ending,             // `;` or `?>`
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum Level {
    Literal(LiteralInteger),
    Parenthesized {
        left_parenthesis: Span, // `(`
        level: Box<Level>,
        right_parenthesis: Span, // `)`
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BreakStatement {
    pub r#break: Span,        // `break`
    pub level: Option<Level>, // `3`
    pub ending: Ending,       // `;` or `?>`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ContinueStatement {
    pub r#continue: Span,     // `continue`
    pub level: Option<Level>, // `2`
    pub ending: Ending,       // `;` or `?>`
}
