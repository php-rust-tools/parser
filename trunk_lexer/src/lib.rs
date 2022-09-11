mod lexer;
mod token;

pub use lexer::{Lexer, LexerError};
pub use token::{OpenTagKind, Span, Token, TokenKind};
