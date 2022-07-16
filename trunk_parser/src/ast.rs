use trunk_lexer::TokenKind;

pub type Block = Vec<Statement>;
pub type Program = Block;

#[derive(Debug, PartialEq, Clone)]
pub struct Identifier {
    name: String,
}

impl From<String> for Identifier {
    fn from(name: String) -> Self {
        Self { name }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Param {
    name: Expression,
}

impl From<String> for Param {
    fn from(name: String) -> Self {
        Self { name: Expression::Variable(name) }
    }
}

impl From<&str> for Param {
    fn from(name: &str) -> Self {
        Self::from(name.to_string())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    InlineHtml(String),
    Function {
        name: Identifier,
        params: Vec<Param>,
        body: Block,
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
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Int(i64),
    Variable(String),
    Infix(Box<Self>, InfixOp, Box<Self>),
    Call(Box<Self>, Vec<Self>),
    Identifier(String),
}

#[derive(Debug, PartialEq, Clone)]
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