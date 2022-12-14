use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::parser::ast::attributes::AttributeGroup;
use crate::parser::ast::constant::ClassishConstant;
use crate::parser::ast::functions::Method;
use crate::parser::ast::identifiers::SimpleIdentifier;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum InterfaceMember {
    Constant(ClassishConstant), // `public const FOO = 123;`
    Method(Method),             // `public function foo(): void;`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InterfaceExtends {
    pub span: Span,                     // `extends`
    pub parents: Vec<SimpleIdentifier>, // `Foo`, `Bar`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InterfaceBody {
    pub start: Span,                   // `{`
    pub end: Span,                     // `}`
    pub members: Vec<InterfaceMember>, // `public const FOO = 123;`, `public function foo(): void;`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Interface {
    pub span: Span,                        // `interface`
    pub attributes: Vec<AttributeGroup>,   // `#[Foo]`
    pub name: SimpleIdentifier,            // `Foo`
    pub extends: Option<InterfaceExtends>, // `extends Bar`
    pub body: InterfaceBody,               // `{ ... }`
}
