use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::parser::ast::literals::LiteralInteger;
use crate::parser::ast::utils::CommaSeparated;
use crate::parser::ast::utils::Parenthesized;
use crate::parser::ast::utils::SemicolonTerminated;
use crate::parser::ast::Ending;
use crate::parser::ast::Expression;
use crate::parser::ast::Statement;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ForeachStatement {
    pub foreach: Span,                                     // `foreach`
    pub iterator: Parenthesized<ForeachStatementIterator>, // `( *expression* as & $var => $value )`
    pub body: ForeachStatementBody,                        // `{ ... }`
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
    pub r#for: Span,                                   // `for`
    pub iterator: Parenthesized<ForStatementIterator>, // `( *expression*; *expression*; *expression* )`
    pub body: ForStatementBody,                        // `{ ... }`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ForStatementIterator {
    pub initializations: SemicolonTerminated<CommaSeparated<Expression>>, // `*expression*;`
    pub conditions: SemicolonTerminated<CommaSeparated<Expression>>,      // `*expression*;`
    pub r#loop: CommaSeparated<Expression>,                               // `*expression*`
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
    pub r#do: Span,                                                // `do`
    pub body: Box<Statement>,                                      // `{ ... }`
    pub r#while: Span,                                             // `while`
    pub condition: SemicolonTerminated<Parenthesized<Expression>>, // `( *expression* )`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct WhileStatement {
    pub r#while: Span,                        // `while`
    pub condition: Parenthesized<Expression>, // `( *expression* )`
    pub body: WhileStatementBody,             // `{ ... }`
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
    Parenthesized(Parenthesized<Box<Level>>),
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
