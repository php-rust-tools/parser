use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::parser::ast::attributes::AttributeGroup;
use crate::parser::ast::constant::ClassishConstant;
use crate::parser::ast::functions::Method;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::Expression;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct UnitEnumCase {
    pub attributes: Vec<AttributeGroup>, // `#[Foo]`
    pub start: Span,                     // `case`
    pub name: SimpleIdentifier,          // `Bar`
    pub end: Span,                       // `;`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum UnitEnumMember {
    Case(UnitEnumCase),         // `case Bar;`
    Method(Method),             // `public function foo(): void { ... }`
    Constant(ClassishConstant), // `public const FOO = 123;`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct UnitEnumBody {
    pub left_brace: Span,             // `{`
    pub members: Vec<UnitEnumMember>, // `...`
    pub right_brace: Span,            // `}`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct UnitEnum {
    pub attributes: Vec<AttributeGroup>,   // `#[Foo]`
    pub span: Span,                        // `enum`
    pub name: SimpleIdentifier,            // `Foo`
    pub implements: Vec<SimpleIdentifier>, // `implements Bar`
    pub body: UnitEnumBody,                // `{ ... }`
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "type", content = "value")]
pub enum BackedEnumType {
    String(Span, Span), // `:` + `string`
    Int(Span, Span),    // `:` + `int`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BackedEnumCase {
    pub attributes: Vec<AttributeGroup>, // `#[Foo]`
    pub case: Span,                      // `case`
    pub name: SimpleIdentifier,          // `Bar`
    pub equals: Span,                    // `=`
    pub value: Expression,               // `123`
    pub semicolon: Span,                 // `;`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum BackedEnumMember {
    Case(BackedEnumCase),
    Method(Method),
    Constant(ClassishConstant),
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BackedEnumBody {
    pub left_brace: Span,               // `{`
    pub members: Vec<BackedEnumMember>, // `...`
    pub right_brace: Span,              // `}`
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BackedEnum {
    pub attributes: Vec<AttributeGroup>,   // `#[Foo]`
    pub span: Span,                        // `enum`
    pub name: SimpleIdentifier,            // `Foo`
    pub backed_type: BackedEnumType,       // `: string`
    pub implements: Vec<SimpleIdentifier>, // `implements Bar`
    pub body: BackedEnumBody,              // `{ ... }`
}
