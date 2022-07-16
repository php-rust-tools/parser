pub type Span = (usize, usize);

#[derive(Debug, PartialEq)]
pub enum OpenTagKind {
    Full,
}

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    Identifier(String),
    Variable(String),
    Function,
    Return,
    Echo,
    Int(i64),
    Plus,
    Minus,
    LessThan,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    OpenTag(OpenTagKind),
    CloseTag,
    SemiColon,
    InlineHtml(String),
}

#[derive(Debug)]
pub struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) span: Span,
}