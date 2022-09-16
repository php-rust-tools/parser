use crate::{ByteString, OpenTagKind, Token, TokenKind};

#[derive(Debug)]
pub enum LexerState {
    Initial,
    Scripting,
    Halted,
}

#[allow(dead_code)]
#[derive(Default)]
pub struct LexerConfig {
    short_tags: bool,
}

#[allow(dead_code)]
pub struct Lexer {
    config: LexerConfig,
    state_stack: Vec<LexerState>,
    chars: Vec<u8>,
    cursor: usize,
    current: Option<u8>,
    col: usize,
    line: usize,
}

impl Lexer {
    pub fn new(config: Option<LexerConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            state_stack: vec![LexerState::Initial],
            chars: Vec::new(),
            cursor: 0,
            current: None,
            line: 1,
            col: 1,
        }
    }

    pub fn tokenize<B: ?Sized + AsRef<[u8]>>(
        &mut self,
        input: &B,
    ) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();
        self.chars = input.as_ref().to_vec();

        self.current = self.chars.get(0).copied();

        while self.current.is_some() {
            match self.state_stack.last().unwrap() {
                // The "Initial" state is used to parse inline HTML. It is essentially a catch-all
                // state that will build up a single token buffer until it encounters an open tag
                // of some description.
                LexerState::Initial => {
                    tokens.append(&mut self.initial()?);
                }
                // The scripting state is entered when an open tag is encountered in the source code.
                // This tells the lexer to start analysing characters at PHP tokens instead of inline HTML.
                LexerState::Scripting => {
                    while let Some(c) = self.current {
                        if !c.is_ascii_whitespace() && ![b'\n', b'\t', b'\r'].contains(&c) {
                            break;
                        }

                        self.next();
                    }

                    // If we have consumed whitespace and then reached the end of the file, we should break.
                    if self.current.is_none() {
                        break;
                    }

                    tokens.push(self.scripting()?);
                }
                // The "Halted" state is entered when the `__halt_compiler` token is encountered.
                // In this state, all the text that follows is no longer parsed as PHP as is collected
                // into a single "InlineHtml" token (kind of cheating, oh well).
                LexerState::Halted => {
                    tokens.push(Token {
                        kind: TokenKind::InlineHtml(self.chars[self.cursor..].into()),
                        span: (self.line, self.col),
                    });
                    break;
                }
            }
        }

        Ok(tokens)
    }

    fn initial(&mut self) -> Result<Vec<Token>, LexerError> {
        let inline_span = (self.line, self.col);
        let mut buffer = Vec::new();
        while let Some(char) = self.current {
            if self.try_read(b"<?php") {
                let tag_span = (self.line, self.col);
                self.skip(5);

                self.enter_state(LexerState::Scripting);

                let mut tokens = vec![];

                if !buffer.is_empty() {
                    tokens.push(Token {
                        kind: TokenKind::InlineHtml(buffer.into()),
                        span: inline_span,
                    });
                }

                tokens.push(Token {
                    kind: TokenKind::OpenTag(OpenTagKind::Full),
                    span: tag_span,
                });

                return Ok(tokens);
            }

            self.next();
            buffer.push(char);
        }

        Ok(vec![Token {
            kind: TokenKind::InlineHtml(buffer.into()),
            span: inline_span,
        }])
    }

    fn scripting(&mut self) -> Result<Token, LexerError> {
        let span = (self.line, self.col);
        let kind = match self.peek_buf() {
            [b'@', ..] => {
                self.next();

                TokenKind::At
            }
            [b'!', b'=', b'=', ..] => {
                self.skip(3);
                TokenKind::BangDoubleEquals
            }
            [b'!', b'=', ..] => {
                self.skip(2);
                TokenKind::BangEquals
            }
            [b'!', ..] => {
                self.next();
                TokenKind::BangEquals
            }
            [b'&', b'&', ..] => {
                self.skip(2);
                TokenKind::BooleanAnd
            }
            [b'&', b'=', ..] => {
                self.skip(2);
                TokenKind::AmpersandEquals
            }
            [b'&', ..] => {
                self.next();
                TokenKind::Ampersand
            }
            [b'?', b'>', ..] => {
                // This is a close tag, we can enter "Initial" mode again.
                self.skip(2);

                self.enter_state(LexerState::Initial);

                TokenKind::CloseTag
            }
            [b'?', b'?', b'=', ..] => {
                self.skip(3);
                TokenKind::CoalesceEqual
            }
            [b'?', b'?', ..] => {
                self.skip(2);
                TokenKind::Coalesce
            }
            [b'?', b':', ..] => {
                self.skip(2);
                TokenKind::QuestionColon
            }
            [b'?', b'-', b'>', ..] => {
                self.skip(3);
                TokenKind::NullsafeArrow
            }
            [b'?', ..] => {
                self.next();
                TokenKind::Question
            }
            [b'=', b'>', ..] => {
                self.skip(2);
                TokenKind::DoubleArrow
            }
            [b'=', b'=', b'=', ..] => {
                self.skip(3);
                TokenKind::TripleEquals
            }
            [b'=', b'=', ..] => {
                self.skip(2);
                TokenKind::DoubleEquals
            }
            [b'=', ..] => {
                self.next();
                TokenKind::Equals
            }
            // Single quoted string.
            [b'\'', ..] => {
                self.next();
                self.tokenize_single_quote_string()?
            }
            [b'b' | b'B', b'\'', ..] => {
                self.skip(2);
                self.tokenize_single_quote_string()?
            }
            [b'"', ..] => {
                self.next();
                self.tokenize_double_quote_string()?
            }
            [b'b' | b'B', b'"', ..] => {
                self.skip(2);
                self.tokenize_double_quote_string()?
            }
            [b'$', ..] => {
                self.next();
                self.tokenize_variable()
            }
            [b'.', b'=', ..] => {
                self.skip(2);
                TokenKind::DotEquals
            }
            [b'.', b'0'..=b'9', ..] => {
                self.next();
                self.tokenize_number(String::from("0."), true)?
            }
            [b'.', b'.', b'.', ..] => {
                self.skip(3);
                TokenKind::Ellipsis
            }
            [b'.', ..] => {
                self.next();
                TokenKind::Dot
            }
            &[b @ b'0'..=b'9', ..] => {
                self.next();
                self.tokenize_number(String::from(b as char), false)?
            }
            &[b'\\', n, ..] if n == b'_' || n.is_ascii_alphabetic() => {
                self.next();

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
            }
            [b'\\', ..] => {
                self.next();
                TokenKind::NamespaceSeparator
            }
            &[b, ..] if b.is_ascii_alphabetic() || b == b'_' => {
                self.next();
                let mut qualified = false;
                let mut last_was_slash = false;

                let mut buffer = vec![b];
                while let Some(next) = self.current {
                    if next.is_ascii_alphanumeric() || next == b'_' {
                        buffer.push(next);
                        self.next();
                        last_was_slash = false;
                        continue;
                    }

                    if next == b'\\' && !last_was_slash {
                        qualified = true;
                        last_was_slash = true;
                        buffer.push(next);
                        self.next();
                        continue;
                    }

                    break;
                }

                if qualified {
                    TokenKind::QualifiedIdentifier(buffer.into())
                } else {
                    let kind = identifier_to_keyword(&buffer)
                        .unwrap_or_else(|| TokenKind::Identifier(buffer.into()));

                    if kind == TokenKind::HaltCompiler {
                        match self.peek_buf() {
                            [b'(', b')', b';', ..] => {
                                self.skip(3);
                                self.col += 3;
                                self.enter_state(LexerState::Halted);
                            }
                            _ => return Err(LexerError::InvalidHaltCompiler),
                        }
                    }

                    kind
                }
            }
            [b'/', b'*', ..] => {
                self.next();
                let mut buffer = vec![b'/'];

                while self.current.is_some() {
                    match self.peek_buf() {
                        [b'*', b'/', ..] => {
                            self.skip(2);
                            buffer.extend_from_slice(b"*/");
                            break;
                        }
                        &[t, ..] => {
                            self.next();
                            buffer.push(t);
                        }
                        [] => unreachable!(),
                    }
                }
                self.next();

                if buffer.starts_with(b"/**") {
                    TokenKind::DocComment(buffer.into())
                } else {
                    TokenKind::Comment(buffer.into())
                }
            }
            [b'#', b'[', ..] => {
                self.skip(2);
                TokenKind::Attribute
            }
            &[ch @ b'/', b'/', ..] | &[ch @ b'#', ..] => {
                let mut buffer = if ch == b'/' {
                    self.skip(2);
                    b"//".to_vec()
                } else {
                    self.next();
                    b"#".to_vec()
                };

                while let Some(c) = self.current {
                    if c == b'\n' {
                        break;
                    }

                    buffer.push(c);
                    self.next();
                }

                self.next();

                TokenKind::Comment(buffer.into())
            }
            [b'/', b'=', ..] => {
                self.skip(2);
                TokenKind::SlashEquals
            }
            [b'/', ..] => {
                self.next();
                TokenKind::Slash
            }
            [b'*', b'*', b'=', ..] => {
                self.skip(3);
                TokenKind::PowEquals
            }
            [b'*', b'*', ..] => {
                self.skip(2);
                TokenKind::Pow
            }
            [b'*', b'=', ..] => {
                self.skip(2);
                TokenKind::AsteriskEqual
            }
            [b'*', ..] => {
                self.next();
                TokenKind::Asterisk
            }
            [b'|', b'|', ..] => {
                self.skip(2);
                TokenKind::Pipe
            }
            [b'|', b'=', ..] => {
                self.skip(2);
                TokenKind::PipeEquals
            }
            [b'|', ..] => {
                self.next();
                TokenKind::Pipe
            }
            [b'^', b'=', ..] => {
                self.skip(2);
                TokenKind::CaretEquals
            }
            [b'^', ..] => {
                self.next();
                TokenKind::Caret
            }
            [b'{', ..] => {
                self.next();
                TokenKind::LeftBrace
            }
            [b'}', ..] => {
                self.next();
                TokenKind::RightBrace
            }
            [b'(', ..] => {
                self.next();

                if self.try_read(b"int)") {
                    self.skip(4);
                    TokenKind::IntCast
                } else if self.try_read(b"integer)") {
                    self.skip(8);
                    TokenKind::IntegerCast
                } else if self.try_read(b"bool)") {
                    self.skip(5);
                    TokenKind::BoolCast
                } else if self.try_read(b"boolean)") {
                    self.skip(8);
                    TokenKind::BooleanCast
                } else if self.try_read(b"float)") {
                    self.skip(6);
                    TokenKind::FloatCast
                } else if self.try_read(b"double)") {
                    self.skip(7);
                    TokenKind::DoubleCast
                } else if self.try_read(b"real)") {
                    self.skip(5);
                    TokenKind::RealCast
                } else if self.try_read(b"string)") {
                    self.skip(7);
                    TokenKind::StringCast
                } else if self.try_read(b"binary)") {
                    self.skip(7);
                    TokenKind::BinaryCast
                } else if self.try_read(b"array)") {
                    self.skip(6);
                    TokenKind::ArrayCast
                } else if self.try_read(b"object)") {
                    self.skip(7);
                    TokenKind::ObjectCast
                } else if self.try_read(b"unset)") {
                    self.skip(6);
                    TokenKind::UnsetCast
                } else {
                    TokenKind::LeftParen
                }
            }
            [b')', ..] => {
                self.next();
                TokenKind::RightParen
            }
            [b';', ..] => {
                self.next();
                TokenKind::SemiColon
            }
            [b'+', b'+', ..] => {
                self.skip(2);
                TokenKind::Increment
            }
            [b'+', b'=', ..] => {
                self.skip(2);
                TokenKind::PlusEquals
            }
            [b'+', ..] => {
                self.next();
                TokenKind::Plus
            }
            [b'%', b'=', ..] => {
                self.skip(2);
                TokenKind::PercentEquals
            }
            [b'%', ..] => {
                self.next();
                TokenKind::Percent
            }
            [b'-', b'-', ..] => {
                self.skip(2);
                TokenKind::Decrement
            }
            [b'-', b'>', ..] => {
                self.skip(2);
                TokenKind::Arrow
            }
            [b'-', b'=', ..] => {
                self.skip(2);
                TokenKind::MinusEquals
            }
            [b'-', ..] => {
                self.next();
                TokenKind::Minus
            }
            [b'<', b'<', b'<', ..] => {
                // TODO: Handle both heredocs and nowdocs.
                self.skip(3);

                todo!("heredocs & nowdocs");
            }
            [b'<', b'<', b'=', ..] => {
                self.skip(3);

                TokenKind::LeftShiftEquals
            }
            [b'<', b'<', ..] => {
                self.skip(2);
                TokenKind::LeftShift
            }
            [b'<', b'=', b'>', ..] => {
                self.skip(3);
                TokenKind::Spaceship
            }
            [b'<', b'=', ..] => {
                self.skip(2);
                TokenKind::LessThanEquals
            }
            [b'<', b'>', ..] => {
                self.skip(2);
                TokenKind::AngledLeftRight
            }
            [b'<', ..] => {
                self.next();
                TokenKind::LessThan
            }
            [b'>', b'>', b'=', ..] => {
                self.skip(3);
                TokenKind::RightShiftEquals
            }
            [b'>', b'>', ..] => {
                self.skip(2);
                TokenKind::RightShift
            }
            [b'>', b'=', ..] => {
                self.skip(2);
                TokenKind::GreaterThanEquals
            }
            [b'>', ..] => {
                self.next();
                TokenKind::GreaterThan
            }
            [b',', ..] => {
                self.next();
                TokenKind::Comma
            }
            [b'[', ..] => {
                self.next();
                TokenKind::LeftBracket
            }
            [b']', ..] => {
                self.next();
                TokenKind::RightBracket
            }
            [b':', b':', ..] => {
                self.skip(2);
                TokenKind::DoubleColon
            }
            [b':', ..] => {
                self.next();
                TokenKind::Colon
            }
            &[b'~', ..] => {
                self.next();
                TokenKind::BitwiseNot
            }
            &[b, ..] => unimplemented!(
                "<scripting> char: {}, line: {}, col: {}",
                b as char,
                self.line,
                self.col
            ),
            // We should never reach this point since we have the empty checks surrounding
            // the call to this function, but it's better to be safe than sorry.
            [] => return Err(LexerError::UnexpectedEndOfFile),
        };

        Ok(Token { kind, span })
    }

    fn tokenize_single_quote_string(&mut self) -> Result<TokenKind, LexerError> {
        let mut buffer = Vec::new();

        loop {
            match self.peek_buf() {
                [b'\'', ..] => {
                    self.next();
                    break;
                }
                &[b'\\', b @ b'\'' | b @ b'\\', ..] => {
                    self.skip(2);
                    buffer.push(b);
                }
                &[b, ..] => {
                    self.next();
                    buffer.push(b);
                }
                [] => return Err(LexerError::UnexpectedEndOfFile),
            }
        }

        Ok(TokenKind::ConstantString(buffer.into()))
    }

    fn tokenize_double_quote_string(&mut self) -> Result<TokenKind, LexerError> {
        let mut buffer = Vec::new();

        loop {
            match self.peek_buf() {
                [b'"', ..] => {
                    self.next();
                    break;
                }
                &[b'\\', b @ (b'"' | b'\\' | b'$'), ..] => {
                    self.skip(2);
                    buffer.push(b);
                }
                &[b'\\', b'n', ..] => {
                    self.skip(2);
                    buffer.push(b'\n');
                }
                &[b'\\', b'r', ..] => {
                    self.skip(2);
                    buffer.push(b'\r');
                }
                &[b'\\', b't', ..] => {
                    self.skip(2);
                    buffer.push(b'\t');
                }
                &[b'\\', b'v', ..] => {
                    self.skip(2);
                    buffer.push(b'\x0b');
                }
                &[b'\\', b'e', ..] => {
                    self.skip(2);
                    buffer.push(b'\x1b');
                }
                &[b'\\', b'f', ..] => {
                    self.skip(2);
                    buffer.push(b'\x0c');
                }
                &[b'\\', b'x', b @ (b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F'), ..] => {
                    self.skip(3);

                    let mut hex = String::from(b as char);
                    if let Some(b @ (b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F')) = self.current {
                        self.next();
                        hex.push(b as char);
                    }

                    let b = u8::from_str_radix(&hex, 16).unwrap();
                    buffer.push(b);
                }
                &[b'\\', b'u', b'{', ..] => {
                    self.skip(3);

                    let mut code_point = String::new();
                    while let Some(b @ (b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F')) = self.current {
                        self.next();
                        code_point.push(b as char);
                    }

                    if code_point.is_empty() || self.current != Some(b'}') {
                        return Err(LexerError::InvalidUnicodeEscape);
                    }
                    self.next();

                    let c = if let Ok(c) = u32::from_str_radix(&code_point, 16) {
                        c
                    } else {
                        return Err(LexerError::InvalidUnicodeEscape);
                    };

                    if let Some(c) = char::from_u32(c) {
                        let mut tmp = [0; 4];
                        let bytes = c.encode_utf8(&mut tmp);
                        buffer.extend(bytes.as_bytes());
                    } else {
                        return Err(LexerError::InvalidUnicodeEscape);
                    }
                }
                &[b'\\', b @ b'0'..=b'7', ..] => {
                    self.skip(2);

                    let mut octal = String::from(b as char);
                    if let Some(b @ b'0'..=b'7') = self.current {
                        self.next();
                        octal.push(b as char);
                    }
                    if let Some(b @ b'0'..=b'7') = self.current {
                        self.next();
                        octal.push(b as char);
                    }

                    if let Ok(b) = u8::from_str_radix(&octal, 8) {
                        buffer.push(b);
                    } else {
                        return Err(LexerError::InvalidOctalEscape);
                    }
                }
                &[b, ..] => {
                    self.next();
                    buffer.push(b);
                }
                [] => return Err(LexerError::UnexpectedEndOfFile),
            }
        }

        Ok(TokenKind::ConstantString(buffer.into()))
    }

    fn tokenize_variable(&mut self) -> TokenKind {
        let mut buffer = Vec::new();

        while let Some(n) = self.current {
            match n {
                b'0'..=b'9' if !buffer.is_empty() => {
                    buffer.push(n);
                    self.next();
                }
                b'a'..=b'z' | b'A'..=b'Z' | 0x80..=0xff | b'_' => {
                    buffer.push(n);
                    self.next();
                }
                _ => break,
            }
        }

        if buffer.is_empty() {
            TokenKind::Dollar
        } else {
            TokenKind::Variable(buffer.into())
        }
    }

    fn tokenize_number(
        &mut self,
        mut buffer: String,
        seen_decimal: bool,
    ) -> Result<TokenKind, LexerError> {
        let mut underscore = false;
        let mut is_float = seen_decimal;

        while let Some(n) = self.current {
            match n {
                b'0'..=b'9' => {
                    underscore = false;
                    buffer.push(n as char);
                    self.next();
                }
                b'.' => {
                    if is_float {
                        return Err(LexerError::UnexpectedCharacter(n));
                    }

                    is_float = true;
                    buffer.push(n as char);
                    self.next();
                }
                b'_' => {
                    if underscore {
                        return Err(LexerError::UnexpectedCharacter(n));
                    }

                    underscore = true;
                    self.next();
                }
                _ => break,
            }
        }

        Ok(if is_float {
            TokenKind::Float(buffer.parse().unwrap())
        } else {
            TokenKind::Int(buffer.parse().unwrap())
        })
    }

    fn enter_state(&mut self, state: LexerState) {
        *self.state_stack.last_mut().unwrap() = state;
    }

    fn peek_buf(&self) -> &[u8] {
        &self.chars[self.cursor..]
    }

    fn try_read(&self, search: &'static [u8]) -> bool {
        self.peek_buf().starts_with(search)
    }

    fn skip(&mut self, count: usize) {
        for _ in 0..count {
            self.next();
        }
    }

    fn next(&mut self) {
        match self.current {
            Some(b'\n') => {
                self.line += 1;
                self.col = 1;
            }
            Some(_) => self.col += 1,
            _ => {}
        }
        self.cursor += 1;
        self.current = self.chars.get(self.cursor).copied();
    }
}

fn identifier_to_keyword(ident: &[u8]) -> Option<TokenKind> {
    Some(match ident {
        b"endif" => TokenKind::EndIf,
        b"from" => TokenKind::From,
        b"and" => TokenKind::LogicalAnd,
        b"or" => TokenKind::LogicalOr,
        b"xor" => TokenKind::LogicalXor,
        b"print" => TokenKind::Print,
        b"__halt_compiler" | b"__HALT_COMPILER" => TokenKind::HaltCompiler,
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
        b"goto" => TokenKind::Goto,
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

#[derive(Debug, Eq, PartialEq)]
pub enum LexerError {
    UnexpectedEndOfFile,
    UnexpectedCharacter(u8),
    InvalidHaltCompiler,
    InvalidOctalEscape,
    InvalidUnicodeEscape,
}

#[cfg(test)]
mod tests {
    use super::Lexer;
    use crate::{ByteString, LexerError, OpenTagKind, Token, TokenKind};

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
        use TokenKind::*;

        assert_tokens(
            "<?php (int) (integer) (bool) (boolean) (float) (double) (real) (string) (binary) (array) (object) (unset)",
            &[
                open!(),
                IntCast,
                IntegerCast,
                BoolCast,
                BooleanCast,
                FloatCast,
                DoubleCast,
                RealCast,
                StringCast,
                BinaryCast,
                ArrayCast,
                ObjectCast,
                UnsetCast
            ],
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
    fn binary_strings() {
        assert_tokens(
            r#"<?php b'single' b"double" "#,
            &[
                open!(),
                TokenKind::ConstantString("single".into()),
                TokenKind::ConstantString("double".into()),
            ],
        )
    }

    #[test]
    fn string_escapes() {
        assert_tokens(
            "<?php '\\ \\' ' ",
            &[open!(), TokenKind::ConstantString("\\ \' ".into())],
        );
        assert_tokens(
            r#"<?php "\n \r \t \v \e \f \\ \$ \" " "#,
            &[
                open!(),
                TokenKind::ConstantString("\n \r \t \x0b \x1b \x0c \\ $ \" ".into()),
            ],
        );
        // octal
        assert_tokens(
            r#"<?php "\0 \7 \66 \377 \9 \0000" "#,
            &[
                open!(),
                TokenKind::ConstantString(b"\0 \x07 \x36 \xff \\9 \00".into()),
            ],
        );
        // hex
        assert_tokens(
            r#"<?php "\x \x0 \xa \xA \xff \xFF" "#,
            &[
                open!(),
                TokenKind::ConstantString(b"\\x \0 \x0a \x0a \xff \xff".into()),
            ],
        );
        // Invalid escapes that should be taken literally.
        assert_tokens(
            r#"<?php "\x \u" "#,
            &[open!(), TokenKind::ConstantString(r"\x \u".into())],
        );
    }

    #[test]
    fn invalid_escapes() {
        assert_error(r#"<?php "\666" "#, LexerError::InvalidOctalEscape);
        assert_error(r#"<?php "\u{" "#, LexerError::InvalidUnicodeEscape);
        assert_error(r#"<?php "\u{}" "#, LexerError::InvalidUnicodeEscape);
        assert_error(r#"<?php "\u{42" "#, LexerError::InvalidUnicodeEscape);
        assert_error(r#"<?php "\u{110000}" "#, LexerError::InvalidUnicodeEscape);
    }

    #[test]
    fn unterminated_strings() {
        assert_error(r#"<?php "unterminated "#, LexerError::UnexpectedEndOfFile);
        assert_error("<?php 'unterminated ", LexerError::UnexpectedEndOfFile);
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
        assert_tokens(
            "<?php -> $",
            &[open!(), TokenKind::Arrow, TokenKind::Dollar],
        );
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

        assert_eq!(spans, &[(1, 1), (1, 7), (1, 18), (1, 19),]);

        let spans = get_spans(
            r#"<?php

function hello_world() {

}"#,
        );

        assert_eq!(
            spans,
            &[(1, 1), (3, 1), (3, 10), (3, 21), (3, 22), (3, 24), (5, 1),]
        );
    }

    #[test]
    fn floats() {
        assert_tokens(
            "<?php 200.5 .05",
            &[open!(), TokenKind::Float(200.5), TokenKind::Float(0.05)],
        );
    }

    #[test]
    fn fully_qualified_ident() {
        assert_tokens(
            "<?php \\Exception \\Foo\\Bar",
            &[
                open!(),
                TokenKind::FullyQualifiedIdentifier(b"\\Exception".into()),
                TokenKind::FullyQualifiedIdentifier(b"\\Foo\\Bar".into()),
            ],
        );
    }

    #[test]
    fn halt_compiler() {
        assert_tokens(
            "<?php __halt_compiler();",
            &[open!(), TokenKind::HaltCompiler],
        );

        assert_tokens(
            "<?php __HALT_COMPILER();",
            &[open!(), TokenKind::HaltCompiler],
        );

        assert_tokens(
            "<?php __halt_compiler(); Some jargon that comes after the halt, oops!",
            &[
                open!(),
                TokenKind::HaltCompiler,
                TokenKind::InlineHtml(
                    " Some jargon that comes after the halt, oops!"
                        .as_bytes()
                        .into(),
                ),
            ],
        );
    }

    fn assert_error<B: ?Sized + AsRef<[u8]>>(source: &B, expected: LexerError) {
        let mut lexer = Lexer::new(None);
        assert_eq!(lexer.tokenize(source), Err(expected));
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
