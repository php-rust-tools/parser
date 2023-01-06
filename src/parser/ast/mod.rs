use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::byte_string::ByteString;
use crate::lexer::token::Span;
use crate::lexer::token::TokenKind;
use crate::node::Node;
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
use crate::parser::ast::goto::GotoStatement;
use crate::parser::ast::goto::LabelStatement;
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

impl Node for Block {
    fn children(&self) -> Vec<&dyn Node> {
        self.iter().map(|s| s as &dyn Node).collect()
    }
}

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

impl Node for StaticVar {
    fn children(&self) -> Vec<&dyn Node> {
        let mut children: Vec<&dyn Node> = vec![&self.var];
        if let Some(default) = &self.default {
            children.push(default);
        }
        children
    }
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

impl Node for HaltCompiler {

}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub struct StaticStatement {
    pub vars: Vec<StaticVar>,
}

impl Node for StaticStatement {
    fn children(&self) -> Vec<&dyn Node> {
        self.vars
            .iter()
            .map(|v| v as &dyn Node)
            .collect()
    }
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

impl Node for SwitchStatement {
    fn children(&self) -> Vec<&dyn Node> {
        let mut children: Vec<&dyn Node> = vec![&self.condition];
        children.extend(self.cases.iter().map(|c| c as &dyn Node));
        children
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub struct EchoStatement {
    pub echo: Span,
    pub values: Vec<Expression>,
    pub ending: Ending,
}

impl Node for EchoStatement {
    fn children(&self) -> Vec<&dyn Node> {
        self.values
            .iter()
            .map(|v| v as &dyn Node)
            .collect()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub struct ReturnStatement {
    pub r#return: Span,
    pub value: Option<Expression>,
    pub ending: Ending,
}

impl Node for ReturnStatement {
    fn children(&self) -> Vec<&dyn Node> {
        if let Some(value) = &self.value {
            vec![value]
        } else {
            vec![]
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub struct UseStatement {
    pub kind: UseKind,
    pub uses: Vec<Use>,
}

impl Node for UseStatement {
    fn children(&self) -> Vec<&dyn Node> {
        self.uses
            .iter()
            .map(|u| u as &dyn Node)
            .collect()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub struct GroupUseStatement {
    pub prefix: SimpleIdentifier,
    pub kind: UseKind,
    pub uses: Vec<Use>,
}

impl Node for GroupUseStatement {
    fn children(&self) -> Vec<&dyn Node> {
        let mut children: Vec<&dyn Node> = vec![&self.prefix];
        children.extend(self.uses.iter().map(|u| u as &dyn Node));
        children
    }
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

impl Node for Statement {
    fn children(&self) -> Vec<&dyn Node> {
        match self {
            Statement::Label(statement) => statement.children(),
            Statement::Goto(statement) => statement.children(),
            Statement::HaltCompiler(statement) => statement.children(),
            Statement::Static(statement) => statement.children(),
            Statement::DoWhile(statement) => statement.children(),
            Statement::While(statement) => statement.children(),
            Statement::For(statement) => statement.children(),
            Statement::Foreach(statement) => statement.children(),
            Statement::Break(statement) => statement.children(),
            Statement::Continue(statement) => statement.children(),
            Statement::Constant(statement) => statement.children(),
            Statement::Function(statement) => statement.children(),
            Statement::Class(statement) => statement.children(),
            Statement::Trait(statement) => statement.children(),
            Statement::Interface(statement) => statement.children(),
            Statement::If(statement) => statement.children(),
            Statement::Switch(statement) => statement.children(),
            Statement::Echo(statement) => statement.children(),
            Statement::Expression(statement) => statement.children(),
            Statement::Return(statement) => statement.children(),
            Statement::Namespace(statement) => statement.children(),
            Statement::Use(statement) => statement.children(),
            Statement::GroupUse(statement) => statement.children(),
            Statement::Comment(statement) => statement.children(),
            Statement::Try(statement) => statement.children(),
            Statement::UnitEnum(statement) => statement.children(),
            Statement::BackedEnum(statement) => statement.children(),
            Statement::Block(statement) => statement.children(),
            Statement::Global(statement) => statement.children(),
            Statement::Declare(statement) => statement.children(),
            _ => vec![],
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub struct ExpressionStatement {
    pub expression: Expression,
    pub ending: Ending,
}

impl Node for ExpressionStatement {
    fn children(&self) -> Vec<&dyn Node> {
        vec![&self.expression]
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub struct GlobalStatement {
    pub global: Span,
    pub variables: Vec<Variable>,
}

impl Node for GlobalStatement {
    fn children(&self) -> Vec<&dyn Node> {
        self.variables
            .iter()
            .map(|v| v as &dyn Node)
            .collect()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub struct BlockStatement {
    pub left_brace: Span,
    pub statements: Vec<Statement>,
    pub right_brace: Span,
}

impl Node for BlockStatement {
    fn children(&self) -> Vec<&dyn Node> {
        self.statements
            .iter()
            .map(|s| s as &dyn Node)
            .collect()
    }
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

impl Node for Case {
    fn children(&self) -> Vec<&dyn Node> {
        let mut children: Vec<&dyn Node> = vec![];
        if let Some(condition) = &self.condition {
            children.push(condition);
        }
        children.extend(self.body.iter().map(|statement| statement as &dyn Node).collect::<Vec<&dyn Node>>());
        children
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Use {
    pub name: SimpleIdentifier,
    pub alias: Option<SimpleIdentifier>,
    pub kind: Option<UseKind>,
}

impl Node for Use {
    fn children(&self) -> Vec<&dyn Node> {
        let mut children: Vec<&dyn Node> = vec![&self.name];
        if let Some(alias) = &self.alias {
            children.push(alias);
        }
        children
    }
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

impl Node for Expression {
    
}

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
