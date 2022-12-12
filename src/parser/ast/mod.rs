use serde::Deserialize;
use serde::Serialize;

use std::fmt::Display;

use crate::lexer::byte_string::ByteString;
use crate::lexer::token::Span;
use crate::lexer::token::TokenKind;
use crate::parser::ast::classes::AnonymousClass;
use crate::parser::ast::classes::Class;
use crate::parser::ast::comments::Comment;
use crate::parser::ast::constant::Constant;
use crate::parser::ast::declares::Declare;
use crate::parser::ast::enums::BackedEnum;
use crate::parser::ast::enums::UnitEnum;
use crate::parser::ast::functions::ArrowFunction;
use crate::parser::ast::functions::Closure;
use crate::parser::ast::functions::Function;
use crate::parser::ast::identifiers::Identifier;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::interfaces::Interface;
use crate::parser::ast::namespaces::Namespace;
use crate::parser::ast::operators::ArithmeticOperation;
use crate::parser::ast::operators::AssignmentOperation;
use crate::parser::ast::operators::BitwiseOperation;
use crate::parser::ast::operators::ComparisonOperation;
use crate::parser::ast::operators::LogicalOperation;
use crate::parser::ast::traits::Trait;
use crate::parser::ast::try_block::TryBlock;
use crate::parser::ast::variables::Variable;

pub mod attributes;
pub mod classes;
pub mod comments;
pub mod constant;
pub mod declares;
pub mod enums;
pub mod functions;
pub mod identifiers;
pub mod interfaces;
pub mod modifiers;
pub mod namespaces;
pub mod operators;
pub mod properties;
pub mod traits;
pub mod try_block;
pub mod variables;

pub type Block = Vec<Statement>;
pub type Program = Block;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum Type {
    Identifier(SimpleIdentifier),
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

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum UseKind {
    Normal,
    Function,
    Const,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StaticVar {
    pub var: Variable,
    pub default: Option<Expression>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
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

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum Statement {
    InlineHtml(ByteString),
    Goto {
        label: SimpleIdentifier,
    },
    Label {
        label: SimpleIdentifier,
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
    Constant(Constant),
    Function(Function),
    Class(Class),
    Trait(Trait),
    Interface(Interface),
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
    ShortEcho {
        span: Span,
        values: Vec<Expression>,
    },
    Echo {
        values: Vec<Expression>,
    },
    Expression {
        expression: Expression,
        end: Span,
    },
    Namespace(Namespace),
    Use {
        uses: Vec<Use>,
        kind: UseKind,
    },
    GroupUse {
        prefix: SimpleIdentifier,
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
        span: Span,
        variables: Vec<Variable>,
    },
    Declare(Declare),
    Noop(Span),
}

// See https://www.php.net/manual/en/language.types.type-juggling.php#language.types.typecasting for more info.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
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

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Case {
    pub condition: Option<Expression>,
    pub body: Block,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Use {
    pub name: SimpleIdentifier,
    pub alias: Option<SimpleIdentifier>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum Expression {
    Die {
        value: Option<Box<Expression>>,
    },
    Exit {
        value: Option<Box<Expression>>,
    },
    ArithmeticOperation(ArithmeticOperation),
    AssignmentOperation(AssignmentOperation),
    BitwiseOperation(BitwiseOperation),
    ComparisonOperation(ComparisonOperation),
    LogicalOperation(LogicalOperation),
    Concat {
        left: Box<Expression>,
        span: Span,
        right: Box<Expression>,
    },
    Instanceof {
        left: Box<Expression>,
        span: Span,
        right: Box<Expression>,
    },
    Reference {
        span: Span,
        right: Box<Expression>,
    },
    Parenthesized {
        start: Span,
        expr: Box<Expression>,
        end: Span,
    },
    List {
        items: Vec<ListItem>,
    },
    Empty,
    VariadicPlaceholder,
    ErrorSuppress {
        span: Span,
        expr: Box<Self>,
    },
    LiteralInteger {
        span: Span,
        value: ByteString,
    },
    LiteralFloat {
        span: Span,
        value: ByteString,
    },
    Identifier(Identifier),
    Variable(Variable),
    Include {
        span: Span,
        kind: IncludeKind,
        path: Box<Expression>,
    },
    Call {
        target: Box<Self>,
        args: Vec<Arg>,
    },
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
        span: Span,
        args: Vec<Arg>,
    },
    LiteralString {
        span: Span,
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
        constant: SimpleIdentifier,
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
    AnonymousClass(AnonymousClass),
    Bool {
        value: bool,
    },
    ArrayIndex {
        array: Box<Self>,
        index: Option<Box<Self>>,
    },
    Null,
    MagicConst {
        span: Span,
        constant: MagicConst,
    },
    Ternary {
        condition: Box<Self>,
        then: Box<Self>,
        r#else: Box<Self>,
    },
    ShortTernary {
        condition: Box<Self>,
        span: Span,
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
    BitwiseNot {
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

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Arg {
    pub name: Option<SimpleIdentifier>,
    pub value: Expression,
    pub unpack: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ClosureUse {
    pub var: Expression,
    pub by_ref: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DefaultMatchArm {
    pub body: Expression,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct MatchArm {
    pub conditions: Vec<Expression>,
    pub body: Expression,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
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

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum StringPart {
    Literal(ByteString),
    Expression(Box<Expression>),
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ArrayItem {
    pub key: Option<Expression>,
    pub value: Expression,
    pub unpack: bool,
    pub by_ref: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ListItem {
    pub key: Option<Expression>,
    pub value: Expression,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ElseIf {
    pub condition: Expression,
    pub body: Block,
}
