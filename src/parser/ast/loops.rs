use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::parser::ast::utils::Braced;
use crate::parser::ast::utils::CommaSeparated;
use crate::parser::ast::utils::Parenthesized;
use crate::parser::ast::utils::SemicolonTerminated;
use crate::parser::ast::Expression;
use crate::parser::ast::Statement;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ForeachLoop {
    pub foreach: Span,                            // `foreach`
    pub iterator: Parenthesized<ForeachIterator>, // `( *expression* as & $var => $value )`
    pub body: ForeachBody,                        // `{ ... }`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ForeachIterator {
    // `&` `*` `$` `identifier`
    Value {
        expression: Expression,  // `*expression*`
        r#as: Span,              // `as`
        ampersand: Option<Span>, // `&`
        value: Expression,       // `$var`
    },
    // `&` `*` `$` `identifier` `=>` `$` `identifier`
    KeyAndValue {
        expression: Expression,  // `*expression*`
        r#as: Span,              // `as`
        ampersand: Option<Span>, // `&`
        key: Expression,         // `$key`
        arrow: Span,             // `=>`
        value: Expression,       // `$value`
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum ForeachBody {
    Statement(Box<Statement>),
    Braced(Braced<Vec<Statement>>),
    Block {
        colon: Span,                // `:`
        statements: Vec<Statement>, // `*statements*`
        endforeach: Span,           // `endforeach`
        semicolon: Span,            // `;`
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ForLoop {
    pub r#for: Span,                          // `for`
    pub iterator: Parenthesized<ForIterator>, // `( *expression*; *expression*; *expression* )`
    pub body: ForBody,                        // `{ ... }`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ForIterator {
    pub initializations: SemicolonTerminated<CommaSeparated<Expression>>, // `*expression*;`
    pub conditions: SemicolonTerminated<CommaSeparated<Expression>>,      // `*expression*;`
    pub r#loop: CommaSeparated<Expression>,                               // `*expression*`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum ForBody {
    Statement(Box<Statement>),
    Braced(Braced<Vec<Statement>>),
    Block {
        colon: Span,                // `:`
        statements: Vec<Statement>, // `*statements*`
        endfor: Span,               // `endfor`
        semicolon: Span,            // `;`
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DoWhileLoop {
    pub r#do: Span,                                                // `do`
    pub body: DoWhileBody,                                         // `{ ... }`
    pub r#while: Span,                                             // `while`
    pub condition: SemicolonTerminated<Parenthesized<Expression>>, // `( *expression* )`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum DoWhileBody {
    Statement(Box<Statement>),
    Braced(Braced<Vec<Statement>>),
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct WhileLoop {
    pub r#while: Span,                        // `while`
    pub condition: Parenthesized<Expression>, // `( *expression* )`
    pub body: WhileBody,                      // `{ ... }`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum WhileBody {
    Statement(Box<Statement>),
    Braced(Braced<Vec<Statement>>),
    Block {
        colon: Span,                // `:`
        statements: Vec<Statement>, // `*statements*`
        endwhile: Span,             // `endwhile`
        semicolon: Span,            // `;`
    },
}
