use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::byte_string::ByteString;
use crate::lexer::token::Span;
use crate::lexer::token::TokenKind;
use crate::parser::ast::arguments::ArgumentPlaceholder;
use crate::parser::ast::arguments::{ArgumentList, SingleArgument};
use crate::parser::ast::classes::AnonymousClass;
use crate::parser::ast::classes::ClassStatement;
use crate::parser::ast::comments::Comment;
use crate::parser::ast::constant::ConstantStatement;
use crate::parser::ast::control_flow::IfStatement;
use crate::parser::ast::declares::DeclareStatement;
use crate::parser::ast::enums::BackedEnumStatement;
use crate::parser::ast::enums::UnitEnumStatement;
use crate::parser::ast::functions::ArrowFunction;
use crate::parser::ast::functions::Closure;
use crate::parser::ast::functions::FunctionStatement;
use crate::parser::ast::goto::LabelStatement;
use crate::parser::ast::goto::GotoStatement;
use crate::parser::ast::identifiers::Identifier;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::interfaces::InterfaceStatement;
use crate::parser::ast::literals::Literal;
use crate::parser::ast::loops::BreakStatement;
use crate::parser::ast::loops::ContinueStatement;
use crate::parser::ast::loops::DoWhileStatement;
use crate::parser::ast::loops::ForStatement;
use crate::parser::ast::loops::ForeachStatement;
use crate::parser::ast::loops::WhileStatement;
use crate::parser::ast::namespaces::NamespaceStatement;
use crate::parser::ast::operators::ArithmeticOperation;
use crate::parser::ast::operators::AssignmentOperation;
use crate::parser::ast::operators::BitwiseOperation;
use crate::parser::ast::operators::ComparisonOperation;
use crate::parser::ast::operators::LogicalOperation;
use crate::parser::ast::traits::TraitStatement;
use crate::parser::ast::try_block::TryStatement;
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
#[serde(rename_all = "snake_case", tag = "type")]
pub struct HaltCompiler {
    pub content: Option<ByteString>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub struct StaticStatement {
    pub vars: Vec<StaticVar>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub struct SwitchStatement {
    pub switch: Span,
    pub left_parenthesis: Span,
    pub condition: Expression,
    pub right_parenthesis: Span,
    pub cases: Vec<Case>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub struct EchoStatement {
    pub echo: Span,
    pub values: Vec<Expression>,
    pub ending: Ending,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub struct ReturnStatement {
    pub r#return: Span,
    pub value: Option<Expression>,
    pub ending: Ending,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub struct UseStatement {
    pub kind: UseKind,
    pub uses: Vec<Use>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub struct GroupUseStatement {
    pub prefix: SimpleIdentifier,
    pub kind: UseKind,
    pub uses: Vec<Use>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum Statement {
    FullOpeningTag(Span),
    ShortOpeningTag(Span),
    EchoOpeningTag(Span),
    ClosingTag(Span),
    InlineHtml(ByteString),
    Label(LabelStatement),
    Goto(GotoStatement),
    HaltCompiler(HaltCompiler),
    Static(StaticStatement),
    DoWhile(DoWhileStatement),
    While(WhileStatement),
    For(ForStatement),
    Foreach(ForeachStatement),
    Break(BreakStatement),
    Continue(ContinueStatement),
    Constant(ConstantStatement),
    Function(FunctionStatement),
    Class(ClassStatement),
    Trait(TraitStatement),
    Interface(InterfaceStatement),
    If(IfStatement),
    Switch(SwitchStatement),
    Echo(EchoStatement),
    Expression(ExpressionStatement),
    Return(ReturnStatement),
    Namespace(NamespaceStatement),
    Use(UseStatement),
    GroupUse(GroupUseStatement),
    Comment(Comment),
    Try(TryStatement),
    UnitEnum(UnitEnumStatement),
    BackedEnum(BackedEnumStatement),
    Block(BlockStatement),
    Global(GlobalStatement),
    Declare(DeclareStatement),
    Noop(Span),
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub struct ExpressionStatement {
    pub expression: Expression,
    pub ending: Ending,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub struct GlobalStatement {
    pub global: Span,
    pub variables: Vec<Variable>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub struct BlockStatement {
    pub left_brace: Span,
    pub statements: Vec<Statement>,
    pub right_brace: Span,
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
        eval: Span,                    // eval
        argument: Box<SingleArgument>, // ("$a = 1")
    },
    // empty($a)
    Empty {
        empty: Span,                   // empty
        argument: Box<SingleArgument>, // ($a)
    },
    // die, die(1)
    Die {
        die: Span,                             // die
        argument: Option<Box<SingleArgument>>, // (1)
    },
    // exit, exit(1)
    Exit {
        exit: Span,                            // exit
        argument: Option<Box<SingleArgument>>, // (1)
    },
    // isset($a), isset($a, ...)
    Isset {
        isset: Span,             // isset
        arguments: ArgumentList, // `($a, ...)`
    },
    // unset($a), isset($a, ...)
    Unset {
        unset: Span,             // unset
        arguments: ArgumentList, // `($a, ...)`
    },
    // print(1), print 1;
    Print {
        print: Span,                           // print
        value: Option<Box<Self>>,              // 1
        argument: Option<Box<SingleArgument>>, // (1)
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
        left_brace: Span,
        default: Option<Box<DefaultMatchArm>>,
        arms: Vec<MatchArm>,
        right_brace: Span,
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
