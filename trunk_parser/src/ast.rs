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
    Void,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub struct Identifier {
    pub name: String,
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
    pub name: Expression,
    pub r#type: Option<Type>,
    pub variadic: bool,
    pub default: Option<Expression>,
    pub flag: Option<PropertyFlag>,
}

impl From<String> for Param {
    fn from(name: String) -> Self {
        Self {
            name: Expression::Variable { name },
            r#type: None,
            variadic: false,
            default: None,
            flag: None,
        }
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
pub struct StaticVar {
    pub var: Expression,
    pub default: Option<Expression>,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum IncludeKind {
    Include,
    IncludeOnce,
    Require,
    RequireOnce,
}

impl From<&TokenKind> for IncludeKind {
    fn from(k: &TokenKind) -> Self {
        match k {
            TokenKind::Include => IncludeKind::Include,
            TokenKind::IncludeOnce => IncludeKind::IncludeOnce,
            TokenKind::Require => IncludeKind::Require,
            TokenKind::RequireOnce => IncludeKind::RequireOnce,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum Statement {
    InlineHtml(String),
    Static {
        vars: Vec<StaticVar>,
    },
    DoWhile {
        condition: Expression,
        body: Block,
    },
    While {
        condition: Expression,
        body: Block,
    },
    For {
        init: Option<Expression>,
        condition: Option<Expression>,
        r#loop: Option<Expression>,
        then: Block,
    },
    Foreach {
        expr: Expression,
        by_ref: bool,
        key_var: Option<Expression>,
        value_var: Expression,
        body: Block,
    },
    Include {
        kind: IncludeKind,
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
        r#else: Option<Block>,
    },
    Return {
        value: Option<Expression>,
    },
    Switch {
        condition: Expression,
        cases: Vec<Case>,
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
    Try {
        body: Block,
        catches: Vec<Catch>,
        finally: Option<Block>,
    },
    Enum {
        name: Identifier,
        implements: Vec<Identifier>,
        backed_type: Option<BackedEnumType>,
        body: Block,
    },
    EnumCase {
        name: Identifier,
        value: Option<Expression>,
    },
    Block {
        body: Block,
    },
    Noop,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub enum CastKind {
    String,
    Object,
    Bool,
    Int,
    Double,
}

impl From<TokenKind> for CastKind {
    fn from(kind: TokenKind) -> Self {
        match kind {
            TokenKind::StringCast => Self::String,
            TokenKind::ObjectCast => Self::Object,
            TokenKind::BoolCast => Self::Bool,
            TokenKind::IntCast => Self::Int,
            TokenKind::DoubleCast => Self::Double,
            _ => unreachable!(),
        }
    }
}

impl From<&TokenKind> for CastKind {
    fn from(kind: &TokenKind) -> Self {
        kind.clone().into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub enum BackedEnumType {
    String,
    Int,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Case {
    pub condition: Option<Expression>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Catch {
    pub types: Vec<Identifier>,
    pub var: Expression,
    pub body: Block,
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
    Static,
    ErrorSuppress {
        expr: Box<Self>,
    },
    Increment {
        value: Box<Self>,
    },
    Decrement {
        value: Box<Self>,
    },
    Int {
        i: i64,
    },
    Float {
        f: f64,
    },
    Variable {
        name: String,
    },
    Infix {
        lhs: Box<Self>,
        op: InfixOp,
        rhs: Box<Self>,
    },
    Call {
        target: Box<Self>,
        args: Vec<Arg>,
    },
    Identifier {
        name: String,
    },
    Array {
        items: Vec<ArrayItem>,
    },
    Closure {
        params: Vec<Param>,
        uses: Vec<ClosureUse>,
        return_type: Option<Type>,
        body: Block,
    },
    ArrowFunction {
        params: Vec<Param>,
        return_type: Option<Type>,
        expr: Box<Self>,
    },
    New {
        target: Box<Self>,
        args: Vec<Arg>,
    },
    ConstantString {
        value: String,
    },
    PropertyFetch {
        target: Box<Self>,
        property: Box<Self>,
    },
    NullsafePropertyFetch {
        target: Box<Self>,
        property: Box<Self>,
    },
    StaticPropertyFetch {
        target: Box<Self>,
        property: Box<Self>,
    },
    ConstFetch {
        target: Box<Self>,
        constant: Identifier,
    },
    MethodCall {
        target: Box<Self>,
        method: Box<Self>,
        args: Vec<Arg>,
    },
    StaticMethodCall {
        target: Box<Self>,
        method: Identifier,
        args: Vec<Arg>,
    },
    AnonymousClass {
        extends: Option<Identifier>,
        implements: Vec<Identifier>,
        body: Block,
    },
    Bool {
        value: bool,
    },
    ArrayIndex {
        array: Box<Self>,
        index: Option<Box<Self>>,
    },
    Null,
    BooleanNot {
        value: Box<Self>,
    },
    MagicConst {
        constant: MagicConst,
    },
    Ternary {
        condition: Box<Self>,
        then: Option<Box<Self>>,
        r#else: Box<Self>,
    },
    Coalesce {
        lhs: Box<Self>,
        rhs: Box<Self>,
    },
    Clone {
        target: Box<Self>,
    },
    Match {
        condition: Box<Self>,
        arms: Vec<MatchArm>,
    },
    Throw {
        value: Box<Expression>,
    },
    Yield {
        value: Box<Expression>,
    },
    Negate {
        value: Box<Expression>,
    },
    Cast {
        kind: CastKind,
        value: Box<Self>,
    },
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Arg {
    pub name: Option<String>,
    pub value: Expression,
    pub unpack: bool,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct ClosureUse {
    pub var: Expression,
    pub by_ref: bool,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct MatchArm {
    pub conditions: Option<Vec<Expression>>,
    pub body: Expression,
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
    ConcatAssign,
    LessThan,
    GreaterThan,
    LessThanEquals,
    GreaterThanEquals,
    Equals,
    Identical,
    NotEquals,
    NotIdentical,
    And,
    Or,
    Assign,
    AddAssign,
    Pow,
    Instanceof,
    CoalesceAssign,
    MulAssign,
    SubAssign,
    DivAssign,
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
            TokenKind::LessThanEquals => Self::LessThanEquals,
            TokenKind::GreaterThanEquals => Self::GreaterThanEquals,
            TokenKind::Dot => Self::Concat,
            TokenKind::DotEquals => Self::ConcatAssign,
            TokenKind::DoubleEquals => Self::Equals,
            TokenKind::TripleEquals => Self::Identical,
            TokenKind::BangEquals => Self::NotEquals,
            TokenKind::BangDoubleEquals => Self::NotIdentical,
            TokenKind::BooleanAnd => Self::And,
            TokenKind::BooleanOr => Self::Or,
            TokenKind::Equals => Self::Assign,
            TokenKind::PlusEquals => Self::AddAssign,
            TokenKind::Pow => Self::Pow,
            TokenKind::Instanceof => Self::Instanceof,
            TokenKind::CoalesceEqual => Self::CoalesceAssign,
            TokenKind::AsteriskEqual => Self::MulAssign,
            TokenKind::MinusEquals => Self::SubAssign,
            TokenKind::SlashEquals => Self::DivAssign,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ElseIf {
    pub condition: Expression,
    pub body: Block,
}
