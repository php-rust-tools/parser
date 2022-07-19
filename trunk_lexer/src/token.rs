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
    Class,
    Public,
    Protected,
    Private,
    Static,
    If,
    Else,
    ElseIf,
    Return,
    Echo,
    Int(i64),
    Plus,
    Minus,
    Equals,
    DoubleEquals,
    TripleEquals,
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
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Default for Token {
    fn default() -> Self {
        Self { kind: TokenKind::Eof, span: (0, 0) }
    }
}