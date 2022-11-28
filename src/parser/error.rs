use std::fmt::Display;

use crate::Span;
use crate::Type;

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub enum ParseError {
    ExpectedToken(Vec<String>, Option<String>, Span),
    MultipleModifiers(String, Span),
    UnexpectedToken(String, Span),
    UnexpectedEndOfFile,
    StandaloneTypeUsedInCombination(Type, Span),
    InvalidClassStatement(String, Span),
    InvalidAbstractFinalFlagCombination(Span),
    ConstantCannotBeStatic(Span),
    ConstantCannotBePrivateFinal(Span),
    TraitCannotContainConstant(Span),
    TryWithoutCatchOrFinally(Span),
    InvalidCatchArgumentType(Span),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExpectedToken(expected, found, span) => {
                let length  = expected.len();
                let expected = if length >= 2 {
                    let (left, right) = expected.split_at(length - 1);

                    format!("{}, or {}", left.join(", "), right[0])
                } else {
                   expected.join(",")
                };

                match found {
                    Some(token) => write!(f, "Parse error: unexpected token `{}`, expecting {} on line {} column {}", token, expected, span.0, span.1),
                    None => write!(f, "Parse error: unexpected end of file, expecting {} on line {} column {}", expected, span.0, span.1),
                }
            },
            Self::MultipleModifiers(modifier, span) => write!(f, "Multiple {} modifiers are not allowed on line {} column {}", modifier, span.0, span.1),
            Self::UnexpectedToken(message, span) => write!(f, "Parse error: unexpected token {} on line {} column {}", message, span.0, span.1),
            Self::InvalidClassStatement(message, span) => write!(f, "Parse error: {} on line {} column {}", message, span.0, span.1),
            Self::UnexpectedEndOfFile => write!(f, "Parse error: unexpected end of file."),
            Self::InvalidAbstractFinalFlagCombination(span) => write!(f, "Parse error: final cannot be used on an abstract class member on line {}", span.0),
            Self::ConstantCannotBeStatic(span) => write!(f, "Parse error: class constant cannot be marked static on line {}", span.0),
            Self::ConstantCannotBePrivateFinal(span) => write!(f, "Parse error: private class constant cannot be marked final since it is not visible to other classes on line {}", span.0),
            Self::TraitCannotContainConstant(span) => write!(f, "Parse error: traits cannot contain constants on line {}", span.0),
            Self::TryWithoutCatchOrFinally(span) => write!(f, "Parse error: cannot use try without catch or finally on line {}", span.0),
            Self::InvalidCatchArgumentType(span) => write!(f, "Parse error: catch types must either describe a single type or union of types on line {}", span.0),
            Self::StandaloneTypeUsedInCombination(r#type, span) => write!(f, "Parse error: {} can only be used as a standalone type on line {}", r#type, span.0)
        }
    }
}
