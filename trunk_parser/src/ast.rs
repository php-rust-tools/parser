use serde::Serialize;
use trunk_lexer::TokenKind;

pub type Block = Vec<Statement>;
pub type Program = Block;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum Type {
    Plain(String),
    Nullable(Box<Type>),
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Identifier {
    name: String,
}

impl From<String> for Identifier {
    fn from(name: String) -> Self {
        Self { name }
    }
}

impl From<&String> for Identifier {
    fn from(name: &String) -> Self {
        Self::from(name.to_string())
    }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Param {
    pub(crate) name: Expression,
    pub(crate) r#type: Option<Type>,
}

impl From<String> for Param {
    fn from(name: String) -> Self {
        Self { name: Expression::Variable(name), r#type: None }
    }
}

impl From<&String> for Param {
    fn from(name: &String) -> Self {
        Self::from(name.to_string())
    }
}

impl From<&str> for Param {
    fn from(name: &str) -> Self {
        Self::from(name.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum MethodFlag {
    Public,
    Protected,
    Private,
    Static,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ClassFlag {
    Final,
    Abstract,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum Statement {
    InlineHtml(String),
    Var {
        var: String,
    },
    Property {
        var: String,
    },
    Function {
        name: Identifier,
        params: Vec<Param>,
        body: Block,
    },
    Class {
        name: Identifier,
        extends: Option<Identifier>,
        implements: Vec<Identifier>,
        body: Block,
        flag: Option<ClassFlag>,
    },
    Method {
        name: Identifier,
        params: Vec<Param>,
        body: Block,
        flags: Vec<MethodFlag>,
    },
    If {
        condition: Expression,
        then: Block,
    },
    Return {
        value: Option<Expression>,
    },
    Echo {
        values: Vec<Expression>,
    },
    Expression {
        expr: Expression,
    },
    Namespace {
        name: String,
        body: Block,
    },
    Use {
        uses: Vec<Use>,
    },
    Comment {
        comment: String,
    },
    Noop,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Use {
    pub name: Identifier,
    pub alias: Option<Identifier>,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum Expression {
    Int(i64),
    Variable(String),
    Infix(Box<Self>, InfixOp, Box<Self>),
    Call(Box<Self>, Vec<Self>),
    Identifier(String),
    Assign(Box<Self>, Box<Self>),
    Array(Vec<ArrayItem>),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArrayItem {
    pub key: Option<Expression>,
    pub value: Expression,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum InfixOp {
    Add,
    Sub,
    LessThan,
}

impl From<TokenKind> for InfixOp {
    fn from(k: TokenKind) -> Self {
        match k {
            TokenKind::Plus => Self::Add,
            TokenKind::Minus => Self::Sub,
            TokenKind::LessThan => Self::LessThan,
            _ => unreachable!()
        }
    }
}