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
    fn children(&self) -> Vec<&dyn Node> {
        match self {
            Expression::Eval { eval, argument } => vec![argument.as_ref()],
            Expression::Empty { empty, argument } => vec![argument.as_ref()],
            Expression::Die { die, argument } => {
                if let Some(argument) = argument {
                    vec![argument.as_ref()]
                } else {
                    vec![]
                }
            },
            Expression::Exit { exit, argument } => {
                if let Some(argument) = argument {
                    vec![argument.as_ref()]
                } else {
                    vec![]
                }
            },
            Expression::Isset { isset, arguments } => vec![arguments],
            Expression::Unset { unset, arguments } => vec![arguments],
            Expression::Print { print, value, argument } => {
                if let Some(argument) = argument {
                    vec![argument.as_ref()]
                } else if let Some(value) = value {
                    vec![value.as_ref()]
                } else {
                    vec![]
                }
            },
            Expression::Literal(literal) => vec![literal],
            Expression::ArithmeticOperation(operation) => vec![operation],
            Expression::AssignmentOperation(operation) => vec![operation],
            Expression::BitwiseOperation(operation) => vec![operation],
            Expression::ComparisonOperation(operation) => vec![operation],
            Expression::LogicalOperation(operation) => vec![operation],
            Expression::Concat { left, dot, right } => vec![left.as_ref(), right.as_ref()],
            Expression::Instanceof { left, instanceof, right } => vec![left.as_ref(), right.as_ref()],
            Expression::Reference { ampersand, right } => vec![right.as_ref()],
            Expression::Parenthesized { start, expr, end } => vec![expr.as_ref()],
            Expression::ErrorSuppress { at, expr } => vec![expr.as_ref()],
            Expression::Identifier(identifier) => vec![identifier],
            Expression::Variable(variable) => vec![variable],
            Expression::Include { include, path } => vec![path.as_ref()],
            Expression::IncludeOnce { include_once, path } => vec![path.as_ref()],
            Expression::Require { require, path } => vec![path.as_ref()],
            Expression::RequireOnce { require_once, path } => vec![path.as_ref()],
            Expression::FunctionCall { target, arguments } => vec![target.as_ref(), arguments],
            Expression::FunctionClosureCreation { target, placeholder } => vec![target.as_ref()],
            Expression::MethodCall { target, arrow, method, arguments } => vec![target.as_ref(), method.as_ref(), arguments],
            Expression::MethodClosureCreation { target, arrow, method, placeholder } => vec![target.as_ref(), method.as_ref()],
            Expression::NullsafeMethodCall { target, question_arrow, method, arguments } => vec![target.as_ref(), method.as_ref(), arguments],
            Expression::StaticMethodCall { target, double_colon, method, arguments } => vec![target.as_ref(), method, arguments],
            Expression::StaticVariableMethodCall { target, double_colon, method, arguments } => vec![target.as_ref(), method, arguments],
            Expression::StaticMethodClosureCreation { target, double_colon, method, placeholder } => vec![target.as_ref(), method],
            Expression::StaticVariableMethodClosureCreation { target, double_colon, method, placeholder } => vec![target.as_ref(), method],
            Expression::PropertyFetch { target, arrow, property } => vec![target.as_ref(), property.as_ref()],
            Expression::NullsafePropertyFetch { target, question_arrow, property } => vec![target.as_ref(), property.as_ref()],
            Expression::StaticPropertyFetch { target, double_colon, property } => vec![target.as_ref(), property],
            Expression::ConstantFetch { target, double_colon, constant } => vec![target.as_ref(), constant],
            Expression::Static => vec![],
            Expression::Self_ => vec![],
            Expression::Parent => vec![],
            Expression::ShortArray { start, items, end } => vec![items],
            Expression::Array { array, start, items, end } => vec![items],
            Expression::List { list, start, items, end } => items.iter().map(|item| item as &dyn Node).collect(),
            Expression::Closure(closure) => closure.children(),
            Expression::ArrowFunction(function) => function.children(),
            Expression::New { new, target, arguments } => {
                let mut children: Vec<&dyn Node> = vec![target.as_ref()];
                if let Some(arguments) = arguments {
                    children.push(arguments);
                }
                children
            },
            Expression::InterpolatedString { parts } => parts.iter().map(|part| part as &dyn Node).collect(),
            Expression::Heredoc { parts } => parts.iter().map(|part| part as &dyn Node).collect(),
            Expression::Nowdoc { value } => vec![],
            Expression::ShellExec { parts } => parts.iter().map(|part| part as &dyn Node).collect(),
            Expression::AnonymousClass(class) => class.children(),
            Expression::Bool { value } => vec![],
            Expression::ArrayIndex { array, left_bracket, index, right_bracket } => {
                let mut children: Vec<&dyn Node> = vec![];
                if let Some(index) = index {
                    children.push(index.as_ref());
                }
                children
            },
            Expression::Null => vec![],
            Expression::MagicConstant(constant) => constant.children(),
            Expression::ShortTernary { condition, question_colon, r#else } => vec![condition.as_ref(), r#else.as_ref()],
            Expression::Ternary { condition, question, then, colon, r#else } => vec![condition.as_ref(), then.as_ref(), r#else.as_ref()],
            Expression::Coalesce { lhs, double_question, rhs } => vec![lhs.as_ref(), rhs.as_ref()],
            Expression::Clone { target } => vec![target.as_ref()],
            Expression::Match { keyword, left_parenthesis, condition, right_parenthesis, left_brace, default, arms, right_brace } => {
                let mut children: Vec<&dyn Node> = vec![condition.as_ref()];
                if let Some(default) = default {
                    children.push(default.as_ref());
                }
                children.extend(arms.iter().map(|arm| arm as &dyn Node).collect::<Vec<&dyn Node>>());
                children
            },
            Expression::Throw { value } => vec![value.as_ref()],
            Expression::Yield { key, value } => {
                let mut children: Vec<&dyn Node> = vec![];
                if let Some(key) = key {
                    children.push(key.as_ref());
                }
                if let Some(value) = value {
                    children.push(value.as_ref());
                }
                children
            },
            Expression::YieldFrom { value } => vec![value.as_ref()],
            Expression::Cast { cast, kind, value } => vec![value.as_ref()],
            Expression::Noop => vec![],
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DefaultMatchArm {
    pub keyword: Span,      // `default`
    pub double_arrow: Span, // `=>`
    pub body: Expression,   // `foo()`
}

impl Node for DefaultMatchArm {
    fn children(&self) -> Vec<&dyn Node> {
        vec![&self.body]
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MatchArm {
    pub conditions: Vec<Expression>,
    pub arrow: Span,
    pub body: Expression,
}

impl Node for MatchArm {
    fn children(&self) -> Vec<&dyn Node> {
        let mut children: Vec<&dyn Node> = self.conditions.iter().map(|condition| condition as &dyn Node).collect();
        children.push(&self.body);
        children
    }
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

impl Node for MagicConstant {
    //
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum StringPart {
    Literal(ByteString),
    Expression(Box<Expression>),
}

impl Node for StringPart {
    fn children(&self) -> Vec<&dyn Node> {
        match self {
            StringPart::Literal(_) => vec![],
            StringPart::Expression(expression) => vec![expression.as_ref()],
        }
    }
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

impl Node for ArrayItem {
    fn children(&self) -> Vec<&dyn Node> {
        match self {
            ArrayItem::Skipped => vec![],
            ArrayItem::Value { value } => vec![value],
            ArrayItem::ReferencedValue { ampersand, value } => vec![value],
            ArrayItem::SpreadValue { ellipsis, value } => vec![value],
            ArrayItem::KeyValue { key, double_arrow, value } => vec![key, value],
            ArrayItem::ReferencedKeyValue { key, double_arrow, ampersand, value } => vec![key, value],
        }
    }
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

impl Node for ListEntry {
    fn children(&self) -> Vec<&dyn Node> {
        match self {
            ListEntry::Skipped => vec![],
            ListEntry::Value { value } => vec![value],
            ListEntry::KeyValue { key, double_arrow, value } => vec![key, value],
        }
    }
}
