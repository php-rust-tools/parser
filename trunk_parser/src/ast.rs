use serde::Serialize;
use trunk_lexer::TokenKind;

pub type Block = Vec<Statement>;
pub type Program = Block;

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub enum Type {
    Plain(String),
    Nullable(String),
    Union(Vec<String>),
    Intersection(Vec<String>),
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
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

impl From<&str> for Identifier {
    fn from(name: &str) -> Self {
        Self::from(name.to_string())
    }
}

pub type ParamList = Vec<Param>;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Param {
    pub(crate) name: Expression,
    pub(crate) r#type: Option<Type>,
    pub(crate) variadic: bool,
    pub(crate) default: Option<Expression>,
}

impl From<String> for Param {
    fn from(name: String) -> Self {
        Self { name: Expression::Variable(name), r#type: None, variadic: false, default: None }
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

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub enum PropertyFlag {
    Public,
    Protected,
    Private,
    Static,
}

impl From<TokenKind> for PropertyFlag {
    fn from(k: TokenKind) -> Self {
        match k {
            TokenKind::Public => Self::Public,
            TokenKind::Protected => Self::Protected,
            TokenKind::Private => Self::Private,
            TokenKind::Static => Self::Static,
            _ => unreachable!("token {:?} can't be converted into property flag.", k),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub enum MethodFlag {
    Final,
    Abstract,
    Public,
    Protected,
    Private,
    Static,
}

impl From<TokenKind> for MethodFlag {
    fn from(k: TokenKind) -> Self {
        match k {
            TokenKind::Final => Self::Final,
            TokenKind::Abstract => Self::Abstract,
            TokenKind::Public => Self::Public,
            TokenKind::Protected => Self::Protected,
            TokenKind::Private => Self::Private,
            TokenKind::Static => Self::Static,
            _ => unreachable!("token {:?} can't be converted into method flag.", k),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub enum ClassFlag {
    Final,
    Abstract,
}

impl From<TokenKind> for ClassFlag {
    fn from(k: TokenKind) -> Self {
        match k {
            TokenKind::Final => Self::Final,
            TokenKind::Abstract => Self::Abstract,
            _ => unreachable!("token {:?} can't be converted into class flag.", k),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub enum UseKind {
    Normal,
    Function,
    Const,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum Statement {
    InlineHtml(String),
    // TODO: Look at removing this and unifying with Property.
    Foreach {
        expr: Expression,
        key_var: Option<Expression>,
        value_var: Expression,
        body: Block,
    },
    Require {
        path: Expression,
    },
    RequireOnce {
        path: Expression,
    },
    Var {
        var: String,
        value: Option<Expression>,
        r#type: Option<Type>,
    },
    Property {
        var: String,
        value: Option<Expression>,
        r#type: Option<Type>,
        flags: Vec<PropertyFlag>,
    },
    Constant {
        name: Identifier,
        value: Expression,
        flags: Vec<ConstFlag>,
    },
    Function {
        name: Identifier,
        params: Vec<Param>,
        body: Block,
        return_type: Option<Type>,
    },
    Class {
        name: Identifier,
        extends: Option<Identifier>,
        implements: Vec<Identifier>,
        body: Block,
        flag: Option<ClassFlag>,
    },
    Trait {
        name: Identifier,
        body: Block,
    },
    TraitUse {
        traits: Vec<Identifier>,
    },
    Interface {
        name: Identifier,
        extends: Vec<Identifier>,
        body: Block,
    },
    Method {
        name: Identifier,
        params: Vec<Param>,
        body: Block,
        flags: Vec<MethodFlag>,
        return_type: Option<Type>,
    },
    If {
        condition: Expression,
        then: Block,
        else_ifs: Vec<ElseIf>,
        r#else: Option<Block>
    },
    Return {
        value: Option<Expression>,
    },
    Switch {
        condition: Expression,
    },
    Break {
        num: Option<Expression>,
    },
    Continue {
        num: Option<Expression>,
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
        kind: UseKind,
    },
    Comment {
        comment: String,
    },
    Noop,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub enum ConstFlag {
    Final,
    Public,
    Protected,
    Private,
}

impl From<TokenKind> for ConstFlag {
    fn from(k: TokenKind) -> Self {
        match k {
            TokenKind::Final => Self::Final,
            TokenKind::Public => Self::Public,
            TokenKind::Protected => Self::Protected,
            TokenKind::Private => Self::Private,
            _ => unreachable!("token {:?} can't be converted into const flag.", k),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
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
    Closure(Vec<Param>, Vec<Expression>, Option<Type>, Block),
    ArrowFunction(Vec<Param>, Option<Type>, Box<Self>),
    New(Box<Self>, Vec<Self>),
    ConstantString(String),
    PropertyFetch(Box<Self>, Identifier),
    StaticPropertyFetch(Box<Self>, Box<Self>),
    ConstFetch(Box<Self>, Identifier),
    MethodCall(Box<Self>, Identifier, Vec<Self>),
    StaticMethodCall(Box<Self>, Identifier, Vec<Self>),
    AnonymousClass(Option<Identifier>, Vec<Identifier>, Block),
    Bool(bool),
    ArrayIndex(Box<Self>, Option<Box<Self>>),
    Null,
    BooleanNot(Box<Self>),
    MagicConst(MagicConst),
    Ternary(Box<Self>, Box<Self>, Box<Self>),
    Coalesce(Box<Self>, Box<Self>),
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub enum MagicConst {
    Dir,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArrayItem {
    pub key: Option<Expression>,
    pub value: Expression,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub enum InfixOp {
    Add,
    Sub,
    Div,
    Mul,
    Concat,
    LessThan,
    GreaterThan,
    Equals,
    Identical,
    NotEquals,
    NotIdentical,
    And,
    Or,
}

impl From<TokenKind> for InfixOp {
    fn from(k: TokenKind) -> Self {
        match k {
            TokenKind::Plus => Self::Add,
            TokenKind::Minus => Self::Sub,
            TokenKind::Asterisk => Self::Mul,
            TokenKind::Slash => Self::Div,
            TokenKind::LessThan => Self::LessThan,
            TokenKind::GreaterThan => Self::GreaterThan,
            TokenKind::Dot => Self::Concat,
            TokenKind::DoubleEquals => Self::Equals,
            TokenKind::TripleEquals => Self::Identical,
            TokenKind::BangEquals => Self::NotEquals,
            TokenKind::BangDoubleEquals => Self::NotIdentical,
            TokenKind::BooleanAnd => Self::And,
            TokenKind::BooleanOr => Self::Or,
            _ => unreachable!()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ElseIf {
    pub condition: Expression,
    pub body: Block,
}