use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::node::Node;
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

impl Node for IfStatement {
    fn children(&self) -> Vec<&dyn Node> {
        vec![&self.condition, &self.body]
    }
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

impl Node for IfStatementBody {
    fn children(&self) -> Vec<&dyn Node> {
        match self {
            IfStatementBody::Statement { statement, elseifs, r#else } => {
                let mut children = vec![statement.as_ref()];
                children.extend(elseifs.iter().map(|elseif| elseif as &dyn Node));
                if let Some(r#else) = r#else {
                    children.push(r#else as &dyn Node);
                }
                children
            },
            IfStatementBody::Block { colon, statements, elseifs, r#else, endif, ending } => {
                let mut children = vec![colon, endif, ending];
                children.extend(statements.iter().map(|statement| statement as &dyn Node));
                children.extend(elseifs.iter().map(|elseif| elseif as &dyn Node));
                if let Some(r#else) = r#else {
                    children.push(r#else as &dyn Node);
                }
                children
            },
        }
    }
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

impl Node for IfStatementElseIf {
    fn children(&self) -> Vec<&dyn Node> {
        vec![&self.condition, self.statement.as_ref()]
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct IfStatementElse {
    pub r#else: Span,              // `else`
    pub statement: Box<Statement>, // `*statement*`
}

impl Node for IfStatementElse {
    fn children(&self) -> Vec<&dyn Node> {
        vec![self.statement.as_ref()]
    }
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

impl Node for IfStatementElseIfBlock {
    fn children(&self) -> Vec<&dyn Node> {
        let mut children: Vec<&dyn Node> = vec![&self.condition];
        children.extend(self.statements.iter().map(|statement| statement as &dyn Node));
        children
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct IfStatementElseBlock {
    pub r#else: Span,               // `else`
    pub colon: Span,                // `:`
    pub statements: Vec<Statement>, // `*statements*`
}

impl Node for IfStatementElseBlock {
    fn children(&self) -> Vec<&dyn Node> {
        self.statements.iter().map(|statement| statement as &dyn Node).collect()
    }
}