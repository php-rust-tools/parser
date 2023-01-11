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
    fn children(&mut self) -> Vec<&mut dyn Node> {
        match self {
            Expression::Eval { eval: _, argument } => vec![argument.as_mut()],
            Expression::Empty { empty: _, argument } => vec![argument.as_mut()],
            Expression::Die { die: _, argument } => {
                if let Some(argument) = argument {
                    vec![argument.as_mut()]
                } else {
                    vec![]
                }
            }
            Expression::Exit { exit: _, argument } => {
                if let Some(argument) = argument {
                    vec![argument.as_mut()]
                } else {
                    vec![]
                }
            }
            Expression::Isset {
                isset: _,
                arguments,
            } => vec![arguments],
            Expression::Unset {
                unset: _,
                arguments,
            } => vec![arguments],
            Expression::Print {
                print: _,
                value,
                argument,
            } => {
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
            Expression::Concat {
                left,
                dot: _,
                right,
            } => vec![left.as_mut(), right.as_mut()],
            Expression::Instanceof {
                left,
                instanceof: _,
                right,
            } => vec![left.as_mut(), right.as_mut()],
            Expression::Reference {
                ampersand: _,
                right,
            } => vec![right.as_mut()],
            Expression::Parenthesized {
                start: _,
                expr,
                end: _,
            } => vec![expr.as_mut()],
            Expression::ErrorSuppress { at: _, expr } => vec![expr.as_mut()],
            Expression::Identifier(identifier) => vec![identifier],
            Expression::Variable(variable) => vec![variable],
            Expression::Include { include: _, path } => vec![path.as_mut()],
            Expression::IncludeOnce {
                include_once: _,
                path,
            } => vec![path.as_mut()],
            Expression::Require { require: _, path } => vec![path.as_mut()],
            Expression::RequireOnce {
                require_once: _,
                path,
            } => vec![path.as_mut()],
            Expression::FunctionCall { target, arguments } => vec![target.as_mut(), arguments],
            Expression::FunctionClosureCreation {
                target,
                placeholder: _,
            } => vec![target.as_mut()],
            Expression::MethodCall {
                target,
                arrow: _,
                method,
                arguments,
            } => vec![target.as_mut(), method.as_mut(), arguments],
            Expression::MethodClosureCreation {
                target,
                arrow: _,
                method,
                placeholder: _,
            } => vec![target.as_mut(), method.as_mut()],
            Expression::NullsafeMethodCall {
                target,
                question_arrow: _,
                method,
                arguments,
            } => vec![target.as_mut(), method.as_mut(), arguments],
            Expression::StaticMethodCall {
                target,
                double_colon: _,
                method,
                arguments,
            } => vec![target.as_mut(), method, arguments],
            Expression::StaticVariableMethodCall {
                target,
                double_colon: _,
                method,
                arguments,
            } => vec![target.as_mut(), method, arguments],
            Expression::StaticMethodClosureCreation {
                target,
                double_colon: _,
                method,
                placeholder: _,
            } => vec![target.as_mut(), method],
            Expression::StaticVariableMethodClosureCreation {
                target,
                double_colon: _,
                method,
                placeholder: _,
            } => vec![target.as_mut(), method],
            Expression::PropertyFetch {
                target,
                arrow: _,
                property,
            } => vec![target.as_mut(), property.as_mut()],
            Expression::NullsafePropertyFetch {
                target,
                question_arrow: _,
                property,
            } => vec![target.as_mut(), property.as_mut()],
            Expression::StaticPropertyFetch {
                target,
                double_colon: _,
                property,
            } => vec![target.as_mut(), property],
            Expression::ConstantFetch {
                target,
                double_colon: _,
                constant,
            } => vec![target.as_mut(), constant],
            Expression::Static => vec![],
            Expression::Self_ => vec![],
            Expression::Parent => vec![],
            Expression::ShortArray {
                start: _,
                items,
                end: _,
            } => vec![items],
            Expression::Array {
                array: _,
                start: _,
                items,
                end: _,
            } => vec![items],
            Expression::List {
                list: _,
                start: _,
                items,
                end: _,
            } => items.iter_mut().map(|item| item as &mut dyn Node).collect(),
            Expression::Closure(closure) => closure.children(),
            Expression::ArrowFunction(function) => function.children(),
            Expression::New {
                new: _,
                target,
                arguments,
            } => {
                let mut children: Vec<&mut dyn Node> = vec![target.as_mut()];
                if let Some(arguments) = arguments {
                    children.push(arguments);
                }
                children
            }
            Expression::InterpolatedString { parts } => {
                parts.iter_mut().map(|part| part as &mut dyn Node).collect()
            }
            Expression::Heredoc { parts } => {
                parts.iter_mut().map(|part| part as &mut dyn Node).collect()
            }
            Expression::Nowdoc { value: _ } => vec![],
            Expression::ShellExec { parts } => {
                parts.iter_mut().map(|part| part as &mut dyn Node).collect()
            }
            Expression::AnonymousClass(class) => class.children(),
            Expression::Bool { value: _ } => vec![],
            Expression::ArrayIndex {
                array: _,
                left_bracket: _,
                index,
                right_bracket: _,
            } => {
                let mut children: Vec<&mut dyn Node> = vec![];
                if let Some(index) = index {
                    children.push(index.as_mut());
                }
                children
            }
            Expression::Null => vec![],
            Expression::MagicConstant(constant) => constant.children(),
            Expression::ShortTernary {
                condition,
                question_colon: _,
                r#else,
            } => vec![condition.as_mut(), r#else.as_mut()],
            Expression::Ternary {
                condition,
                question: _,
                then,
                colon: _,
                r#else,
            } => vec![condition.as_mut(), then.as_mut(), r#else.as_mut()],
            Expression::Coalesce {
                lhs,
                double_question: _,
                rhs,
            } => vec![lhs.as_mut(), rhs.as_mut()],
            Expression::Clone { target } => vec![target.as_mut()],
            Expression::Match {
                keyword: _,
                left_parenthesis: _,
                condition,
                right_parenthesis: _,
                left_brace: _,
                default,
                arms,
                right_brace: _,
            } => {
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
            Expression::Throw { value } => vec![value.as_mut()],
            Expression::Yield { key, value } => {
                let mut children: Vec<&mut dyn Node> = vec![];
                if let Some(key) = key {
                    children.push(key.as_mut());
                }
                if let Some(value) = value {
                    children.push(value.as_mut());
                }
                children
            }
            Expression::YieldFrom { value } => vec![value.as_mut()],
            Expression::Cast {
                cast: _,
                kind: _,
                value,
            } => vec![value.as_mut()],
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
