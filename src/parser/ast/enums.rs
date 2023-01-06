use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::node::Node;
use crate::parser::ast::attributes::AttributeGroup;
use crate::parser::ast::constant::ClassishConstant;
use crate::parser::ast::functions::ConcreteMethod;
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

impl Node for UnitEnumCase {
    fn children(&self) -> Vec<&dyn Node> {
        vec![&self.name as &dyn Node]
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum UnitEnumMember {
    Case(UnitEnumCase),         // `case Bar;`
    Method(ConcreteMethod),     // `public function foo(): void { ... }`
    Constant(ClassishConstant), // `public const FOO = 123;`
}

impl Node for UnitEnumMember {
    fn children(&self) -> Vec<&dyn Node> {
        match self {
            UnitEnumMember::Case(case) => vec![case],
            UnitEnumMember::Method(method) => vec![method],
            UnitEnumMember::Constant(constant) => vec![constant],
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct UnitEnumBody {
    pub left_brace: Span,             // `{`
    pub members: Vec<UnitEnumMember>, // `...`
    pub right_brace: Span,            // `}`
}

impl Node for UnitEnumBody {
    fn children(&self) -> Vec<&dyn Node> {
        self.members.iter().map(|m| m as &dyn Node).collect()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct UnitEnumStatement {
    pub attributes: Vec<AttributeGroup>,   // `#[Foo]`
    pub r#enum: Span,                      // `enum`
    pub name: SimpleIdentifier,            // `Foo`
    pub implements: Vec<SimpleIdentifier>, // `implements Bar`
    pub body: UnitEnumBody,                // `{ ... }`
}

impl Node for UnitEnumStatement {
    fn children(&self) -> Vec<&dyn Node> {
        let mut children: Vec<&dyn Node> = vec![&self.name];
        for implement in &self.implements {
            children.push(implement);
        }
        children.push(&self.body);
        children
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "type", content = "value")]
pub enum BackedEnumType {
    String(Span, Span), // `:` + `string`
    Int(Span, Span),    // `:` + `int`
}

impl Node for BackedEnumType {
    //
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

impl Node for BackedEnumCase {
    fn children(&self) -> Vec<&dyn Node> {
        vec![&self.name, &self.value]
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum BackedEnumMember {
    Case(BackedEnumCase),
    Method(ConcreteMethod),
    Constant(ClassishConstant),
}

impl Node for BackedEnumMember {
    fn children(&self) -> Vec<&dyn Node> {
        match self {
            BackedEnumMember::Case(case) => vec![case],
            BackedEnumMember::Method(method) => vec![method],
            BackedEnumMember::Constant(constant) => vec![constant],
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BackedEnumBody {
    pub left_brace: Span,               // `{`
    pub members: Vec<BackedEnumMember>, // `...`
    pub right_brace: Span,              // `}`
}

impl Node for BackedEnumBody {
    fn children(&self) -> Vec<&dyn Node> {
        self.members.iter().map(|m| m as &dyn Node).collect()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BackedEnumStatement {
    pub attributes: Vec<AttributeGroup>,   // `#[Foo]`
    pub r#enum: Span,                      // `enum`
    pub name: SimpleIdentifier,            // `Foo`
    pub backed_type: BackedEnumType,       // `: string`
    pub implements: Vec<SimpleIdentifier>, // `implements Bar`
    pub body: BackedEnumBody,              // `{ ... }`
}

impl Node for BackedEnumStatement {
    fn children(&self) -> Vec<&dyn Node> {
        let mut children: Vec<&dyn Node> = vec![&self.name, &self.backed_type];
        for implement in &self.implements {
            children.push(implement);
        }
        children.push(&self.body);
        children
    }
}
