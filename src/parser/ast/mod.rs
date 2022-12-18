use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::byte_string::ByteString;
use crate::lexer::token::Span;
use crate::lexer::token::TokenKind;
use crate::parser::ast::arguments::ArgumentList;
use crate::parser::ast::arguments::ArgumentPlaceholder;
use crate::parser::ast::classes::AnonymousClass;
use crate::parser::ast::classes::Class;
use crate::parser::ast::comments::Comment;
use crate::parser::ast::constant::Constant;
use crate::parser::ast::control_flow::IfStatement;
use crate::parser::ast::declares::Declare;
use crate::parser::ast::enums::BackedEnum;
use crate::parser::ast::enums::UnitEnum;
use crate::parser::ast::functions::ArrowFunction;
use crate::parser::ast::functions::Closure;
use crate::parser::ast::functions::Function;
use crate::parser::ast::goto::GotoLabel;
use crate::parser::ast::goto::GotoStatement;
use crate::parser::ast::identifiers::Identifier;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::interfaces::Interface;
use crate::parser::ast::loops::BreakStatement;
use crate::parser::ast::loops::ContinueStatement;
use crate::parser::ast::loops::DoWhileStatement;
use crate::parser::ast::loops::ForStatement;
use crate::parser::ast::loops::ForeachStatement;
use crate::parser::ast::loops::WhileStatement;
use crate::parser::ast::namespaces::Namespace;
use crate::parser::ast::operators::ArithmeticOperation;
use crate::parser::ast::operators::AssignmentOperation;
use crate::parser::ast::operators::BitwiseOperation;
use crate::parser::ast::operators::ComparisonOperation;
use crate::parser::ast::operators::LogicalOperation;
use crate::parser::ast::traits::Trait;
use crate::parser::ast::try_block::TryBlock;
use crate::parser::ast::utils::Braced;
use crate::parser::ast::utils::Bracketed;
use crate::parser::ast::utils::CommaSeparated;
use crate::parser::ast::utils::Parenthesized;
use crate::parser::ast::variables::Variable;

pub mod arguments;
pub mod attributes;
pub mod classes;
pub mod comments;
pub mod constant;
pub mod control_flow;
pub mod data_type;
pub mod declares;
pub mod enums;
pub mod functions;
pub mod goto;
pub mod identifiers;
pub mod interfaces;
pub mod loops;
pub mod modifiers;
pub mod namespaces;
pub mod operators;
pub mod properties;
pub mod traits;
pub mod try_block;
pub mod utils;
pub mod variables;

pub type Block = Vec<Statement>;
pub type Program = Block;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum UseKind {
    Normal,
    Function,
    Const,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct StaticVar {
    pub var: Variable,
    pub default: Option<Expression>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum Ending {
    Semicolon(Span),
    CloseTag(Span),
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum Statement {
    FullOpeningTag(Span),
    ShortOpeningTag(Span),
    EchoOpeningTag(Span),
    ClosingTag(Span),
    InlineHtml(ByteString),
    GotoLabel(GotoLabel),
    Goto(GotoStatement),
    HaltCompiler {
        content: Option<ByteString>,
    },
    Static {
        vars: Vec<StaticVar>,
    },
    DoWhile(DoWhileStatement),
    While(WhileStatement),
    For(ForStatement),
    Foreach(ForeachStatement),
    Break(BreakStatement),
    Continue(ContinueStatement),
    Constant(Constant),
    Function(Function),
    Class(Class),
    Trait(Trait),
    Interface(Interface),
    If(IfStatement),
    Switch {
        #[serde(flatten)]
        condition: Parenthesized<Expression>,
        cases: Vec<Case>,
    },
    Echo {
        echo: Span,
        values: Vec<Expression>,
        ending: Ending,
    },
    Expression {
        expression: Expression,
        ending: Ending,
    },
    Return {
        r#return: Span,
        value: Option<Expression>,
        ending: Ending,
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
    Block(Braced<Vec<Statement>>),
    Global {
        span: Span,
        variables: Vec<Variable>,
    },
    Declare(Declare),
    Noop(Span),
}

// See https://www.php.net/manual/en/language.types.type-juggling.php#language.types.typecasting for more info.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
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

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Case {
    pub condition: Option<Expression>,
    pub body: Block,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Use {
    pub name: SimpleIdentifier,
    pub alias: Option<SimpleIdentifier>,
    pub kind: Option<UseKind>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum Expression {
    // eval("$a = 1")
    Eval {
        value: Box<Self>,
    },
    // die, die(1)
    Die {
        value: Option<Box<Self>>,
    },
    // exit, exit(1)
    Exit {
        value: Option<Box<Self>>,
    },
    // echo "foo"
    Echo {
        values: Vec<Self>,
    },
    ArithmeticOperation(ArithmeticOperation),
    AssignmentOperation(AssignmentOperation),
    BitwiseOperation(BitwiseOperation),
    ComparisonOperation(ComparisonOperation),
    LogicalOperation(LogicalOperation),
    // $a . $b
    Concat {
        left: Box<Self>,
        span: Span,
        right: Box<Self>,
    },
    // $foo instanceof Bar
    Instanceof {
        left: Box<Self>,
        span: Span,
        right: Box<Self>,
    },
    // &$foo
    Reference {
        span: Span,
        right: Box<Self>,
    },
    // ($a && $b)
    Parenthesized {
        start: Span,
        expr: Box<Self>,
        end: Span,
    },
    // list($a, $b)
    List {
        items: Vec<ListItem>,
    },
    Empty,
    // @foo()
    ErrorSuppress {
        span: Span,
        expr: Box<Self>,
    },
    // 1, 1_000, 0123, 0o123, 0x123, 0b1010
    LiteralInteger {
        span: Span,
        value: ByteString,
    },
    // 1.12345
    LiteralFloat {
        span: Span,
        value: ByteString,
    },
    // foo | foo_bar | _foo | foo123
    Identifier(Identifier),
    Variable(Variable),
    // include "foo.php"
    Include {
        span: Span,
        path: Box<Self>,
    },
    // include_once "foo.php"
    IncludeOnce {
        span: Span,
        path: Box<Self>,
    },
    // require "foo.php"
    Require {
        span: Span,
        path: Box<Self>,
    },
    // require_once "foo.php"
    RequireOnce {
        span: Span,
        path: Box<Self>,
    },
    // `foo(1, 2, 3)`
    FunctionCall {
        target: Box<Self>,       // `foo`
        arguments: ArgumentList, // `(1, 2, 3)`
    },
    // `foo(...)`
    FunctionClosureCreation {
        target: Box<Self>,                // `foo`
        placeholder: ArgumentPlaceholder, // `(...)`
    },
    // `$foo->bar(1, 2, 3)`
    MethodCall {
        target: Box<Self>,       // `$foo`
        span: Span,              // `->`
        method: Box<Self>,       // `bar`
        arguments: ArgumentList, // `(1, 2, 3)`
    },
    // `$foo->bar(...)`
    MethodClosureCreation {
        target: Box<Self>,                // `$foo`
        span: Span,                       // `->`
        method: Box<Self>,                // `bar`
        placeholder: ArgumentPlaceholder, // `(...)`
    },
    // `$foo?->bar(1, 2, 3)`
    NullsafeMethodCall {
        target: Box<Self>,       // `$foo`
        span: Span,              // `?->`
        method: Box<Self>,       // `bar`
        arguments: ArgumentList, // `(1, 2, 3)`
    },
    // `Foo::bar(1, 2, 3)`
    StaticMethodCall {
        target: Box<Self>,       // `Foo`
        double_colon: Span,      // `::`
        method: Box<Self>,       // `bar`
        arguments: ArgumentList, // `(1, 2, 3)`
    },
    // `Foo::bar(...)`
    StaticMethodClosureCreation {
        target: Box<Self>,                // `Foo`
        double_colon: Span,               // `::`
        method: Box<Self>,                // `bar`
        placeholder: ArgumentPlaceholder, // `(...)`
    },
    // `static`
    Static,
    // `self`
    Self_,
    // `parent`
    Parent,
    // `[1, 2, 3]`
    ShortArray(Bracketed<CommaSeparated<ArrayItem>>),
    // `array(1, 2, 3)`
    Array {
        span: Span,            // `array`
        start: Span,           // `(`
        items: Vec<ArrayItem>, // `1, 2, 3`
        end: Span,             // `)`
    },
    // `function() {}`
    Closure(Closure),
    // `fn() => $foo`
    ArrowFunction(ArrowFunction),
    // `new Foo(1, 2, 3)`
    New {
        span: Span,                      // `new`
        target: Box<Self>,               // `Foo`
        arguments: Option<ArgumentList>, // `(1, 2, 3)`
    },
    // `'foo'`
    LiteralString {
        span: Span,
        value: ByteString,
    },
    // `"foo $bar foo"`
    InterpolatedString {
        parts: Vec<StringPart>,
    },
    // `<<<"EOT"` / `<<<EOT`
    Heredoc {
        parts: Vec<StringPart>,
    },
    // `<<<'EOT'`
    Nowdoc {
        value: ByteString,
    },
    // ``foo``
    ShellExec {
        parts: Vec<StringPart>,
    },
    // `foo()->bar`
    PropertyFetch {
        target: Box<Self>,   // `foo()`
        span: Span,          // `->`
        property: Box<Self>, // `bar`
    },
    // `foo()?->bar`
    NullsafePropertyFetch {
        target: Box<Self>,   // `foo()`
        span: Span,          // `?->`
        property: Box<Self>, // `bar`
    },
    // `foo()::bar`
    StaticPropertyFetch {
        target: Box<Self>,   // `foo()`
        span: Span,          // `::`
        property: Box<Self>, // `bar`
    },
    ConstFetch {
        target: Box<Self>,
        constant: SimpleIdentifier,
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
    // `foo() ?: bar()`
    ShortTernary {
        condition: Box<Self>, // `foo()`
        span: Span,           // `?:`
        r#else: Box<Self>,    // `bar()`
    },
    // `foo() ? bar() : baz()`
    Ternary {
        condition: Box<Self>, // `foo()`
        span: Span,           // `?`
        then: Box<Self>,      // `bar()`
        colon: Span,          // `:`
        r#else: Box<Self>,    // `baz()`
    },
    Coalesce {
        lhs: Box<Self>,
        rhs: Box<Self>,
    },
    Clone {
        target: Box<Self>,
    },

    // TODO(azjezz): create a separate structure for `Match`
    Match {
        keyword: Span,
        condition: Parenthesized<Box<Self>>,
        // TODO(azjezz): create a separate structure for `default` and `arms` to hold `{` and `}` spans.
        default: Option<Box<DefaultMatchArm>>,
        arms: Vec<MatchArm>,
    },
    Throw {
        value: Box<Self>,
    },
    Yield {
        key: Option<Box<Self>>,
        value: Option<Box<Self>>,
    },
    YieldFrom {
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
    Noop,
}

// TODO(azjezz): create a separate enum for MatchArm and DefaultMatchArm

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DefaultMatchArm {
    pub keyword: Span,    // `default`
    pub arrow: Span,      // `=>`
    pub body: Expression, // `foo()`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MatchArm {
    pub conditions: Vec<Expression>,
    pub arrow: Span,
    pub body: Expression,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
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

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum StringPart {
    Literal(ByteString),
    Expression(Box<Expression>),
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ArrayItem {
    pub key: Option<Expression>,
    pub value: Expression,
    pub unpack: bool,
    pub by_ref: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ListItem {
    pub key: Option<Expression>,
    pub value: Expression,
}
