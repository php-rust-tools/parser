pub mod attributes;
pub mod comments;
pub mod constant;
pub mod enums;
pub mod functions;
pub mod identifiers;
pub mod interfaces;
pub mod modifiers;
pub mod try_block;
pub mod variables;

use std::fmt::Display;

use crate::lexer::byte_string::ByteString;
use crate::lexer::token::Span;
use crate::lexer::token::TokenKind;
use crate::parser::ast::attributes::AttributeGroup;
use crate::parser::ast::comments::Comment;
use crate::parser::ast::constant::ClassishConstant;
use crate::parser::ast::constant::Constant;
use crate::parser::ast::enums::BackedEnum;
use crate::parser::ast::enums::UnitEnum;
use crate::parser::ast::functions::ArrowFunction;
use crate::parser::ast::functions::Closure;
use crate::parser::ast::functions::Function;
use crate::parser::ast::functions::Method;
use crate::parser::ast::identifiers::Identifier;
use crate::parser::ast::interfaces::Interface;
use crate::parser::ast::modifiers::ClassModifierGroup;
use crate::parser::ast::modifiers::PropertyModifierGroup;
use crate::parser::ast::modifiers::VisibilityModifier;
use crate::parser::ast::try_block::TryBlock;
use crate::parser::ast::variables::Variable;

pub type Block = Vec<Statement>;
pub type Program = Block;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Type {
    Identifier(Identifier),
    // TODO: add `start` and `end` for all types.
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

    pub fn is_bottom(&self) -> bool {
        matches!(self, Type::Never | Type::Void)
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
pub enum UseKind {
    Normal,
    Function,
    Const,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StaticVar {
    pub var: Variable,
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
        visibility: Option<VisibilityModifier>,
    },
    Visibility {
        r#trait: Option<Identifier>,
        method: Identifier,
        visibility: VisibilityModifier,
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
        init: Vec<Expression>,
        condition: Vec<Expression>,
        r#loop: Vec<Expression>,
        then: Block,
    },
    Foreach {
        expr: Expression,
        by_ref: bool,
        key_var: Option<Expression>,
        value_var: Expression,
        body: Block,
    },
    Property {
        var: Variable,
        attributes: Vec<AttributeGroup>,
        value: Option<Expression>,
        r#type: Option<Type>,
        modifiers: PropertyModifierGroup,
    },
    Constant(Constant),
    ClassishConstant(ClassishConstant),
    Function(Function),
    Class {
        name: Identifier,
        attributes: Vec<AttributeGroup>,
        extends: Option<Identifier>,
        implements: Vec<Identifier>,
        body: Block,
        modifiers: ClassModifierGroup,
    },
    Trait {
        name: Identifier,
        attributes: Vec<AttributeGroup>,
        body: Block,
    },
    TraitUse {
        traits: Vec<Identifier>,
        adaptations: Vec<TraitAdaptation>,
    },
    Interface(Interface),
    Method(Method),
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
        name: Identifier,
        body: Block,
    },
    BracedNamespace {
        name: Option<Identifier>,
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
    Comment(Comment),
    Try(TryBlock),
    UnitEnum(UnitEnum),
    BackedEnum(BackedEnum),
    Block {
        body: Block,
    },
    Global {
        vars: Vec<Variable>,
    },
    Declare {
        declares: Vec<DeclareItem>,
        body: Block,
    },
    Noop(Span),
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Use {
    pub name: Identifier,
    pub alias: Option<Identifier>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    List {
        items: Vec<ListItem>,
    },
    Empty,
    VariadicPlaceholder,
    ErrorSuppress {
        span: Span,
        expr: Box<Self>,
    },
    Increment {
        value: Box<Self>,
    },
    Decrement {
        value: Box<Self>,
    },
    LiteralInteger {
        i: ByteString,
    },
    LiteralFloat {
        f: ByteString,
    },
    Variable(Variable),
    DynamicVariable {
        name: Box<Self>,
    },
    Infix {
        lhs: Box<Self>,
        op: InfixOp,
        rhs: Box<Self>,
    },
    Include {
        span: Span,
        kind: IncludeKind,
        path: Box<Expression>,
    },
    Call {
        target: Box<Self>,
        args: Vec<Arg>,
    },
    Identifier(Identifier),
    Static,
    Self_,
    Parent,
    Array {
        items: Vec<ArrayItem>,
    },
    Closure(Closure),
    ArrowFunction(ArrowFunction),
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
    Heredoc {
        parts: Vec<StringPart>,
    },
    Nowdoc {
        value: ByteString,
    },
    ShellExec {
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
        attributes: Vec<AttributeGroup>,
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
        span: Span,
        value: Box<Self>,
    },
    MagicConst {
        span: Span,
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
        default: Option<Box<DefaultMatchArm>>,
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
        span: Span,
        value: Box<Self>,
    },
    UnaryPlus {
        span: Span,
        value: Box<Self>,
    },
    BitwiseNot {
        span: Span,
        value: Box<Self>,
    },
    PreDecrement {
        span: Span,
        value: Box<Self>,
    },
    PreIncrement {
        span: Span,
        value: Box<Self>,
    },
    Print {
        span: Span,
        value: Box<Self>,
    },
    Cast {
        span: Span,
        kind: CastKind,
        value: Box<Self>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct Arg {
    pub name: Option<Identifier>,
    pub value: Expression,
    pub unpack: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClosureUse {
    pub var: Expression,
    pub by_ref: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DefaultMatchArm {
    pub body: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MatchArm {
    pub conditions: Vec<Expression>,
    pub body: Expression,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum MagicConst {
    Directory,
    File,
    Line,
    Class,
    Function,
    Method,
    Namespace,
    Trait,
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
    pub by_ref: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    pub key: Option<Expression>,
    pub value: Expression,
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
    AssignRef,
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
