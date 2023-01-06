use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::node::Node;
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

impl Node for ForeachStatement {
    fn children(&self) -> Vec<&dyn Node> {
        vec![&self.iterator, &self.body]
    }
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

impl Node for ForeachStatementIterator {
    fn children(&self) -> Vec<&dyn Node> {
        match self {
            ForeachStatementIterator::Value { expression, value, .. } => {
                vec![expression, value]
            }
            ForeachStatementIterator::KeyAndValue {
                expression,
                key,
                value,
                ..
            } => vec![expression, key, value],
        }
    }
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

impl Node for ForeachStatementBody {
    fn children(&self) -> Vec<&dyn Node> {
        match self {
            ForeachStatementBody::Statement(statement) => vec![statement.as_ref() as &dyn Node],
            ForeachStatementBody::Block { statements, .. } => {
                statements.iter().map(|s| s as &dyn Node).collect()
            }
        }
    }
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

impl Node for ForStatement {
    fn children(&self) -> Vec<&dyn Node> {
        vec![&self.iterator, &self.body]
    }
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

impl Node for ForStatementIterator {
    fn children(&self) -> Vec<&dyn Node> {
        let mut children = vec![];
        children.extend(self.initializations.inner.iter().map(|x| x as &dyn Node));
        children.extend(self.conditions.inner.iter().map(|x| x as &dyn Node));
        children.extend(self.r#loop.inner.iter().map(|x| x as &dyn Node));
        children
    }
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

impl Node for ForStatementBody {
    fn children(&self) -> Vec<&dyn Node> {
        match self {
            ForStatementBody::Statement(statement) => vec![statement],
            ForStatementBody::Block { statements, .. } => statements.iter().map(|x| x as &dyn Node).collect(),
        }
    }
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

impl Node for DoWhileStatement {
    fn children(&self) -> Vec<&dyn Node> {
        vec![self.body.as_ref() as &dyn Node, &self.condition]
    }
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

impl Node for WhileStatement {
    fn children(&self) -> Vec<&dyn Node> {
        vec![&self.condition, &self.body]
    }
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

impl Node for WhileStatementBody {
    fn children(&self) -> Vec<&dyn Node> {
        match self {
            WhileStatementBody::Statement(statement) => vec![statement.as_ref() as &dyn Node],
            WhileStatementBody::Block { statements, .. } => statements.iter().map(|s| s as &dyn Node).collect(),
        }
    }
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

impl Node for Level {
    fn children(&self) -> Vec<&dyn Node> {
        match self {
            Level::Literal(literal) => vec![literal],
            Level::Parenthesized { left_parenthesis, level, right_parenthesis } => level.children(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BreakStatement {
    pub r#break: Span,        // `break`
    pub level: Option<Level>, // `3`
    pub ending: Ending,       // `;` or `?>`
}

impl Node for BreakStatement {
    fn children(&self) -> Vec<&dyn Node> {
        match &self.level {
            Some(level) => vec![level],
            None => vec![],
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ContinueStatement {
    pub r#continue: Span,     // `continue`
    pub level: Option<Level>, // `2`
    pub ending: Ending,       // `;` or `?>`
}

impl Node for ContinueStatement {
    fn children(&self) -> Vec<&dyn Node> {
        match &self.level {
            Some(level) => vec![level],
            None => vec![],
        }
    }
}