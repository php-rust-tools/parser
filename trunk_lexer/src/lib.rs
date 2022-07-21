#![feature(let_chains)]

mod token;
mod lexer;

pub use token::{Token, TokenKind, Span, OpenTagKind};
pub use lexer::{Lexer, LexerError};