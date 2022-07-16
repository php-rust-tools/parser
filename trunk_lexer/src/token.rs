pub type Span = (usize, usize);

#[derive(Debug, PartialEq, Clone)]
pub enum OpenTagKind {
    Full,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    Identifier(String),
    Variable(String),
    Function,
    If,
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
    Comma,
    InlineHtml(String),
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}