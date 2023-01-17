use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::node::Node;
use crate::parser::ast::Expression;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum ArithmeticOperationExpression {
    Addition {
        left: Box<Expression>,
        plus: Span,
        right: Box<Expression>,
    },
    Subtraction {
        left: Box<Expression>,
        minus: Span,
        right: Box<Expression>,
    },
    Multiplication {
        left: Box<Expression>,
        asterisk: Span,
        right: Box<Expression>,
    },
    Division {
        left: Box<Expression>,
        slash: Span,
        right: Box<Expression>,
    },
    Modulo {
        left: Box<Expression>,
        percent: Span,
        right: Box<Expression>,
    },
    Exponentiation {
        left: Box<Expression>,
        pow: Span,
        right: Box<Expression>,
    },
    Negative {
        minus: Span,
        right: Box<Expression>,
    },
    Positive {
        plus: Span,
        right: Box<Expression>,
    },
    PreIncrement {
        increment: Span,
        right: Box<Expression>,
    },
    PostIncrement {
        left: Box<Expression>,
        increment: Span,
    },
    PreDecrement {
        decrement: Span,
        right: Box<Expression>,
    },
    PostDecrement {
        left: Box<Expression>,
        decrement: Span,
    },
}

impl Node for ArithmeticOperationExpression {
    fn children(&mut self) -> Vec<&mut dyn Node> {
        match self {
            ArithmeticOperationExpression::Addition { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            ArithmeticOperationExpression::Subtraction { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            ArithmeticOperationExpression::Multiplication { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            ArithmeticOperationExpression::Division { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            ArithmeticOperationExpression::Modulo { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            ArithmeticOperationExpression::Exponentiation { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            ArithmeticOperationExpression::Negative { right, .. } => vec![right.as_mut()],
            ArithmeticOperationExpression::Positive { right, .. } => vec![right.as_mut()],
            ArithmeticOperationExpression::PreIncrement { right, .. } => vec![right.as_mut()],
            ArithmeticOperationExpression::PostIncrement { left, .. } => vec![left.as_mut()],
            ArithmeticOperationExpression::PreDecrement { right, .. } => vec![right.as_mut()],
            ArithmeticOperationExpression::PostDecrement { left, .. } => vec![left.as_mut()],
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum AssignmentOperationExpression {
    Assign {
        left: Box<Expression>,
        equals: Span,
        right: Box<Expression>,
    },
    Addition {
        left: Box<Expression>,
        plus_equals: Span,
        right: Box<Expression>,
    },
    Subtraction {
        left: Box<Expression>,
        minus_equals: Span,
        right: Box<Expression>,
    },
    Multiplication {
        left: Box<Expression>,
        asterisk_equals: Span,
        right: Box<Expression>,
    },
    Division {
        left: Box<Expression>,
        slash_equals: Span,
        right: Box<Expression>,
    },
    Modulo {
        left: Box<Expression>,
        percent_equals: Span,
        right: Box<Expression>,
    },
    Exponentiation {
        left: Box<Expression>,
        pow_equals: Span,
        right: Box<Expression>,
    },
    Concat {
        left: Box<Expression>,
        dot_equals: Span,
        right: Box<Expression>,
    },
    BitwiseAnd {
        left: Box<Expression>,
        ampersand_equals: Span,
        right: Box<Expression>,
    },
    BitwiseOr {
        left: Box<Expression>,
        pipe_equals: Span,
        right: Box<Expression>,
    },
    BitwiseXor {
        left: Box<Expression>,
        caret_equals: Span,
        right: Box<Expression>,
    },
    LeftShift {
        left: Box<Expression>,
        left_shift_equals: Span,
        right: Box<Expression>,
    },
    RightShift {
        left: Box<Expression>,
        right_shift_equals: Span,
        right: Box<Expression>,
    },
    Coalesce {
        left: Box<Expression>,
        coalesce_equals: Span,
        right: Box<Expression>,
    },
}

impl Node for AssignmentOperationExpression {
    fn children(&mut self) -> Vec<&mut dyn Node> {
        match self {
            AssignmentOperationExpression::Assign { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            AssignmentOperationExpression::Addition { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            AssignmentOperationExpression::Subtraction { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            AssignmentOperationExpression::Multiplication { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            AssignmentOperationExpression::Division { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            AssignmentOperationExpression::Modulo { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            AssignmentOperationExpression::Exponentiation { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            AssignmentOperationExpression::Concat { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            AssignmentOperationExpression::BitwiseAnd { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            AssignmentOperationExpression::BitwiseOr { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            AssignmentOperationExpression::BitwiseXor { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            AssignmentOperationExpression::LeftShift { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            AssignmentOperationExpression::RightShift { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            AssignmentOperationExpression::Coalesce { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum BitwiseOperationExpression {
    And {
        left: Box<Expression>,
        and: Span,
        right: Box<Expression>,
    },
    Or {
        left: Box<Expression>,
        or: Span,
        right: Box<Expression>,
    },
    Xor {
        left: Box<Expression>,
        xor: Span,
        right: Box<Expression>,
    },
    LeftShift {
        left: Box<Expression>,
        left_shift: Span,
        right: Box<Expression>,
    },
    RightShift {
        left: Box<Expression>,
        right_shift: Span,
        right: Box<Expression>,
    },
    Not {
        not: Span,
        right: Box<Expression>,
    },
}

impl Node for BitwiseOperationExpression {
    fn children(&mut self) -> Vec<&mut dyn Node> {
        match self {
            BitwiseOperationExpression::And { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            BitwiseOperationExpression::Or { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            BitwiseOperationExpression::Xor { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            BitwiseOperationExpression::LeftShift { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            BitwiseOperationExpression::RightShift { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            BitwiseOperationExpression::Not { right, .. } => vec![right.as_mut()],
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum ComparisonOperationExpression {
    Equal {
        left: Box<Expression>,
        double_equals: Span,
        right: Box<Expression>,
    },
    Identical {
        left: Box<Expression>,
        triple_equals: Span,
        right: Box<Expression>,
    },
    NotEqual {
        left: Box<Expression>,
        bang_equals: Span,
        right: Box<Expression>,
    },
    AngledNotEqual {
        left: Box<Expression>,
        angled_left_right: Span,
        right: Box<Expression>,
    },
    NotIdentical {
        left: Box<Expression>,
        bang_double_equals: Span,
        right: Box<Expression>,
    },
    LessThan {
        left: Box<Expression>,
        less_than: Span,
        right: Box<Expression>,
    },
    GreaterThan {
        left: Box<Expression>,
        greater_than: Span,
        right: Box<Expression>,
    },
    LessThanOrEqual {
        left: Box<Expression>,
        less_than_equals: Span,
        right: Box<Expression>,
    },
    GreaterThanOrEqual {
        left: Box<Expression>,
        greater_than_equals: Span,
        right: Box<Expression>,
    },
    Spaceship {
        left: Box<Expression>,
        spaceship: Span,
        right: Box<Expression>,
    },
}

impl Node for ComparisonOperationExpression {
    fn children(&mut self) -> Vec<&mut dyn Node> {
        match self {
            ComparisonOperationExpression::Equal { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            ComparisonOperationExpression::Identical { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            ComparisonOperationExpression::NotEqual { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            ComparisonOperationExpression::AngledNotEqual { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            ComparisonOperationExpression::NotIdentical { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            ComparisonOperationExpression::LessThan { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            ComparisonOperationExpression::GreaterThan { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            ComparisonOperationExpression::LessThanOrEqual { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            ComparisonOperationExpression::GreaterThanOrEqual { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            ComparisonOperationExpression::Spaceship { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum LogicalOperationExpression {
    And {
        left: Box<Expression>,
        double_ampersand: Span,
        right: Box<Expression>,
    },
    Or {
        left: Box<Expression>,
        double_pipe: Span,
        right: Box<Expression>,
    },
    Not {
        bang: Span,
        right: Box<Expression>,
    },
    LogicalAnd {
        left: Box<Expression>,
        and: Span,
        right: Box<Expression>,
    },
    LogicalOr {
        left: Box<Expression>,
        or: Span,
        right: Box<Expression>,
    },
    LogicalXor {
        left: Box<Expression>,
        xor: Span,
        right: Box<Expression>,
    },
}

impl Node for LogicalOperationExpression {
    fn children(&mut self) -> Vec<&mut dyn Node> {
        match self {
            LogicalOperationExpression::And { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            LogicalOperationExpression::Or { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            LogicalOperationExpression::Not { right, .. } => vec![right.as_mut()],
            LogicalOperationExpression::LogicalAnd { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            LogicalOperationExpression::LogicalOr { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
            LogicalOperationExpression::LogicalXor { left, right, .. } => {
                vec![left.as_mut(), right.as_mut()]
            }
        }
    }
}
