use crate::lexer::token::Token;
use crate::lexer::token::TokenKind;

/// Token stream.
///
/// # Examples
///
/// ```rust
/// use php_parser_rs::lexer::token::Token;
/// use php_parser_rs::lexer::token::TokenKind;
/// use php_parser_rs::lexer::stream::TokenStream;
///
/// let tokens = vec![
///     Token { kind: TokenKind::SingleLineComment("// some class".into()), span: (1, 1) },
///     Token { kind: TokenKind::Readonly, span: (2, 1) },
///     Token { kind: TokenKind::Class, span: (2, 10) },
///     Token { kind: TokenKind::Enum, span: (2, 16) },
///     Token { kind: TokenKind::LeftBrace, span: (2, 21) },
///     Token { kind: TokenKind::SingleLineComment("// empty body!".into()), span: (3, 1) },
///     Token { kind: TokenKind::RightBrace, span: (4, 1) },
///     Token { kind: TokenKind::Eof, span: (0, 0) },
/// ];
///
/// let mut stream = TokenStream::new(tokens);
///
/// assert!(matches!(stream.current().kind, TokenKind::Readonly));
/// assert!(matches!(stream.peek().kind, TokenKind::Class));
/// assert!(matches!(stream.lookahead(1).kind, TokenKind::Enum));
/// assert!(matches!(stream.lookahead(2).kind, TokenKind::LeftBrace));
/// assert!(matches!(stream.lookahead(3).kind, TokenKind::RightBrace));
/// assert!(matches!(stream.lookahead(4).kind, TokenKind::Eof));
/// assert!(matches!(stream.lookahead(5).kind, TokenKind::Eof));
///
/// stream.next();
///
/// assert!(matches!(stream.current().kind, TokenKind::Class));
///
/// stream.next();
/// stream.next();
/// stream.next();
///
/// assert!(matches!(stream.current().kind, TokenKind::RightBrace));
///
/// stream.next();
///
/// assert!(matches!(stream.current().kind, TokenKind::Eof));
/// assert!(stream.is_eof());
///
/// assert_eq!(stream.comments(), vec![
///     Token { kind: TokenKind::SingleLineComment("// some class".into()), span: (1, 1) },
///     Token { kind: TokenKind::SingleLineComment("// empty body!".into()), span: (3, 1) },
/// ]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenStream<'a> {
    tokens: &'a [Token],
    length: usize,
    comments: Vec<&'a Token>,
    cursor: usize,
}

/// Token stream.
impl<'a> TokenStream<'a> {
    pub fn new(tokens: &'a [Token]) -> TokenStream {
        let length = tokens.len();

        let mut stream = TokenStream {
            tokens,
            length,
            comments: vec![],
            cursor: 0,
        };

        stream.collect_comments();

        stream
    }

    /// Move cursor to next token.
    ///
    /// Comments are collected.
    pub fn next(&mut self) {
        self.cursor += 1;
        self.collect_comments();
    }

    /// Get current token.
    pub const fn current(&self) -> &'a Token {
        let position = if self.cursor >= self.length {
            self.length - 1
        } else {
            self.cursor
        };

        &self.tokens[position]
    }

    /// Peek next token.
    ///
    /// All comments are skipped.
    pub const fn peek(&self) -> &'a Token {
        self.peek_nth(1)
    }

    /// Peek nth+1 token.
    ///
    /// All comments are skipped.
    pub const fn lookahead(&self, n: usize) -> &'a Token {
        self.peek_nth(n + 1)
    }

    /// Peek nth token.
    ///
    /// All comments are skipped.
    #[inline(always)]
    const fn peek_nth(&self, n: usize) -> &'a Token {
        let mut cursor = self.cursor + 1;
        let mut target = 1;
        loop {
            if cursor >= self.length {
                return &self.tokens[self.length - 1];
            }

            let current = &self.tokens[cursor];

            if matches!(
                current.kind,
                TokenKind::SingleLineComment(_)
                    | TokenKind::MultiLineComment(_)
                    | TokenKind::HashMarkComment(_)
                    | TokenKind::DocumentComment(_)
            ) {
                cursor += 1;
                continue;
            }

            if target == n {
                return current;
            }

            target += 1;
            cursor += 1;
        }
    }

    /// Check if current token is EOF.
    pub fn is_eof(&self) -> bool {
        if self.cursor >= self.length {
            return true;
        }

        self.tokens[self.cursor].kind == TokenKind::Eof
    }

    /// Get all comments.
    #[allow(dead_code)]
    pub fn comments(&mut self) -> Vec<&'a Token> {
        let mut comments = vec![];

        std::mem::swap(&mut self.comments, &mut comments);

        comments
    }

    fn collect_comments(&mut self) {
        loop {
            if self.cursor >= self.length {
                break;
            }

            let current = &self.tokens[self.cursor];

            if !matches!(
                current.kind,
                TokenKind::SingleLineComment(_)
                    | TokenKind::MultiLineComment(_)
                    | TokenKind::HashMarkComment(_)
                    | TokenKind::DocumentComment(_)
            ) {
                break;
            }

            self.comments.push(current);
            self.cursor += 1;
        }
    }
}

impl<'a> Default for TokenStream<'a> {
    fn default() -> Self {
        Self::new(&[])
    }
}

impl<'a> From<&'a Vec<Token>> for TokenStream<'a> {
    fn from(tokens: &'a Vec<Token>) -> Self {
        Self::new(tokens.as_slice())
    }
}
