use std::fmt::Display;

use crate::lexer::token::Span;

pub type SyntaxResult<T> = Result<T, SyntaxError>;

#[derive(Debug, Eq, PartialEq)]
pub enum SyntaxError {
    UnexpectedEndOfFile(Span),
    UnexpectedError(Span),
    UnexpectedCharacter(u8, Span),
    InvalidHaltCompiler(Span),
    InvalidOctalEscape(Span),
    InvalidOctalLiteral(Span),
    InvalidUnicodeEscape(Span),
    UnpredictableState(Span),
}

impl Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedEndOfFile(span) => write!(
                f,
                "Syntax Error: unexpected end of file on line {} column {}",
                span.0, span.1
            ),
            Self::UnexpectedError(span) => write!(
                f,
                "Syntax Error: unexpected error on line {} column {}",
                span.0, span.1
            ),
            Self::UnexpectedCharacter(char, span) => write!(
                f,
                "Syntax Error: unexpected character `{:?}` on line {} column {}",
                *char as char, span.0, span.1
            ),
            Self::InvalidHaltCompiler(span) => write!(
                f,
                "Syntax Error: invalid halt compiler on line {} column {}",
                span.0, span.1
            ),
            Self::InvalidOctalEscape(span) => write!(
                f,
                "Syntax Error: invalid octal escape on line {} column {}",
                span.0, span.1
            ),
            Self::InvalidOctalLiteral(span) => write!(
                f,
                "Syntax Error: invalid octal literal on line {} column {}",
                span.0, span.1
            ),
            Self::InvalidUnicodeEscape(span) => write!(
                f,
                "Syntax Error: invalid unicode escape on line {} column {}",
                span.0, span.1
            ),
            Self::UnpredictableState(span) => write!(
                f,
                "Syntax Error: Reached an unpredictable state on line {} column {}",
                span.0, span.1
            ),
        }
    }
}
