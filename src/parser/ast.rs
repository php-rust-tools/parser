use std::fmt::Display;

use crate::lexer::byte_string::ByteString;
use crate::lexer::token::TokenKind;

pub type Block = Vec<Statement>;
pub type Program = Block;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum TryBlockCaughtType {
    Identifier(Identifier),
    Union(Vec<Identifier>),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Type {
    Identifier(Identifier),
    Nullable(Box<Type>),
    Union(Vec<Type>),
    Intersection(Vec<Type>),
    Void,
    Null,
    True,
    False,
    Never,
    Float,
    Boolean,
    Integer,
    String,
    Array,
    Object,
    Mixed,
    Callable,
    Iterable,
    StaticReference,
    SelfReference,
    ParentReference,
}

impl Type {
    pub fn standalone(&self) -> bool {
        matches!(self, Type::Mixed | Type::Never | Type::Void)
    }

    pub fn nullable(&self) -> bool {
        matches!(self, Type::Nullable(_))
    }

    pub fn includes_callable(&self) -> bool {
        match &self {
            Self::Callable => true,
            Self::Union(types) | Self::Intersection(types) => {
                types.iter().any(|x| x.includes_callable())
            }
            _ => false,
        }
    }

    pub fn includes_class_scoped(&self) -> bool {
        match &self {
            Self::StaticReference | Self::SelfReference | Self::ParentReference => true,
            Self::Union(types) | Self::Intersection(types) => {
                types.iter().any(|x| x.includes_class_scoped())
            }
            _ => false,
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Type::Identifier(inner) => write!(f, "{}", inner),
            Type::Nullable(inner) => write!(f, "{}", inner),
            Type::Union(inner) => write!(
                f,
                "{}",
                inner
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<String>>()
                    .join("|")
            ),
            Type::Intersection(inner) => write!(
                f,
                "{}",
                inner
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<String>>()
                    .join("&")
            ),
            Type::Void => write!(f, "void"),
            Type::Null => write!(f, "null"),
            Type::True => write!(f, "true"),
            Type::False => write!(f, "false"),
            Type::Never => write!(f, "never"),
            Type::Float => write!(f, "float"),
            Type::Boolean => write!(f, "bool"),
            Type::Integer => write!(f, "int"),
            Type::String => write!(f, "string"),
            Type::Array => write!(f, "array"),
            Type::Object => write!(f, "object"),
            Type::Mixed => write!(f, "mixed"),
            Type::Callable => write!(f, "callable"),
            Type::Iterable => write!(f, "iterable"),
            Type::StaticReference => write!(f, "static"),
            Type::SelfReference => write!(f, "self"),
            Type::ParentReference => write!(f, "parent"),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Identifier {
    pub name: ByteString,
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl From<ByteString> for Identifier {
    fn from(name: ByteString) -> Self {
        Self { name }
    }
}

impl From<&ByteString> for Identifier {
    fn from(name: &ByteString) -> Self {
        Self::from(name.clone())
    }
}

pub type ParamList = Vec<Param>;

#[derive(Debug, PartialEq, Clone)]
pub struct Param {
    pub name: Expression,
    pub r#type: Option<Type>,
    pub variadic: bool,
    pub default: Option<Expression>,
    pub flags: Vec<PropertyFlag>,
    pub by_ref: bool,
}

impl From<ByteString> for Param {
    fn from(name: ByteString) -> Self {
        Self {
            name: Expression::Variable { name },
            r#type: None,
            variadic: false,
            default: None,
            flags: vec![],
            by_ref: false,
        }
    }
}

impl From<&ByteString> for Param {
    fn from(name: &ByteString) -> Self {
        Self::from(name.clone())
    }
}

impl From<&[u8]> for Param {
    fn from(name: &[u8]) -> Self {
        Self::from(ByteString::from(name))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PropertyFlag {
    Public,
    Protected,
    Private,
    Static,
    Readonly,
}

impl From<TokenKind> for PropertyFlag {
    fn from(k: TokenKind) -> Self {
        (&k).into()
    }
}

impl From<&TokenKind> for PropertyFlag {
    fn from(k: &TokenKind) -> Self {
        match k {
            TokenKind::Public => Self::Public,
            TokenKind::Protected => Self::Protected,
            TokenKind::Private => Self::Private,
            TokenKind::Static => Self::Static,
            TokenKind::Readonly => Self::Readonly,
            _ => unreachable!("token {:?} can't be converted into property flag.", k),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ClassFlag {
    Final,
    Abstract,
    Readonly,
}

impl From<TokenKind> for ClassFlag {
    fn from(k: TokenKind) -> Self {
        (&k).into()
    }
}

impl From<&TokenKind> for ClassFlag {
    fn from(k: &TokenKind) -> Self {
        match k {
            TokenKind::Final => Self::Final,
            TokenKind::Abstract => Self::Abstract,
            TokenKind::Readonly => Self::Readonly,
            _ => unreachable!("token {:?} can't be converted into class flag.", k),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum UseKind {
    Normal,
    Function,
    Const,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StaticVar {
    pub var: Expression,
    pub default: Option<Expression>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraitAdaptation {
    Alias {
        r#trait: Option<Identifier>,
        method: Identifier,
        alias: Identifier,
        visibility: Option<MethodFlag>,
    },
    Visibility {
        r#trait: Option<Identifier>,
        method: Identifier,
        visibility: MethodFlag,
    },
    Precedence {
        r#trait: Option<Identifier>,
        method: Identifier,
        insteadof: Vec<Identifier>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    InlineHtml(ByteString),
    Goto {
        label: Identifier,
    },
    Label {
        label: Identifier,
    },
    HaltCompiler {
        content: Option<ByteString>,
    },
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
        var: ByteString,
        value: Option<Expression>,
        r#type: Option<Type>,
    },
    Property {
        var: ByteString,
        value: Option<Expression>,
        r#type: Option<Type>,
        flags: Vec<PropertyFlag>,
    },
    Constant {
        constants: Vec<Constant>,
    },
    ClassishConstant {
        name: Identifier,
        value: Expression,
        flags: Vec<ConstFlag>,
    },
    Function {
        name: Identifier,
        params: Vec<Param>,
        body: Block,
        return_type: Option<Type>,
        by_ref: bool,
    },
    Class {
        name: Identifier,
        extends: Option<Identifier>,
        implements: Vec<Identifier>,
        body: Block,
        flags: Vec<ClassFlag>,
    },
    Trait {
        name: Identifier,
        body: Block,
    },
    TraitUse {
        traits: Vec<Identifier>,
        adaptations: Vec<TraitAdaptation>,
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
        by_ref: bool,
    },
    AbstractMethod {
        name: Identifier,
        params: Vec<Param>,
        flags: Vec<MethodFlag>,
        return_type: Option<Type>,
        by_ref: bool,
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
        name: ByteString,
        body: Block,
    },
    BracedNamespace {
        name: Option<ByteString>,
        body: Block,
    },
    Use {
        uses: Vec<Use>,
        kind: UseKind,
    },
    GroupUse {
        prefix: Identifier,
        kind: UseKind,
        uses: Vec<Use>,
    },
    Comment {
        comment: ByteString,
    },
    Try {
        body: Block,
        catches: Vec<Catch>,
        finally: Option<Block>,
    },
    UnitEnum {
        name: Identifier,
        implements: Vec<Identifier>,
        body: Block,
    },
    BackedEnum {
        name: Identifier,
        implements: Vec<Identifier>,
        backed_type: BackedEnumType,
        body: Block,
    },
    UnitEnumCase {
        name: Identifier,
    },
    BackedEnumCase {
        name: Identifier,
        value: Expression,
    },
    Block {
        body: Block,
    },
    Global {
        vars: Vec<Identifier>,
    },
    Declare {
        declares: Vec<DeclareItem>,
        body: Block,
    },
    Noop,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Constant {
    pub name: Identifier,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeclareItem {
    pub key: Identifier,
    pub value: Expression,
}

// See https://www.php.net/manual/en/language.types.type-juggling.php#language.types.typecasting for more info.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CastKind {
    Int,
    Bool,
    Float,
    String,
    Array,
    Object,
    Unset,
}

impl From<TokenKind> for CastKind {
    fn from(kind: TokenKind) -> Self {
        match kind {
            TokenKind::StringCast | TokenKind::BinaryCast => Self::String,
            TokenKind::ObjectCast => Self::Object,
            TokenKind::BoolCast | TokenKind::BooleanCast => Self::Bool,
            TokenKind::IntCast | TokenKind::IntegerCast => Self::Int,
            TokenKind::FloatCast | TokenKind::DoubleCast | TokenKind::RealCast => Self::Float,
            TokenKind::UnsetCast => Self::Unset,
            TokenKind::ArrayCast => Self::Array,
            _ => unreachable!(),
        }
    }
}

impl From<&TokenKind> for CastKind {
    fn from(kind: &TokenKind) -> Self {
        kind.clone().into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BackedEnumType {
    String,
    Int,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Case {
    pub condition: Option<Expression>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Catch {
    pub types: TryBlockCaughtType,
    pub var: Option<Expression>,
    pub body: Block,
}

#[derive(Debug, Clone, Eq, PartialEq)]
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Use {
    pub name: Identifier,
    pub alias: Option<Identifier>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Empty,
    VariadicPlaceholder,
    ErrorSuppress {
        expr: Box<Self>,
    },
    Increment {
        value: Box<Self>,
    },
    Decrement {
        value: Box<Self>,
    },
    LiteralInteger {
        i: i64,
    },
    LiteralFloat {
        f: f64,
    },
    Variable {
        name: ByteString,
    },
    DynamicVariable {
        name: Box<Self>,
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
        name: ByteString,
    },
    Static,
    Array {
        items: Vec<ArrayItem>,
    },
    Closure {
        params: Vec<Param>,
        uses: Vec<ClosureUse>,
        return_type: Option<Type>,
        body: Block,
        r#static: bool,
        by_ref: bool,
    },
    ArrowFunction {
        params: Vec<Param>,
        return_type: Option<Type>,
        expr: Box<Self>,
        by_ref: bool,
        r#static: bool,
    },
    New {
        target: Box<Self>,
        args: Vec<Arg>,
    },
    LiteralString {
        value: ByteString,
    },
    InterpolatedString {
        parts: Vec<StringPart>,
    },
    PropertyFetch {
        target: Box<Self>,
        property: Box<Self>,
    },
    NullsafePropertyFetch {
        target: Box<Self>,
        property: Box<Self>,
    },
    NullsafeMethodCall {
        target: Box<Self>,
        method: Box<Self>,
        args: Vec<Arg>,
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
        method: Box<Self>,
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
        key: Option<Box<Self>>,
        value: Option<Box<Self>>,
    },
    YieldFrom {
        value: Box<Self>,
    },
    Negate {
        value: Box<Self>,
    },
    UnaryPlus {
        value: Box<Self>,
    },
    BitwiseNot {
        value: Box<Self>,
    },
    PreDecrement {
        value: Box<Self>,
    },
    PreIncrement {
        value: Box<Self>,
    },
    Print {
        value: Box<Self>,
    },
    Cast {
        kind: CastKind,
        value: Box<Self>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct Arg {
    pub name: Option<ByteString>,
    pub value: Expression,
    pub unpack: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClosureUse {
    pub var: Expression,
    pub by_ref: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MatchArm {
    pub conditions: Option<Vec<Expression>>,
    pub body: Expression,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum MagicConst {
    Dir,
}

#[derive(Debug, PartialEq, Clone)]
pub enum StringPart {
    Const(ByteString),
    Expr(Box<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayItem {
    pub key: Option<Expression>,
    pub value: Expression,
    pub unpack: bool,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum InfixOp {
    Add,
    Sub,
    Div,
    Mul,
    Mod,
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
    LeftShift,
    RightShift,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LogicalAnd,
    LogicalOr,
    LogicalXor,
    Spaceship,
    PowAssign,
    ModAssign,
    BitwiseAndAssign,
    BitwiseOrAssign,
    BitwiseXorAssign,
    LeftShiftAssign,
    RightShiftAssign,
}

impl From<TokenKind> for InfixOp {
    fn from(k: TokenKind) -> Self {
        match k {
            TokenKind::Percent => Self::Mod,
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
            TokenKind::BangEquals | TokenKind::AngledLeftRight => Self::NotEquals,
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
            TokenKind::LeftShift => Self::LeftShift,
            TokenKind::RightShift => Self::RightShift,
            TokenKind::Ampersand => Self::BitwiseAnd,
            TokenKind::Pipe => Self::BitwiseOr,
            TokenKind::Caret => Self::BitwiseXor,
            TokenKind::Spaceship => Self::Spaceship,
            TokenKind::LogicalAnd => Self::LogicalAnd,
            TokenKind::LogicalOr => Self::LogicalOr,
            TokenKind::LogicalXor => Self::LogicalXor,
            TokenKind::PowEquals => Self::PowAssign,
            TokenKind::PercentEquals => Self::ModAssign,
            TokenKind::AmpersandEquals => Self::BitwiseAndAssign,
            TokenKind::PipeEquals => Self::BitwiseOrAssign,
            TokenKind::CaretEquals => Self::BitwiseXorAssign,
            TokenKind::LeftShiftEquals => Self::LeftShiftAssign,
            TokenKind::RightShiftEquals => Self::RightShiftAssign,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElseIf {
    pub condition: Expression,
    pub body: Block,
}
