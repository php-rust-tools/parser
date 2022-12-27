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
use crate::parser::ast::literals::Literal;
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
use crate::parser::ast::utils::CommaSeparated;
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
pub mod literals;
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
        switch: Span,
        left_parenthesis: Span,
        condition: Expression,
        right_parenthesis: Span,
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
    Block {
        left_brace: Span,
        statements: Vec<Statement>,
        right_brace: Span,
    },
    Global {
        global: Span,
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
    // empty($a)
    Empty {
        empty: Span,      // empty
        start: Span,      // `(`
        value: Box<Self>, // $a
        end: Span,        // `)`
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
    Literal(Literal),
    ArithmeticOperation(ArithmeticOperation),
    AssignmentOperation(AssignmentOperation),
    BitwiseOperation(BitwiseOperation),
    ComparisonOperation(ComparisonOperation),
    LogicalOperation(LogicalOperation),
    // $a . $b
    Concat {
        left: Box<Self>,
        dot: Span,
        right: Box<Self>,
    },
    // $foo instanceof Bar
    Instanceof {
        left: Box<Self>,
        instanceof: Span,
        right: Box<Self>,
    },
    // &$foo
    Reference {
        ampersand: Span,
        right: Box<Self>,
    },
    // ($a && $b)
    Parenthesized {
        start: Span,
        expr: Box<Self>,
        end: Span,
    },
    // @foo()
    ErrorSuppress {
        at: Span,
        expr: Box<Self>,
    },
    Identifier(Identifier),
    Variable(Variable),
    // include "foo.php"
    Include {
        include: Span,
        path: Box<Self>,
    },
    // include_once "foo.php"
    IncludeOnce {
        include_once: Span,
        path: Box<Self>,
    },
    // require "foo.php"
    Require {
        require: Span,
        path: Box<Self>,
    },
    // require_once "foo.php"
    RequireOnce {
        require_once: Span,
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
        arrow: Span,             // `->`
        method: Box<Self>,       // `bar`
        arguments: ArgumentList, // `(1, 2, 3)`
    },
    // `$foo->bar(...)`
    MethodClosureCreation {
        target: Box<Self>,                // `$foo`
        arrow: Span,                      // `->`
        method: Box<Self>,                // `bar`
        placeholder: ArgumentPlaceholder, // `(...)`
    },
    // `$foo?->bar(1, 2, 3)`
    NullsafeMethodCall {
        target: Box<Self>,       // `$foo`
        question_arrow: Span,    // `?->`
        method: Box<Self>,       // `bar`
        arguments: ArgumentList, // `(1, 2, 3)`
    },
    // `Foo::bar(1, 2, 3)`
    StaticMethodCall {
        target: Box<Self>,       // `Foo`
        double_colon: Span,      // `::`
        method: Identifier,      // `bar`
        arguments: ArgumentList, // `(1, 2, 3)`
    },
    // `Foo::$bar(1, 2, 3)`
    StaticVariableMethodCall {
        target: Box<Self>,       // `Foo`
        double_colon: Span,      // `::`
        method: Variable,        // `$bar`
        arguments: ArgumentList, // `(1, 2, 3)`
    },
    // `Foo::bar(...)`
    StaticMethodClosureCreation {
        target: Box<Self>,                // `Foo`
        double_colon: Span,               // `::`
        method: Identifier,               // `bar`
        placeholder: ArgumentPlaceholder, // `(...)`
    },
    // `Foo::$bar(...)`
    StaticVariableMethodClosureCreation {
        target: Box<Self>,                // `Foo`
        double_colon: Span,               // `::`
        method: Variable,                 // `$bar`
        placeholder: ArgumentPlaceholder, // `(...)`
    },
    // `foo()->bar`
    PropertyFetch {
        target: Box<Self>,   // `foo()`
        arrow: Span,         // `->`
        property: Box<Self>, // `bar`
    },
    // `foo()?->bar`
    NullsafePropertyFetch {
        target: Box<Self>,    // `foo()`
        question_arrow: Span, // `?->`
        property: Box<Self>,  // `bar`
    },
    // `foo()::$bar`
    StaticPropertyFetch {
        target: Box<Self>,  // `foo()`
        double_colon: Span, // `::`
        property: Variable, // `$bar`
    },
    // `foo()::bar` or `foo()::{$name}`
    ConstantFetch {
        target: Box<Self>,    // `foo()`
        double_colon: Span,   // `::`
        constant: Identifier, // `bar`
    },
    // `static`
    Static,
    // `self`
    Self_,
    // `parent`
    Parent,
    // `[1, 2, 3]`
    ShortArray {
        start: Span,                      // `[`
        items: CommaSeparated<ArrayItem>, // `1, 2, 3`
        end: Span,                        // `]`
    },
    // `array(1, 2, 3)`
    Array {
        array: Span,                      // `array`
        start: Span,                      // `(`
        items: CommaSeparated<ArrayItem>, // `1, 2, 3`
        end: Span,                        // `)`
    },
    // list($a, $b)
    List {
        list: Span,            // `list`
        start: Span,           // `(`
        items: Vec<ListEntry>, // `$a, $b`
        end: Span,             // `)`
    },
    // `function() {}`
    Closure(Closure),
    // `fn() => $foo`
    ArrowFunction(ArrowFunction),
    // `new Foo(1, 2, 3)`
    New {
        new: Span,                       // `new`
        target: Box<Self>,               // `Foo`
        arguments: Option<ArgumentList>, // `(1, 2, 3)`
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
    AnonymousClass(AnonymousClass),
    Bool {
        value: bool,
    },
    ArrayIndex {
        array: Box<Self>,
        left_bracket: Span,
        index: Option<Box<Self>>,
        right_bracket: Span,
    },
    Null,
    MagicConstant(MagicConstant),
    // `foo() ?: bar()`
    ShortTernary {
        condition: Box<Self>, // `foo()`
        question_colon: Span, // `?:`
        r#else: Box<Self>,    // `bar()`
    },
    // `foo() ? bar() : baz()`
    Ternary {
        condition: Box<Self>, // `foo()`
        question: Span,       // `?`
        then: Box<Self>,      // `bar()`
        colon: Span,          // `:`
        r#else: Box<Self>,    // `baz()`
    },
    // `foo() ?? bar()`
    Coalesce {
        lhs: Box<Self>,
        double_question: Span,
        rhs: Box<Self>,
    },
    Clone {
        target: Box<Self>,
    },

    // TODO(azjezz): create a separate structure for `Match`
    Match {
        keyword: Span,
        left_parenthesis: Span,
        condition: Box<Self>,
        right_parenthesis: Span,
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
        print: Span,
        value: Box<Self>,
    },
    Cast {
        cast: Span,
        kind: CastKind,
        value: Box<Self>,
    },
    Noop,
}

// TODO(azjezz): create a separate enum for MatchArm and DefaultMatchArm

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DefaultMatchArm {
    pub keyword: Span,      // `default`
    pub double_arrow: Span, // `=>`
    pub body: Expression,   // `foo()`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MatchArm {
    pub conditions: Vec<Expression>,
    pub arrow: Span,
    pub body: Expression,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum MagicConstant {
    Directory(Span),
    File(Span),
    Line(Span),
    Class(Span),
    Function(Span),
    Method(Span),
    Namespace(Span),
    Trait(Span),
    CompilerHaltOffset(Span),
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum StringPart {
    Literal(ByteString),
    Expression(Box<Expression>),
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum ArrayItem {
    Skipped,
    Value {
        value: Expression, // `$foo`
    },
    ReferencedValue {
        ampersand: Span,   // `&`
        value: Expression, // `$foo`
    },
    SpreadValue {
        ellipsis: Span,    // `...`
        value: Expression, // `$foo`
    },
    KeyValue {
        key: Expression,    // `$foo`
        double_arrow: Span, // `=>`
        value: Expression,  // `$bar`
    },
    ReferencedKeyValue {
        key: Expression,    // `$foo`
        double_arrow: Span, // `=>`
        ampersand: Span,    // `&`
        value: Expression,  // `$bar`
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum ListEntry {
    Skipped,
    Value {
        value: Expression, // `$foo`
    },
    KeyValue {
        key: Expression,    // `$foo`
        double_arrow: Span, // `=>`
        value: Expression,  // `$bar`
    },
}
