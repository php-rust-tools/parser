use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::node::Node;
use crate::parser::ast::Expression;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum ArithmeticOperation {
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

impl Node for ArithmeticOperation {
    fn children(&self) -> Vec<&dyn Node> {
        match self {
            ArithmeticOperation::Addition { left, right, .. } => {
                vec![left.as_ref(), right.as_ref()]
            }
            ArithmeticOperation::Subtraction { left, right, .. } => {
                vec![left.as_ref(), right.as_ref()]
            }
            ArithmeticOperation::Multiplication { left, right, .. } => {
                vec![left.as_ref(), right.as_ref()]
            }
            ArithmeticOperation::Division { left, right, .. } => {
                vec![left.as_ref(), right.as_ref()]
            }
            ArithmeticOperation::Modulo { left, right, .. } => vec![left.as_ref(), right.as_ref()],
            ArithmeticOperation::Exponentiation { left, right, .. } => {
                vec![left.as_ref(), right.as_ref()]
            }
            ArithmeticOperation::Negative { right, .. } => vec![right.as_ref()],
            ArithmeticOperation::Positive { right, .. } => vec![right.as_ref()],
            ArithmeticOperation::PreIncrement { right, .. } => vec![right.as_ref()],
            ArithmeticOperation::PostIncrement { left, .. } => vec![left.as_ref()],
            ArithmeticOperation::PreDecrement { right, .. } => vec![right.as_ref()],
            ArithmeticOperation::PostDecrement { left, .. } => vec![left.as_ref()],
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum AssignmentOperation {
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

impl Node for AssignmentOperation {
    fn children(&self) -> Vec<&dyn Node> {
        match self {
            AssignmentOperation::Assign { left, right, .. } => vec![left.as_ref(), right.as_ref()],
            AssignmentOperation::Addition { left, right, .. } => {
                vec![left.as_ref(), right.as_ref()]
            }
            AssignmentOperation::Subtraction { left, right, .. } => {
                vec![left.as_ref(), right.as_ref()]
            }
            AssignmentOperation::Multiplication { left, right, .. } => {
                vec![left.as_ref(), right.as_ref()]
            }
            AssignmentOperation::Division { left, right, .. } => {
                vec![left.as_ref(), right.as_ref()]
            }
            AssignmentOperation::Modulo { left, right, .. } => vec![left.as_ref(), right.as_ref()],
            AssignmentOperation::Exponentiation { left, right, .. } => {
                vec![left.as_ref(), right.as_ref()]
            }
            AssignmentOperation::Concat { left, right, .. } => vec![left.as_ref(), right.as_ref()],
            AssignmentOperation::BitwiseAnd { left, right, .. } => {
                vec![left.as_ref(), right.as_ref()]
            }
            AssignmentOperation::BitwiseOr { left, right, .. } => {
                vec![left.as_ref(), right.as_ref()]
            }
            AssignmentOperation::BitwiseXor { left, right, .. } => {
                vec![left.as_ref(), right.as_ref()]
            }
            AssignmentOperation::LeftShift { left, right, .. } => {
                vec![left.as_ref(), right.as_ref()]
            }
            AssignmentOperation::RightShift { left, right, .. } => {
                vec![left.as_ref(), right.as_ref()]
            }
            AssignmentOperation::Coalesce { left, right, .. } => {
                vec![left.as_ref(), right.as_ref()]
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum BitwiseOperation {
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

impl Node for BitwiseOperation {
    fn children(&self) -> Vec<&dyn Node> {
        match self {
            BitwiseOperation::And { left, right, .. } => vec![left.as_ref(), right.as_ref()],
            BitwiseOperation::Or { left, right, .. } => vec![left.as_ref(), right.as_ref()],
            BitwiseOperation::Xor { left, right, .. } => vec![left.as_ref(), right.as_ref()],
            BitwiseOperation::LeftShift { left, right, .. } => vec![left.as_ref(), right.as_ref()],
            BitwiseOperation::RightShift { left, right, .. } => vec![left.as_ref(), right.as_ref()],
            BitwiseOperation::Not { right, .. } => vec![right.as_ref()],
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum ComparisonOperation {
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

impl Node for ComparisonOperation {
    fn children(&self) -> Vec<&dyn Node> {
        match self {
            ComparisonOperation::Equal { left, right, .. } => vec![left.as_ref(), right.as_ref()],
            ComparisonOperation::Identical { left, right, .. } => {
                vec![left.as_ref(), right.as_ref()]
            }
            ComparisonOperation::NotEqual { left, right, .. } => {
                vec![left.as_ref(), right.as_ref()]
            }
            ComparisonOperation::AngledNotEqual { left, right, .. } => {
                vec![left.as_ref(), right.as_ref()]
            }
            ComparisonOperation::NotIdentical { left, right, .. } => {
                vec![left.as_ref(), right.as_ref()]
            }
            ComparisonOperation::LessThan { left, right, .. } => {
                vec![left.as_ref(), right.as_ref()]
            }
            ComparisonOperation::GreaterThan { left, right, .. } => {
                vec![left.as_ref(), right.as_ref()]
            }
            ComparisonOperation::LessThanOrEqual { left, right, .. } => {
                vec![left.as_ref(), right.as_ref()]
            }
            ComparisonOperation::GreaterThanOrEqual { left, right, .. } => {
                vec![left.as_ref(), right.as_ref()]
            }
            ComparisonOperation::Spaceship { left, right, .. } => {
                vec![left.as_ref(), right.as_ref()]
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum LogicalOperation {
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

impl Node for LogicalOperation {
    fn children(&self) -> Vec<&dyn Node> {
        match self {
            LogicalOperation::And { left, right, .. } => vec![left.as_ref(), right.as_ref()],
            LogicalOperation::Or { left, right, .. } => vec![left.as_ref(), right.as_ref()],
            LogicalOperation::Not { right, .. } => vec![right.as_ref()],
            LogicalOperation::LogicalAnd { left, right, .. } => vec![left.as_ref(), right.as_ref()],
            LogicalOperation::LogicalOr { left, right, .. } => vec![left.as_ref(), right.as_ref()],
            LogicalOperation::LogicalXor { left, right, .. } => vec![left.as_ref(), right.as_ref()],
        }
    }
}
