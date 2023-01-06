use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::node::Node;
use crate::parser::ast::attributes::AttributeGroup;
use crate::parser::ast::constant::ClassishConstant;
use crate::parser::ast::functions::AbstractConstructor;
use crate::parser::ast::functions::AbstractMethod;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::utils::CommaSeparated;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum InterfaceMember {
    Constant(ClassishConstant),       // `public const FOO = 123;`
    Constructor(AbstractConstructor), // `public function __construct(): void;`
    Method(AbstractMethod),           // `public function foo(): void;`
}

impl Node for InterfaceMember {
    fn children(&self) -> Vec<&dyn Node> {
        match self {
            InterfaceMember::Constant(constant) => vec![constant],
            InterfaceMember::Constructor(constructor) => vec![constructor],
            InterfaceMember::Method(method) => vec![method],
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InterfaceExtends {
    pub extends: Span,                             // `extends`
    pub parents: CommaSeparated<SimpleIdentifier>, // `Foo`, `Bar`
}

impl Node for InterfaceExtends {
    fn children(&self) -> Vec<&dyn Node> {
        self.parents.children()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InterfaceBody {
    pub left_brace: Span,              // `{`
    pub members: Vec<InterfaceMember>, // `public const FOO = 123;`, `public function foo(): void;`
    pub right_brace: Span,             // `}`
}

impl Node for InterfaceBody {
    fn children(&self) -> Vec<&dyn Node> {
        self.members
            .iter()
            .map(|member| member as &dyn Node)
            .collect()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InterfaceStatement {
    pub attributes: Vec<AttributeGroup>,   // `#[Foo]`
    pub interface: Span,                   // `interface`
    pub name: SimpleIdentifier,            // `Foo`
    pub extends: Option<InterfaceExtends>, // `extends Bar`
    pub body: InterfaceBody,               // `{ ... }`
}

impl Node for InterfaceStatement {
    fn children(&self) -> Vec<&dyn Node> {
        let mut children: Vec<&dyn Node> = vec![&self.name];
        if let Some(extends) = &self.extends {
            children.push(extends);
        }
        children
    }
}
