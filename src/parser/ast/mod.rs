use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::byte_string::ByteString;
use crate::lexer::token::Span;
use crate::lexer::token::TokenKind;
use crate::node::Node;
use crate::parser::ast::arguments::ArgumentPlaceholder;
use crate::parser::ast::arguments::{ArgumentList, SingleArgument};
use crate::parser::ast::classes::AnonymousClassExpression;
use crate::parser::ast::classes::ClassStatement;
use crate::parser::ast::comments::Comment;
use crate::parser::ast::constant::ConstantStatement;
use crate::parser::ast::control_flow::IfStatement;
use crate::parser::ast::declares::DeclareStatement;
use crate::parser::ast::enums::BackedEnumStatement;
use crate::parser::ast::enums::UnitEnumStatement;
use crate::parser::ast::functions::ArrowFunctionExpression;
use crate::parser::ast::functions::ClosureExpression;
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
use crate::parser::ast::operators::ArithmeticOperationExpression;
use crate::parser::ast::operators::AssignmentOperationExpression;
use crate::parser::ast::operators::BitwiseOperationExpression;
use crate::parser::ast::operators::ComparisonOperationExpression;
use crate::parser::ast::operators::LogicalOperationExpression;
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
    fn children(&mut self) -> Vec<&mut dyn Node> {
        self.iter_mut().map(|s| s as &mut dyn Node).collect()
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
    fn children(&mut self) -> Vec<&mut dyn Node> {
        let mut children: Vec<&mut dyn Node> = vec![&mut self.var];
        if let Some(default) = &mut self.default {
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

impl Node for HaltCompiler {}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub struct StaticStatement {
    pub vars: Vec<StaticVar>,
}

impl Node for StaticStatement {
    fn children(&mut self) -> Vec<&mut dyn Node> {
        self.vars.iter_mut().map(|v| v as &mut dyn Node).collect()
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
    fn children(&mut self) -> Vec<&mut dyn Node> {
        let mut children: Vec<&mut dyn Node> = vec![&mut self.condition];
        children.extend(self.cases.iter_mut().map(|c| c as &mut dyn Node));
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
    fn children(&mut self) -> Vec<&mut dyn Node> {
        self.values.iter_mut().map(|v| v as &mut dyn Node).collect()
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
    fn children(&mut self) -> Vec<&mut dyn Node> {
        if let Some(value) = &mut self.value {
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
    fn children(&mut self) -> Vec<&mut dyn Node> {
        self.uses.iter_mut().map(|u| u as &mut dyn Node).collect()
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
    fn children(&mut self) -> Vec<&mut dyn Node> {
        let mut children: Vec<&mut dyn Node> = vec![&mut self.prefix];
        children.extend(self.uses.iter_mut().map(|u| u as &mut dyn Node));
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
    fn children(&mut self) -> Vec<&mut dyn Node> {
        match self {
            Statement::Label(statement) => vec![statement],
            Statement::Goto(statement) => vec![statement],
            Statement::HaltCompiler(statement) => vec![statement],
            Statement::Static(statement) => vec![statement],
            Statement::DoWhile(statement) => vec![statement],
            Statement::While(statement) => vec![statement],
            Statement::For(statement) => vec![statement],
            Statement::Foreach(statement) => vec![statement],
            Statement::Break(statement) => vec![statement],
            Statement::Continue(statement) => vec![statement],
            Statement::Constant(statement) => vec![statement],
            Statement::Function(statement) => vec![statement],
            Statement::Class(statement) => vec![statement],
            Statement::Trait(statement) => vec![statement],
            Statement::Interface(statement) => vec![statement],
            Statement::If(statement) => vec![statement],
            Statement::Switch(statement) => vec![statement],
            Statement::Echo(statement) => vec![statement],
            Statement::Expression(statement) => vec![statement],
            Statement::Return(statement) => vec![statement],
            Statement::Namespace(statement) => vec![statement],
            Statement::Use(statement) => vec![statement],
            Statement::GroupUse(statement) => vec![statement],
            Statement::Comment(statement) => vec![statement],
            Statement::Try(statement) => vec![statement],
            Statement::UnitEnum(statement) => vec![statement],
            Statement::BackedEnum(statement) => vec![statement],
            Statement::Block(statement) => vec![statement],
            Statement::Global(statement) => vec![statement],
            Statement::Declare(statement) => vec![statement],
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
    fn children(&mut self) -> Vec<&mut dyn Node> {
        vec![&mut self.expression]
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub struct GlobalStatement {
    pub global: Span,
    pub variables: Vec<Variable>,
}

impl Node for GlobalStatement {
    fn children(&mut self) -> Vec<&mut dyn Node> {
        self.variables
            .iter_mut()
            .map(|v| v as &mut dyn Node)
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
    fn children(&mut self) -> Vec<&mut dyn Node> {
        self.statements
            .iter_mut()
            .map(|s| s as &mut dyn Node)
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
    fn children(&mut self) -> Vec<&mut dyn Node> {
        let mut children: Vec<&mut dyn Node> = vec![];
        if let Some(condition) = &mut self.condition {
            children.push(condition);
        }
        children.extend(
            self.body
                .iter_mut()
                .map(|statement| statement as &mut dyn Node)
                .collect::<Vec<&mut dyn Node>>(),
        );
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
    fn children(&mut self) -> Vec<&mut dyn Node> {
        let mut children: Vec<&mut dyn Node> = vec![&mut self.name];
        if let Some(alias) = &mut self.alias {
            children.push(alias);
        }
        children
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct EvalExpression {
    pub eval: Span,
    // eval
    pub argument: Box<SingleArgument>, // ("$a = 1")
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct EmptyExpression {
    pub empty: Span,
    // empty
    pub argument: Box<SingleArgument>, // ($a)
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct DieExpression {
    pub die: Span,
    // die
    pub argument: Option<Box<SingleArgument>>, // (1)
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ExitExpression {
    pub exit: Span,
    // exit
    pub argument: Option<Box<SingleArgument>>, // (1)
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct IssetExpression {
    pub isset: Span,
    // isset
    pub arguments: ArgumentList, // `($a, ...)`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct UnsetExpression {
    pub unset: Span,
    // unset
    pub arguments: ArgumentList, // `($a, ...)`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PrintExpression {
    pub print: Span,
    // print
    pub value: Option<Box<Self>>,
    // 1
    pub argument: Option<Box<SingleArgument>>, // (1)
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ConcatExpression {
    pub left: Box<Self>,
    pub dot: Span,
    pub right: Box<Self>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct InstanceofExpression {
    pub left: Box<Self>,
    pub instanceof: Span,
    pub right: Box<Self>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ReferenceExpression {
    pub ampersand: Span,
    pub right: Box<Self>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ParenthesizedExpression {
    pub start: Span,
    pub expr: Box<Self>,
    pub end: Span,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ErrorSuppressExpression {
    pub at: Span,
    pub expr: Box<Self>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct IncludeExpression {
    pub include: Span,
    pub path: Box<Self>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct IncludeOnceExpression {
    pub include_once: Span,
    pub path: Box<Self>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct RequireExpression {
    pub require: Span,
    pub path: Box<Self>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct RequireOnceExpression {
    pub require_once: Span,
    pub path: Box<Self>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct FunctionCallExpression {
    pub target: Box<Self>,
    // `foo`
    pub arguments: ArgumentList, // `(1, 2, 3)`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct FunctionClosureCreationExpression {
    pub target: Box<Self>,
    // `foo`
    pub placeholder: ArgumentPlaceholder, // `(...)`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct MethodCallExpression {
    pub target: Box<Self>,
    // `$foo`
    pub arrow: Span,
    // `->`
    pub method: Box<Self>,
    // `bar`
    pub arguments: ArgumentList, // `(1, 2, 3)`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct MethodClosureCreationExpression {
    pub target: Box<Self>,
    // `$foo`
    pub arrow: Span,
    // `->`
    pub method: Box<Self>,
    // `bar`
    pub placeholder: ArgumentPlaceholder, // `(...)`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct NullsafeMethodCallExpression {
    pub target: Box<Self>,
    // `$foo`
    pub question_arrow: Span,
    // `?->`
    pub method: Box<Self>,
    // `bar`
    pub arguments: ArgumentList, // `(1, 2, 3)`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct StaticMethodCallExpression {
    pub target: Box<Self>,
    // `Foo`
    pub double_colon: Span,
    // `::`
    pub method: Identifier,
    // `bar`
    pub arguments: ArgumentList, // `(1, 2, 3)`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct StaticVariableMethodCallExpression {
    pub target: Box<Self>,
    // `Foo`
    pub double_colon: Span,
    // `::`
    pub method: Variable,
    // `$bar`
    pub arguments: ArgumentList, // `(1, 2, 3)`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct StaticMethodClosureCreationExpression {
    pub target: Box<Self>,
    // `Foo`
    pub double_colon: Span,
    // `::`
    pub method: Identifier,
    // `bar`
    pub placeholder: ArgumentPlaceholder, // `(...)`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct StaticVariableMethodClosureCreationExpression {
    pub target: Box<Self>,
    // `Foo`
    pub double_colon: Span,
    // `::`
    pub method: Variable,
    // `$bar`
    pub placeholder: ArgumentPlaceholder, // `(...)`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PropertyFetchExpression {
    pub target: Box<Self>,
    // `foo()`
    pub arrow: Span,
    // `->`
    pub property: Box<Self>, // `bar`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct NullsafePropertyFetchExpression {
    pub target: Box<Self>,
    // `foo()`
    pub question_arrow: Span,
    // `?->`
    pub property: Box<Self>,  // `bar`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct StaticPropertyFetchExpression {
    pub target: Box<Self>,
    // `foo()`
    pub double_colon: Span,
    // `::`
    pub property: Variable, // `$bar`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ConstantFetchExpression {
    pub target: Box<Self>,
    // `foo()`
    pub double_colon: Span,
    // `::`
    pub constant: Identifier, // `bar`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ShortArrayExpression {
    pub start: Span,
    // `[`
    pub items: CommaSeparated<ArrayItem>,
    // `1, 2, 3`
    pub end: Span,                        // `]`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ArrayExpression {
    pub array: Span,
    // `array`
    pub start: Span,
    // `(`
    pub items: CommaSeparated<ArrayItem>,
    // `1, 2, 3`
    pub end: Span,                        // `)`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ListExpression {
    pub list: Span,
    // `list`
    pub start: Span,
    // `(`
    pub items: Vec<ListEntry>,
    // `$a, $b`
    pub end: Span,             // `)`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct NewExpression {
    pub new: Span,
    // `new`
    pub target: Box<Self>,
    // `Foo`
    pub arguments: Option<ArgumentList>, // `(1, 2, 3)`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct InterpolatedStringExpression {
    pub parts: Vec<StringPart>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct HeredocExpression {
    pub parts: Vec<StringPart>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct NowdocExpression {
    pub value: ByteString,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ShellExecExpression {
    pub parts: Vec<StringPart>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct BoolExpression {
    pub value: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ArrayIndexExpression {
    pub array: Box<Self>,
    pub left_bracket: Span,
    pub index: Option<Box<Self>>,
    pub right_bracket: Span,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ShortTernaryExpression {
    pub condition: Box<Self>,
    // `foo()`
    pub question_colon: Span,
    // `?:`
    pub r#else: Box<Self>,    // `bar()`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct TernaryExpression {
    pub condition: Box<Self>,
    // `foo()`
    pub question: Span,
    // `?`
    pub then: Box<Self>,
    // `bar()`
    pub colon: Span,
    // `:`
    pub r#else: Box<Self>,    // `baz()`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CoalesceExpression {
    pub lhs: Box<Self>,
    pub double_question: Span,
    pub rhs: Box<Self>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CloneExpression {
    pub target: Box<Self>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct MatchExpression {
    pub keyword: Span,
    pub left_parenthesis: Span,
    pub condition: Box<Self>,
    pub right_parenthesis: Span,
    pub left_brace: Span,
    pub default: Option<Box<DefaultMatchArm>>,
    pub arms: Vec<MatchArm>,
    pub right_brace: Span,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ThrowExpression {
    pub value: Box<Self>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct YieldExpression {
    pub key: Option<Box<Self>>,
    pub value: Option<Box<Self>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct YieldFromExpression {
    pub value: Box<Self>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CastExpression {
    pub cast: Span,
    pub kind: CastKind,
    pub value: Box<Self>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum Expression {
    // eval("$a = 1")
    Eval(EvalExpression),
    // empty($a)
    Empty(EmptyExpression),
    // die, die(1)
    Die(DieExpression),
    // exit, exit(1)
    Exit(ExitExpression),
    // isset($a), isset($a, ...)
    Isset(IssetExpression),
    // unset($a), isset($a, ...)
    Unset(UnsetExpression),
    // print(1), print 1;
    Print(PrintExpression),
    Literal(Literal),
    ArithmeticOperation(ArithmeticOperationExpression),
    AssignmentOperation(AssignmentOperationExpression),
    BitwiseOperation(BitwiseOperationExpression),
    ComparisonOperation(ComparisonOperationExpression),
    LogicalOperation(LogicalOperationExpression),
    // $a . $b
    Concat(ConcatExpression),
    // $foo instanceof Bar
    Instanceof(InstanceofExpression),
    // &$foo
    Reference(ReferenceExpression),
    // ($a && $b)
    Parenthesized(ParenthesizedExpression),
    // @foo()
    ErrorSuppress(ErrorSuppressExpression),
    // `foo`, `foo_bar`, etc
    Identifier(Identifier),
    // `$foo`, `$foo_bar`, etc
    Variable(Variable),
    // include "foo.php"
    Include(IncludeExpression),
    // include_once "foo.php"
    IncludeOnce(IncludeOnceExpression),
    // require "foo.php"
    Require(RequireExpression),
    // require_once "foo.php"
    RequireOnce(RequireOnceExpression),
    // `foo(1, 2, 3)`
    FunctionCall(FunctionCallExpression),
    // `foo(...)`
    FunctionClosureCreation(FunctionClosureCreationExpression),
    // `$foo->bar(1, 2, 3)`
    MethodCall(MethodCallExpression),
    // `$foo->bar(...)`
    MethodClosureCreation(MethodClosureCreationExpression),
    // `$foo?->bar(1, 2, 3)`
    NullsafeMethodCall(NullsafeMethodCallExpression),
    // `Foo::bar(1, 2, 3)`
    StaticMethodCall(StaticMethodCallExpression),
    // `Foo::$bar(1, 2, 3)`
    StaticVariableMethodCall(StaticVariableMethodCallExpression),
    // `Foo::bar(...)`
    StaticMethodClosureCreation(StaticMethodClosureCreationExpression),
    // `Foo::$bar(...)`
    StaticVariableMethodClosureCreation(StaticVariableMethodClosureCreationExpression),
    // `foo()->bar`
    PropertyFetch(PropertyFetchExpression),
    // `foo()?->bar`
    NullsafePropertyFetch(NullsafePropertyFetchExpression),
    // `foo()::$bar`
    StaticPropertyFetch(StaticPropertyFetchExpression),
    // `foo()::bar` or `foo()::{$name}`
    ConstantFetch(ConstantFetchExpression),
    // `static`
    Static,
    // `self`
    Self_,
    // `parent`
    Parent,
    // `[1, 2, 3]`
    ShortArray(ShortArrayExpression),
    // `array(1, 2, 3)`
    Array(ArrayExpression),
    // list($a, $b)
    List(ListExpression),
    // `function() {}`
    Closure(ClosureExpression),
    // `fn() => $foo`
    ArrowFunction(ArrowFunctionExpression),
    // `new Foo(1, 2, 3)`
    New(NewExpression),
    // `"foo $bar foo"`
    InterpolatedString(InterpolatedStringExpression),
    // `<<<"EOT"` / `<<<EOT`
    Heredoc(HeredocExpression),
    // `<<<'EOT'`
    Nowdoc(NowdocExpression),
    // ``foo``
    ShellExec(ShellExecExpression),
    // `new class { ... }`
    AnonymousClass(AnonymousClassExpression),
    // `true`, `false`
    Bool(BoolExpression),
    // `$foo[0]`
    ArrayIndex(ArrayIndexExpression),
    // `null`
    Null,
    // `__DIR__`, etc
    MagicConstant(MagicConstantExpression),
    // `foo() ?: bar()`
    ShortTernary(ShortTernaryExpression),
    // `foo() ? bar() : baz()`
    Ternary(TernaryExpression),
    // `foo() ?? bar()`
    Coalesce(CoalesceExpression),
    // `clone $foo`
    Clone(CloneExpression),
    // `match ($foo) { ... }`
    Match(MatchExpression),
    // `throw new Exception`
    Throw(ThrowExpression),
    // `yield $foo`
    Yield(YieldExpression),
    // `yield from foo()`
    YieldFrom(YieldFromExpression),
    // `(int) "1"`, etc
    Cast(CastExpression),
    // ;
    Noop,
}

impl Node for Expression {
    fn children(&mut self) -> Vec<&mut dyn Node> {
        match self {
            Expression::Eval(EvalExpression { eval: _, argument }) => vec![argument.as_mut()],
            Expression::Empty(EmptyExpression { empty: _, argument }) => vec![argument.as_mut()],
            Expression::Die(DieExpression { die: _, argument }) => {
                if let Some(argument) = argument {
                    vec![argument.as_mut()]
                } else {
                    vec![]
                }
            }
            Expression::Exit(ExitExpression { exit: _, argument }) => {
                if let Some(argument) = argument {
                    vec![argument.as_mut()]
                } else {
                    vec![]
                }
            }
            Expression::Isset(IssetExpression { isset: _, arguments }) => vec![arguments],
            Expression::Unset(UnsetExpression { unset: _, arguments }) => vec![arguments],
            Expression::Print(PrintExpression { print: _, value, argument }) => {
                if let Some(argument) = argument {
                    vec![argument.as_mut()]
                } else if let Some(value) = value {
                    vec![value.as_mut()]
                } else {
                    vec![]
                }
            }
            Expression::Literal(literal) => vec![literal],
            Expression::ArithmeticOperation(operation) => vec![operation],
            Expression::AssignmentOperation(operation) => vec![operation],
            Expression::BitwiseOperation(operation) => vec![operation],
            Expression::ComparisonOperation(operation) => vec![operation],
            Expression::LogicalOperation(operation) => vec![operation],
            Expression::Concat(ConcatExpression { left, dot: _, right }) => vec![left.as_mut(), right.as_mut()],
            Expression::Instanceof(InstanceofExpression { left, instanceof: _, right }) => vec![left.as_mut(), right.as_mut()],
            Expression::Reference(ReferenceExpression { ampersand: _, right }) => vec![right.as_mut()],
            Expression::Parenthesized(ParenthesizedExpression { start: _, expr, end: _ }) => vec![expr.as_mut()],
            Expression::ErrorSuppress(ErrorSuppressExpression { at: _, expr }) => vec![expr.as_mut()],
            Expression::Identifier(identifier) => vec![identifier],
            Expression::Variable(variable) => vec![variable],
            Expression::Include(IncludeExpression { include: _, path }) => vec![path.as_mut()],
            Expression::IncludeOnce(IncludeOnceExpression { include_once: _, path }) => vec![path.as_mut()],
            Expression::Require(RequireExpression { require: _, path }) => vec![path.as_mut()],
            Expression::RequireOnce(RequireOnceExpression { require_once: _, path }) => vec![path.as_mut()],
            Expression::FunctionCall(FunctionCallExpression { target, arguments }) => vec![target.as_mut(), arguments],
            Expression::FunctionClosureCreation(FunctionClosureCreationExpression { target, placeholder: _ }) => vec![target.as_mut()],
            Expression::MethodCall(MethodCallExpression { target, arrow: _, method, arguments }) => vec![target.as_mut(), method.as_mut(), arguments],
            Expression::MethodClosureCreation(MethodClosureCreationExpression { target, arrow: _, method, placeholder: _ }) => vec![target.as_mut(), method.as_mut()],
            Expression::NullsafeMethodCall(NullsafeMethodCallExpression { target, question_arrow: _, method, arguments }) => vec![target.as_mut(), method.as_mut(), arguments],
            Expression::StaticMethodCall(StaticMethodCallExpression { target, double_colon: _, method, arguments }) => vec![target.as_mut(), method, arguments],
            Expression::StaticVariableMethodCall(StaticVariableMethodCallExpression { target, double_colon: _, method, arguments }) => vec![target.as_mut(), method, arguments],
            Expression::StaticMethodClosureCreation(StaticMethodClosureCreationExpression { target, double_colon: _, method, placeholder: _ }) => vec![target.as_mut(), method],
            Expression::StaticVariableMethodClosureCreation(StaticVariableMethodClosureCreationExpression { target, double_colon: _, method, placeholder: _ }) => vec![target.as_mut(), method],
            Expression::PropertyFetch(PropertyFetchExpression { target, arrow: _, property }) => vec![target.as_mut(), property.as_mut()],
            Expression::NullsafePropertyFetch(NullsafePropertyFetchExpression { target, question_arrow: _, property }) => vec![target.as_mut(), property.as_mut()],
            Expression::StaticPropertyFetch(StaticPropertyFetchExpression { target, double_colon: _, property }) => vec![target.as_mut(), property],
            Expression::ConstantFetch(ConstantFetchExpression { target, double_colon: _, constant }) => vec![target.as_mut(), constant],
            Expression::Static => vec![],
            Expression::Self_ => vec![],
            Expression::Parent => vec![],
            Expression::ShortArray(ShortArrayExpression { start: _, items, end: _ }) => vec![items],
            Expression::Array(ArrayExpression { array: _, start: _, items, end: _ }) => vec![items],
            Expression::List(ListExpression { list: _, start: _, items, end: _ }) => items.iter_mut().map(|item| item as &mut dyn Node).collect(),
            Expression::Closure(closure) => closure.children(),
            Expression::ArrowFunction(function) => function.children(),
            Expression::New(NewExpression { new: _, target, arguments }) => {
                let mut children: Vec<&mut dyn Node> = vec![target.as_mut()];
                if let Some(arguments) = arguments {
                    children.push(arguments);
                }
                children
            }
            Expression::InterpolatedString(InterpolatedStringExpression { parts }) => {
                parts.iter_mut().map(|part| part as &mut dyn Node).collect()
            }
            Expression::Heredoc(HeredocExpression { parts }) => {
                parts.iter_mut().map(|part| part as &mut dyn Node).collect()
            }
            Expression::Nowdoc(NowdocExpression { value: _ }) => vec![],
            Expression::ShellExec(ShellExecExpression { parts }) => {
                parts.iter_mut().map(|part| part as &mut dyn Node).collect()
            }
            Expression::AnonymousClass(class) => class.children(),
            Expression::Bool(BoolExpression { value: _ }) => vec![],
            Expression::ArrayIndex(ArrayIndexExpression { array: _, left_bracket: _, index, right_bracket: _ }) => {
                let mut children: Vec<&mut dyn Node> = vec![];
                if let Some(index) = index {
                    children.push(index.as_mut());
                }
                children
            }
            Expression::Null => vec![],
            Expression::MagicConstant(constant) => constant.children(),
            Expression::ShortTernary(ShortTernaryExpression { condition, question_colon: _, r#else }) => vec![condition.as_mut(), r#else.as_mut()],
            Expression::Ternary(TernaryExpression { condition, question: _, then, colon: _, r#else }) => vec![condition.as_mut(), then.as_mut(), r#else.as_mut()],
            Expression::Coalesce(CoalesceExpression { lhs, double_question: _, rhs }) => vec![lhs.as_mut(), rhs.as_mut()],
            Expression::Clone(CloneExpression { target }) => vec![target.as_mut()],
            Expression::Match(MatchExpression { keyword: _, left_parenthesis: _, condition, right_parenthesis: _, left_brace: _, default, arms, right_brace: _ }) => {
                let mut children: Vec<&mut dyn Node> = vec![condition.as_mut()];
                if let Some(default) = default {
                    children.push(default.as_mut());
                }
                children.extend(
                    arms.iter_mut()
                        .map(|arm| arm as &mut dyn Node)
                        .collect::<Vec<&mut dyn Node>>(),
                );
                children
            }
            Expression::Throw(ThrowExpression { value }) => vec![value.as_mut()],
            Expression::Yield(YieldExpression { key, value }) => {
                let mut children: Vec<&mut dyn Node> = vec![];
                if let Some(key) = key {
                    children.push(key.as_mut());
                }
                if let Some(value) = value {
                    children.push(value.as_mut());
                }
                children
            }
            Expression::YieldFrom(YieldFromExpression { value }) => vec![value.as_mut()],
            Expression::Cast(CastExpression { cast: _, kind: _, value }) => vec![value.as_mut()],
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
    fn children(&mut self) -> Vec<&mut dyn Node> {
        vec![&mut self.body]
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
    fn children(&mut self) -> Vec<&mut dyn Node> {
        let mut children: Vec<&mut dyn Node> = self
            .conditions
            .iter_mut()
            .map(|condition| condition as &mut dyn Node)
            .collect();
        children.push(&mut self.body);
        children
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum MagicConstantExpression {
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

impl Node for MagicConstantExpression {
    //
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum StringPart {
    Literal(ByteString),
    Expression(Box<Expression>),
}

impl Node for StringPart {
    fn children(&mut self) -> Vec<&mut dyn Node> {
        match self {
            StringPart::Literal(_) => vec![],
            StringPart::Expression(expression) => vec![expression.as_mut()],
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
    fn children(&mut self) -> Vec<&mut dyn Node> {
        match self {
            ArrayItem::Skipped => vec![],
            ArrayItem::Value { value } => vec![value],
            ArrayItem::ReferencedValue {
                ampersand: _,
                value,
            } => vec![value],
            ArrayItem::SpreadValue { ellipsis: _, value } => vec![value],
            ArrayItem::KeyValue {
                key,
                double_arrow: _,
                value,
            } => vec![key, value],
            ArrayItem::ReferencedKeyValue {
                key,
                double_arrow: _,
                ampersand: _,
                value,
            } => vec![key, value],
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
    fn children(&mut self) -> Vec<&mut dyn Node> {
        match self {
            ListEntry::Skipped => vec![],
            ListEntry::Value { value } => vec![value],
            ListEntry::KeyValue {
                key,
                double_arrow: _,
                value,
            } => vec![key, value],
        }
    }
}
