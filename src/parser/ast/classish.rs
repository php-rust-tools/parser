use crate::lexer::token::Span;
use crate::parser::ast::identifiers::Identifier;
use crate::parser::ast::modifiers::ConstantModifierGroup;
use crate::parser::ast::Expression;

#[derive(Debug, Clone, PartialEq)]
pub struct ClassishConstant {
    pub start: Span,
    pub end: Span,
    pub name: Identifier,
    pub value: Expression,
    pub modifiers: ConstantModifierGroup,
}
