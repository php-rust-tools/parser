use crate::{ByteString, OpenTagKind, Token, TokenKind};

#[derive(Debug)]
pub enum LexerState {
    Initial,
    Scripting,
}

#[allow(dead_code)]
#[derive(Default)]
pub struct LexerConfig {
    short_tags: bool,
}

#[allow(dead_code)]
pub struct Lexer {
    config: LexerConfig,
    state: LexerState,
    chars: Vec<u8>,
    cursor: usize,
    current: Option<u8>,
    peek: Option<u8>,
    col: usize,
    line: usize,
}

impl Lexer {
    pub fn new(config: Option<LexerConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            state: LexerState::Initial,
            chars: Vec::new(),
            cursor: 0,
            current: None,
            peek: None,
            line: 1,
            col: 0,
        }
    }

    pub fn tokenize<B: ?Sized + AsRef<[u8]>>(
        &mut self,
        input: &B,
    ) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();
        self.chars = input.as_ref().to_vec();

        self.next();
        self.next();

        while self.peek.is_some() {
            match self.state {
                // The "Initial" state is used to parse inline HTML. It is essentially a catch-all
                // state that will build up a single token buffer until it encounters an open tag
                // of some description.
                LexerState::Initial => {
                    tokens.append(&mut self.initial()?);
                }
                // The scripting state is entered when an open tag is encountered in the source code.
                // This tells the lexer to start analysing characters at PHP tokens instead of inline HTML.
                LexerState::Scripting => {
                    while let Some(c) = self.peek {
                        if !c.is_ascii_whitespace() && ![b'\n', b'\t', b'\r'].contains(&c) {
                            break;
                        }

                        if c == b'\n' {
                            self.line += 1;
                            self.col = 0;
                        } else {
                            self.col += 1;
                        }

                        self.next();
                    }

                    // If we have consumed whitespace and then reached the end of the file, we should break.
                    if self.peek.is_none() {
                        break;
                    }

                    tokens.push(self.scripting()?);
                }
            }
        }

        Ok(tokens)
    }

    #[allow(dead_code)]
    fn initial(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut buffer = Vec::new();
        while let Some(char) = self.current {
            match char {
                b'<' => {
                    // This is disgusting and can most definitely be tidied up with a multi-peek iterator.
                    if let Some(b'?') = self.peek {
                        self.next();

                        if let Some(b'p') = self.peek {
                            self.next();

                            if let Some(b'h') = self.peek {
                                self.next();

                                if let Some(b'p') = self.peek {
                                    self.next();

                                    self.col += 4;

                                    self.enter_state(LexerState::Scripting);

                                    let mut tokens = vec![];

                                    if !buffer.is_empty() {
                                        tokens.push(Token {
                                            kind: TokenKind::InlineHtml(buffer.into()),
                                            span: (self.line, self.col.saturating_sub(5)),
                                        });
                                    }

                                    tokens.push(Token {
                                        kind: TokenKind::OpenTag(OpenTagKind::Full),
                                        span: (self.line, self.col),
                                    });

                                    return Ok(tokens);
                                }
                            } else {
                                self.col += 3;

                                buffer.push(b'h');
                            }
                        } else {
                            self.col += 2;

                            buffer.push(b'?');
                        }
                    } else {
                        self.next();

                        self.col += 1;

                        buffer.push(char);
                    }
                }
                _ => {
                    self.next();
                    buffer.push(char);
                }
            }
        }

        Ok(vec![Token {
            kind: TokenKind::InlineHtml(buffer.into()),
            span: (self.line, self.col),
        }])
    }

    fn scripting(&mut self) -> Result<Token, LexerError> {
        // We should never reach this point since we have the empty checks surrounding
        // the call to this function, but it's better to be safe than sorry.
        if self.peek.is_none() {
            return Err(LexerError::UnexpectedEndOfFile);
        }

        // Since we have the check above, we can safely unwrap the result of `.next()`
        // to help reduce the amount of indentation.
        self.next();
        let char = self.current.unwrap();

        let kind = match char {
            b'@' => {
                self.col += 1;

                TokenKind::At
            }
            b'!' => {
                self.col += 1;

                if let Some(b'=') = self.peek {
                    self.col += 1;

                    self.next();

                    if let Some(b'=') = self.peek {
                        self.col += 1;

                        self.next();

                        TokenKind::BangDoubleEquals
                    } else {
                        TokenKind::BangEquals
                    }
                } else {
                    TokenKind::Bang
                }
            }
            b'&' => {
                self.col += 1;

                if let Some(b'&') = self.peek {
                    self.col += 1;

                    self.next();

                    TokenKind::BooleanAnd
                } else {
                    TokenKind::Ampersand
                }
            }
            b'?' => {
                // This is a close tag, we can enter "Initial" mode again.
                if let Some(b'>') = self.peek {
                    self.next();
                    self.next();

                    self.col += 2;

                    self.enter_state(LexerState::Initial);

                    TokenKind::CloseTag
                } else if let Some(b'?') = self.peek {
                    self.col += 1;

                    self.next();

                    if let Some(b'=') = self.peek {
                        self.col += 1;

                        self.next();

                        TokenKind::CoalesceEqual
                    } else {
                        TokenKind::Coalesce
                    }
                } else if let Some(b':') = self.peek {
                    self.col += 1;
                    self.next();
                    TokenKind::QuestionColon
                } else if self.try_read(b"->") {
                    self.col += 1;
                    self.skip(2);
                    TokenKind::NullsafeArrow
                } else {
                    TokenKind::Question
                }
            }
            b'=' => {
                if let Some(b'=') = self.peek {
                    self.next();

                    if let Some(b'=') = self.peek {
                        self.next();

                        self.col += 3;

                        TokenKind::TripleEquals
                    } else {
                        self.col += 2;

                        TokenKind::DoubleEquals
                    }
                } else if let Some(b'>') = self.peek {
                    self.next();
                    self.col += 1;
                    TokenKind::DoubleArrow
                } else {
                    self.col += 1;

                    TokenKind::Equals
                }
            }
            // Single quoted string.
            b'\'' => {
                self.col += 1;

                let mut buffer = Vec::new();
                let mut escaping = false;

                while let Some(n) = self.peek {
                    if !escaping && n == b'\'' {
                        self.next();

                        break;
                    }

                    if n == b'\\' && !escaping {
                        escaping = true;
                        self.next();
                        continue;
                    }

                    if escaping && [b'\\', b'\''].contains(&n) {
                        escaping = false;
                        buffer.push(n);
                        self.next();
                        continue;
                    }

                    if n == b'\n' {
                        self.line += 1;
                        self.col = 0;
                    } else {
                        self.col += 1;
                    }

                    escaping = false;

                    buffer.push(n);
                    self.next();
                }

                TokenKind::ConstantString(buffer.into())
            }
            b'"' => {
                self.col += 1;

                let mut buffer = Vec::new();
                let mut escaping = false;

                while let Some(n) = self.peek {
                    if !escaping && n == b'"' {
                        self.next();

                        break;
                    }

                    if n == b'\\' && !escaping {
                        escaping = true;
                        self.next();
                        continue;
                    }

                    if escaping && [b'\\', b'"'].contains(&n) {
                        escaping = false;
                        buffer.push(n);
                        self.next();
                        continue;
                    }

                    if n == b'\n' {
                        self.line += 1;
                        self.col = 0;
                    } else {
                        self.col += 1;
                    }

                    escaping = false;

                    buffer.push(n);
                    self.next();
                }

                TokenKind::ConstantString(buffer.into())
            }
            b'$' => {
                let mut buffer = Vec::new();

                self.col += 1;

                while let Some(n) = self.peek {
                    match n {
                        b'0'..=b'9' if !buffer.is_empty() => {
                            self.col += 1;
                            buffer.push(n);
                            self.next();
                        }
                        b'a'..=b'z' | b'A'..=b'Z' | 0x80..=0xff | b'_' => {
                            self.col += 1;

                            buffer.push(n);
                            self.next();
                        }
                        _ => break,
                    }
                }

                TokenKind::Variable(buffer.into())
            }
            b'.' => {
                self.col += 1;

                if let Some(b'0'..=b'9') = self.peek {
                    let mut buffer = String::from("0.");
                    let mut underscore = false;

                    while let Some(n) = self.peek {
                        match n {
                            b'0'..=b'9' => {
                                underscore = false;
                                buffer.push(n as char);
                                self.next();

                                self.col += 1;
                            }
                            b'_' => {
                                if underscore {
                                    return Err(LexerError::UnexpectedCharacter(n));
                                }

                                underscore = true;
                                self.next();

                                self.col += 1;
                            }
                            _ => break,
                        }
                    }

                    TokenKind::Float(buffer.parse().unwrap())
                } else if let Some(b'.') = self.peek {
                    self.next();

                    self.col += 1;

                    if let Some(b'.') = self.peek {
                        self.next();

                        self.col += 1;

                        TokenKind::Ellipsis
                    } else {
                        todo!("don't know how to handle this case yet, it should just be 2 Dot tokens...")
                    }
                } else if let Some(b'=') = self.peek {
                    self.next();
                    self.col += 1;
                    TokenKind::DotEquals
                } else {
                    TokenKind::Dot
                }
            }
            b'0'..=b'9' => {
                let mut buffer = String::from(char as char);
                let mut underscore = false;
                let mut is_float = false;

                self.col += 1;

                while let Some(n) = self.peek {
                    match n {
                        b'0'..=b'9' => {
                            underscore = false;
                            buffer.push(n as char);
                            self.next();

                            self.col += 1;
                        }
                        b'.' => {
                            if is_float {
                                return Err(LexerError::UnexpectedCharacter(n));
                            }

                            is_float = true;
                            buffer.push(n as char);
                            self.next();
                            self.col += 1;
                        }
                        b'_' => {
                            if underscore {
                                return Err(LexerError::UnexpectedCharacter(n));
                            }

                            underscore = true;
                            self.next();

                            self.col += 1;
                        }
                        _ => break,
                    }
                }

                if is_float {
                    TokenKind::Float(buffer.parse().unwrap())
                } else {
                    TokenKind::Int(buffer.parse().unwrap())
                }
            }
            b'\\' => {
                self.col += 1;

                if self
                    .peek
                    .map_or(false, |n| n == b'_' || n.is_ascii_alphabetic())
                {
                    match self.scripting()? {
                        Token {
                            kind:
                                TokenKind::Identifier(ByteString(mut i))
                                | TokenKind::QualifiedIdentifier(ByteString(mut i)),
                            ..
                        } => {
                            i.insert(0, b'\\');
                            TokenKind::FullyQualifiedIdentifier(i.into())
                        }
                        s => unreachable!("{:?}", s),
                    }
                } else {
                    TokenKind::NamespaceSeparator
                }
            }
            _ if char.is_ascii_alphabetic() || char == b'_' => {
                self.col += 1;

                let mut qualified = false;
                let mut last_was_slash = false;

                let mut buffer = vec![char];
                while let Some(next) = self.peek {
                    if next.is_ascii_alphanumeric() || next == b'_' {
                        buffer.push(next);
                        self.next();
                        self.col += 1;
                        last_was_slash = false;
                        continue;
                    }

                    if next == b'\\' && !last_was_slash {
                        qualified = true;
                        last_was_slash = true;
                        buffer.push(next);
                        self.next();
                        self.col += 1;
                        continue;
                    }

                    break;
                }

                if qualified {
                    TokenKind::QualifiedIdentifier(buffer.into())
                } else {
                    identifier_to_keyword(&buffer).unwrap_or_else(|| TokenKind::Identifier(buffer.into()))
                }
            }
            b'/' | b'#' => {
                self.col += 1;

                fn read_till_end_of_line(s: &mut Lexer) -> Vec<u8> {
                    s.col += 1;

                    let mut buffer = Vec::new();

                    while let Some(c) = s.peek {
                        if c == b'\n' {
                            break;
                        }

                        buffer.push(c);
                        s.next();
                    }

                    buffer
                }

                if char == b'/' && self.peek == Some(b'*') {
                    let mut buffer = vec![char];

                    while self.peek.is_some() {
                        self.next();

                        let t = self.current.unwrap();

                        match t {
                            b'*' => {
                                if let Some(b'/') = self.peek {
                                    self.col += 2;
                                    buffer.extend_from_slice(b"*/");
                                    self.next();
                                    break;
                                } else {
                                    self.col += 1;
                                    buffer.push(t);
                                }
                            }
                            b'\n' => {
                                self.line += 1;
                                self.col = 0;

                                buffer.push(b'\n');
                            }
                            _ => {
                                self.col += 1;

                                buffer.push(t);
                            }
                        }
                    }

                    if buffer.starts_with(b"/**") {
                        TokenKind::DocComment(buffer.into())
                    } else {
                        TokenKind::Comment(buffer.into())
                    }
                } else if let Some(b'=') = self.peek {
                    self.col += 1;
                    self.next();
                    TokenKind::SlashEquals
                } else if char == b'/' && self.peek != Some(b'/') {
                    TokenKind::Slash
                } else if char == b'#' && self.peek == Some(b'[') {
                    TokenKind::Attribute
                } else {
                    self.next();
                    let current = self.current.unwrap();
                    let mut buffer = read_till_end_of_line(self);
                    buffer.splice(0..0, [char, current]);

                    TokenKind::Comment(buffer.into())
                }
            }
            b'*' => {
                self.col += 1;

                if let Some(b'*') = self.peek {
                    self.col += 1;
                    self.next();
                    TokenKind::Pow
                } else if let Some(b'=') = self.peek {
                    self.col += 1;
                    self.next();
                    TokenKind::AsteriskEqual
                } else {
                    TokenKind::Asterisk
                }
            }
            b'|' => {
                self.col += 1;

                if let Some(b'|') = self.peek {
                    self.col += 1;

                    self.next();

                    TokenKind::BooleanOr
                } else {
                    TokenKind::Pipe
                }
            }
            b'{' => {
                self.col += 1;
                TokenKind::LeftBrace
            }
            b'}' => {
                self.col += 1;
                TokenKind::RightBrace
            }
            b'(' => {
                self.col += 1;

                if self.try_read(b"string)") {
                    self.col += 7;
                    self.skip(8);

                    TokenKind::StringCast
                } else if self.try_read(b"object)") {
                    self.col += 7;
                    self.skip(8);

                    TokenKind::ObjectCast
                } else if self.try_read(b"bool)") {
                    self.col += 5;
                    self.skip(6);
                    TokenKind::BoolCast
                } else if self.try_read(b"int)") {
                    self.col += 4;
                    self.skip(5);
                    TokenKind::IntCast
                } else if self.try_read(b"float)") {
                    self.col += 6;
                    self.skip(7);
                    TokenKind::DoubleCast
                } else {
                    TokenKind::LeftParen
                }
            }
            b')' => {
                self.col += 1;
                TokenKind::RightParen
            }
            b';' => {
                self.col += 1;
                TokenKind::SemiColon
            }
            b'+' => {
                self.col += 1;

                if let Some(b'=') = self.peek {
                    self.col += 1;

                    self.next();

                    TokenKind::PlusEquals
                } else if let Some(b'+') = self.peek {
                    self.col += 1;

                    self.next();

                    TokenKind::Increment
                } else {
                    TokenKind::Plus
                }
            }
            b'-' => {
                self.col += 1;

                if let Some(b'>') = self.peek {
                    self.col += 1;

                    self.next();

                    TokenKind::Arrow
                } else if let Some(b'=') = self.peek {
                    self.col += 1;
                    self.next();
                    TokenKind::MinusEquals
                } else {
                    TokenKind::Minus
                }
            }
            b'<' => {
                self.col += 1;

                if let Some(b'=') = self.peek {
                    self.next();

                    self.col += 1;

                    TokenKind::LessThanEquals
                } else if let Some(b'<') = self.peek {
                    self.next();

                    if let Some(b'<') = self.peek {
                        // TODO: Handle both heredocs and nowdocs.
                        self.next();

                        todo!("heredocs & nowdocs");
                    } else {
                        TokenKind::LeftShift
                    }
                } else {
                    TokenKind::LessThan
                }
            }
            b'>' => {
                self.col += 1;

                if let Some(b'=') = self.peek {
                    self.next();

                    self.col += 1;

                    TokenKind::GreaterThanEquals
                } else {
                    TokenKind::GreaterThan
                }
            }
            b',' => {
                self.col += 1;
                TokenKind::Comma
            }
            b'[' => {
                self.col += 1;
                TokenKind::LeftBracket
            }
            b']' => {
                self.col += 1;
                TokenKind::RightBracket
            }
            b':' => {
                self.col += 1;

                if let Some(b':') = self.peek {
                    self.col += 1;

                    self.next();
                    TokenKind::DoubleColon
                } else {
                    TokenKind::Colon
                }
            }
            _ => unimplemented!(
                "<scripting> char: {}, line: {}, col: {}",
                char,
                self.line,
                self.col
            ),
        };

        Ok(Token {
            kind,
            span: (self.line, self.col),
        })
    }

    fn enter_state(&mut self, state: LexerState) {
        self.state = state;
    }

    fn char_at(&self, idx: usize) -> Option<&u8> {
        self.chars.get(idx)
    }

    fn try_read(&self, search: &'static [u8]) -> bool {
        if self.current.is_none() || self.peek.is_none() {
            return false;
        }

        let start = self.cursor.saturating_sub(1);
        let mut buffer = Vec::new();

        for i in 0..search.len() {
            match self.char_at(start + i) {
                Some(char) => buffer.push(*char),
                _ => return false,
            };
        }

        buffer == search
    }

    fn skip(&mut self, count: usize) {
        for _ in 0..count {
            self.next();
        }
    }

    fn next(&mut self) {
        self.current = self.peek;
        self.peek = self.chars.get(self.cursor).cloned();
        self.cursor += 1;
    }
}

#[allow(dead_code)]
fn identifier_to_keyword(ident: &[u8]) -> Option<TokenKind> {
    Some(match ident {
        b"readonly" => TokenKind::Readonly,
        b"global" => TokenKind::Global,
        b"match" => TokenKind::Match,
        b"abstract" => TokenKind::Abstract,
        b"array" => TokenKind::Array,
        b"as" => TokenKind::As,
        b"break" => TokenKind::Break,
        b"case" => TokenKind::Case,
        b"catch" => TokenKind::Catch,
        b"class" => TokenKind::Class,
        b"clone" => TokenKind::Clone,
        b"continue" => TokenKind::Continue,
        b"const" => TokenKind::Const,
        b"declare" => TokenKind::Declare,
        b"default" => TokenKind::Default,
        b"do" => TokenKind::Do,
        b"echo" => TokenKind::Echo,
        b"else" => TokenKind::Else,
        b"elseif" => TokenKind::ElseIf,
        b"enum" => TokenKind::Enum,
        b"extends" => TokenKind::Extends,
        b"false" | b"FALSE" => TokenKind::False,
        b"final" => TokenKind::Final,
        b"finally" => TokenKind::Finally,
        b"fn" => TokenKind::Fn,
        b"for" => TokenKind::For,
        b"foreach" => TokenKind::Foreach,
        b"function" => TokenKind::Function,
        b"if" => TokenKind::If,
        b"include" => TokenKind::Include,
        b"include_once" => TokenKind::IncludeOnce,
        b"implements" => TokenKind::Implements,
        b"interface" => TokenKind::Interface,
        b"instanceof" => TokenKind::Instanceof,
        b"namespace" => TokenKind::Namespace,
        b"new" => TokenKind::New,
        b"null" | b"NULL" => TokenKind::Null,
        b"private" => TokenKind::Private,
        b"protected" => TokenKind::Protected,
        b"public" => TokenKind::Public,
        b"require" => TokenKind::Require,
        b"require_once" => TokenKind::RequireOnce,
        b"return" => TokenKind::Return,
        b"static" => TokenKind::Static,
        b"switch" => TokenKind::Switch,
        b"throw" => TokenKind::Throw,
        b"trait" => TokenKind::Trait,
        b"true" | b"TRUE" => TokenKind::True,
        b"try" => TokenKind::Try,
        b"use" => TokenKind::Use,
        b"var" => TokenKind::Var,
        b"yield" => TokenKind::Yield,
        b"__DIR__" => TokenKind::DirConstant,
        b"while" => TokenKind::While,
        _ => return None,
    })
}

#[derive(Debug)]
pub enum LexerError {
    UnexpectedEndOfFile,
    UnexpectedCharacter(u8),
}

#[cfg(test)]
mod tests {
    use super::Lexer;
    use crate::{ByteString, OpenTagKind, Token, TokenKind};

    macro_rules! open {
        () => {
            TokenKind::OpenTag(OpenTagKind::Full)
        };
        ($kind:expr) => {
            TokenKind::OpenTag($kind)
        };
    }
    macro_rules! int {
        ($i:expr) => {
            TokenKind::Int($i)
        };
    }

    fn var<B: ?Sized + Into<ByteString>>(v: B) -> TokenKind {
        TokenKind::Variable(v.into())
    }

    #[test]
    fn basic_tokens() {
        assert_tokens("<?php ?>", &[open!(), TokenKind::CloseTag]);
    }

    #[test]
    fn close_tag_followed_by_content() {
        assert_tokens(
            "<?php ?> <html>",
            &[
                open!(),
                TokenKind::CloseTag,
                TokenKind::InlineHtml(" <html>".into()),
            ],
        );
    }

    #[test]
    fn inline_html() {
        assert_tokens(
            "Hello, world!\n<?php",
            &[TokenKind::InlineHtml("Hello, world!\n".into()), open!()],
        );
    }

    #[test]
    fn keywords() {
        assert_tokens("<?php function if else elseif echo return class extends implements public protected private static null NULL true TRUE false FALSE use const namespace interface new foreach instanceof readonly", &[
            open!(),
            TokenKind::Function,
            TokenKind::If,
            TokenKind::Else,
            TokenKind::ElseIf,
            TokenKind::Echo,
            TokenKind::Return,
            TokenKind::Class,
            TokenKind::Extends,
            TokenKind::Implements,
            TokenKind::Public,
            TokenKind::Protected,
            TokenKind::Private,
            TokenKind::Static,
            TokenKind::Null,
            TokenKind::Null,
            TokenKind::True,
            TokenKind::True,
            TokenKind::False,
            TokenKind::False,
            TokenKind::Use,
            TokenKind::Const,
            TokenKind::Namespace,
            TokenKind::Interface,
            TokenKind::New,
            TokenKind::Foreach,
            TokenKind::Instanceof,
            TokenKind::Readonly,
        ]);
    }

    #[test]
    fn casts() {
        assert_tokens(
            "<?php (object) (string)",
            &[open!(), TokenKind::ObjectCast, TokenKind::StringCast],
        );
    }

    #[test]
    fn constant_single_quote_strings() {
        assert_tokens(
            r#"<?php 'Hello, world!' 'I\'m a developer.' 'This is a backslash \\.' 'This is a multi-line
string.'"#,
            &[
                open!(),
                TokenKind::ConstantString("Hello, world!".into()),
                TokenKind::ConstantString("I'm a developer.".into()),
                TokenKind::ConstantString("This is a backslash \\.".into()),
                TokenKind::ConstantString("This is a multi-line\nstring.".into()),
            ],
        );
    }

    #[test]
    fn single_line_comments() {
        assert_tokens(
            r#"<?php
        // Single line comment.
        # Another single line comment.
        "#,
            &[
                open!(),
                TokenKind::Comment("// Single line comment.".into()),
                TokenKind::Comment("# Another single line comment.".into()),
            ],
        );
    }

    #[test]
    fn multi_line_comments() {
        assert_tokens(
            r#"<?php
/*
Hello
*/"#,
            &[open!(), TokenKind::Comment("/*\nHello\n*/".into())],
        )
    }

    #[test]
    fn multi_line_comments_before_structure() {
        assert_tokens(
            r#"<?php
/*
Hello
*/
function"#,
            &[
                open!(),
                TokenKind::Comment("/*\nHello\n*/".into()),
                TokenKind::Function,
            ],
        )
    }

    #[test]
    fn vars() {
        assert_tokens(
            b"<?php $one $_one $One $one_one $a1 $\xff",
            &[
                open!(),
                var("one"),
                var("_one"),
                var("One"),
                var("one_one"),
                var("a1"),
                var(b"\xff"),
            ],
        );
    }

    #[test]
    fn nums() {
        assert_tokens(
            "<?php 1 1_000 1_000_000",
            &[open!(), int!(1), int!(1_000), int!(1_000_000)],
        );
    }

    #[test]
    fn punct() {
        assert_tokens(
            "<?php {}();, :: :",
            &[
                open!(),
                TokenKind::LeftBrace,
                TokenKind::RightBrace,
                TokenKind::LeftParen,
                TokenKind::RightParen,
                TokenKind::SemiColon,
                TokenKind::Comma,
                TokenKind::DoubleColon,
                TokenKind::Colon,
            ],
        );
    }

    #[test]
    fn sigils() {
        assert_tokens("<?php ->", &[open!(), TokenKind::Arrow]);
    }

    #[test]
    fn math() {
        assert_tokens(
            "<?php + - <",
            &[
                open!(),
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::LessThan,
            ],
        );
    }

    #[test]
    fn identifiers() {
        assert_tokens(
            "<?php \\ Unqualified Is\\Qualified",
            &[
                open!(),
                TokenKind::NamespaceSeparator,
                TokenKind::Identifier("Unqualified".into()),
                TokenKind::QualifiedIdentifier("Is\\Qualified".into()),
            ],
        );
    }

    #[test]
    fn equals() {
        assert_tokens(
            "<?php = == ===",
            &[
                open!(),
                TokenKind::Equals,
                TokenKind::DoubleEquals,
                TokenKind::TripleEquals,
            ],
        );
    }

    #[test]
    fn span_tracking() {
        let spans = get_spans("<?php hello_world()");

        assert_eq!(spans, &[(1, 4), (1, 16), (1, 17), (1, 18),]);

        let spans = get_spans(
            r#"<?php

function hello_world() {

}"#,
        );

        assert_eq!(
            spans,
            &[(1, 4), (3, 8), (3, 20), (3, 21), (3, 22), (3, 24), (5, 1),]
        );
    }

    #[test]
    fn floats() {
        assert_tokens(
            "<?php 200.5 .05",
            &[open!(), TokenKind::Float(200.5), TokenKind::Float(0.05)],
        );
    }

    fn assert_tokens<B: ?Sized + AsRef<[u8]>>(source: &B, expected: &[TokenKind]) {
        let mut kinds = vec![];

        for token in get_tokens(source) {
            kinds.push(token.kind);
        }

        assert_eq!(kinds, expected);
    }

    fn get_spans(source: &str) -> Vec<(usize, usize)> {
        let tokens = get_tokens(source);
        let mut spans = vec![];

        for token in tokens {
            spans.push(token.span);
        }

        spans
    }

    fn get_tokens<B: ?Sized + AsRef<[u8]>>(source: &B) -> Vec<Token> {
        let mut lexer = Lexer::new(None);
        lexer.tokenize(source).unwrap()
    }
}
