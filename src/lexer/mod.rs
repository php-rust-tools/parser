pub mod byte_string;
pub mod error;
mod macros;
pub mod token;

use std::num::IntErrorKind;

use crate::lexer::byte_string::ByteString;
use crate::lexer::error::SyntaxError;
use crate::lexer::token::OpenTagKind;
use crate::lexer::token::Span;
use crate::lexer::token::Token;
use crate::lexer::token::TokenKind;

use crate::ident;
use crate::ident_start;

#[derive(Debug, PartialEq, Eq)]
pub enum LexerState {
    Initial,
    Scripting,
    Halted,
    DoubleQuote,
    LookingForVarname,
    LookingForProperty,
    VarOffset,
}

pub struct Lexer {
    state_stack: Vec<LexerState>,
    chars: Vec<u8>,
    cursor: usize,
    current: Option<u8>,
    span: Span,
}

impl Default for Lexer {
    fn default() -> Self {
        Self::new()
    }
}

impl Lexer {
    pub fn new() -> Self {
        Self {
            state_stack: vec![LexerState::Initial],
            chars: Vec::new(),
            cursor: 0,
            current: None,
            span: (1, 1),
        }
    }

    pub fn tokenize<B: ?Sized + AsRef<[u8]>>(
        &mut self,
        input: &B,
    ) -> Result<Vec<Token>, SyntaxError> {
        let mut tokens = Vec::new();
        self.chars = input.as_ref().to_vec();

        self.current = self.chars.first().copied();

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
                    self.skip_whitespace();

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
                        span: self.span,
                    });
                    break;
                }
                // The double quote state is entered when inside a double-quoted string that
                // contains variables.
                LexerState::DoubleQuote => tokens.extend(self.double_quote()?),
                // LookingForProperty is entered inside double quotes,
                // backticks, or a heredoc, expecting a variable name.
                // If one isn't found, it switches to scripting.
                LexerState::LookingForVarname => {
                    if let Some(token) = self.looking_for_varname() {
                        tokens.push(token);
                    }
                }
                // LookingForProperty is entered inside double quotes,
                // backticks, or a heredoc, expecting an arrow followed by a
                // property name.
                LexerState::LookingForProperty => {
                    tokens.push(self.looking_for_property()?);
                }
                LexerState::VarOffset => {
                    if self.current.is_none() {
                        break;
                    }

                    tokens.push(self.var_offset()?);
                }
            }
        }

        Ok(tokens)
    }

    fn skip_whitespace(&mut self) {
        while let Some(b' ' | b'\n' | b'\r' | b'\t') = self.current {
            self.next();
        }
    }

    fn initial(&mut self) -> Result<Vec<Token>, SyntaxError> {
        let inline_span = self.span;
        let mut buffer = Vec::new();
        while let Some(char) = self.current {
            if self.try_read(b"<?php") {
                let tag_span = self.span;
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

    fn scripting(&mut self) -> Result<Token, SyntaxError> {
        let span = self.span;
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
                TokenKind::Bang
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
            [b'$', ident_start!(), ..] => {
                self.next();
                self.tokenize_variable()
            }
            [b'$', ..] => {
                self.next();
                TokenKind::Dollar
            }
            [b'.', b'=', ..] => {
                self.skip(2);
                TokenKind::DotEquals
            }
            [b'.', b'0'..=b'9', ..] => self.tokenize_number()?,
            [b'.', b'.', b'.', ..] => {
                self.skip(3);
                TokenKind::Ellipsis
            }
            [b'.', ..] => {
                self.next();
                TokenKind::Dot
            }
            &[b'0'..=b'9', ..] => self.tokenize_number()?,
            &[b'\\', ident_start!(), ..] => {
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
            &[b @ ident_start!(), ..] => {
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
                                self.enter_state(LexerState::Halted);
                            }
                            _ => return Err(SyntaxError::InvalidHaltCompiler(self.span)),
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
                self.push_state(LexerState::Scripting);
                TokenKind::LeftBrace
            }
            [b'}', ..] => {
                self.next();
                self.pop_state();
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
                self.span.0,
                self.span.1
            ),
            // We should never reach this point since we have the empty checks surrounding
            // the call to this function, but it's better to be safe than sorry.
            [] => return Err(SyntaxError::UnexpectedEndOfFile(self.span)),
        };

        Ok(Token { kind, span })
    }

    fn double_quote(&mut self) -> Result<Vec<Token>, SyntaxError> {
        let span = self.span;
        let mut buffer = Vec::new();
        let kind = loop {
            match self.peek_buf() {
                [b'$', b'{', ..] => {
                    self.skip(2);
                    self.push_state(LexerState::LookingForVarname);
                    break TokenKind::DollarLeftBrace;
                }
                [b'{', b'$', ..] => {
                    // Intentionally only consume the left brace.
                    self.next();
                    self.push_state(LexerState::Scripting);
                    break TokenKind::LeftBrace;
                }
                [b'"', ..] => {
                    self.next();
                    self.enter_state(LexerState::Scripting);
                    break TokenKind::DoubleQuote;
                }
                [b'$', ident_start!(), ..] => {
                    self.next();
                    let ident = self.consume_identifier();

                    match self.peek_buf() {
                        [b'[', ..] => self.push_state(LexerState::VarOffset),
                        [b'-', b'>', ident_start!(), ..]
                        | [b'?', b'-', b'>', ident_start!(), ..] => {
                            self.push_state(LexerState::LookingForProperty)
                        }
                        _ => {}
                    }

                    break TokenKind::Variable(ident.into());
                }
                &[b, ..] => {
                    self.next();
                    buffer.push(b);
                }
                [] => return Err(SyntaxError::UnexpectedEndOfFile(self.span)),
            }
        };

        let mut tokens = Vec::new();
        if !buffer.is_empty() {
            tokens.push(Token {
                kind: TokenKind::StringPart(buffer.into()),
                span,
            })
        }

        tokens.push(Token { kind, span });
        Ok(tokens)
    }

    fn looking_for_varname(&mut self) -> Option<Token> {
        if let Some(ident) = self.peek_identifier() {
            if let Some(b'[' | b'}') = self.peek_byte(ident.len()) {
                let ident = ident.to_vec();
                let span = self.span;
                self.skip(ident.len());
                self.enter_state(LexerState::Scripting);
                return Some(Token {
                    kind: TokenKind::Identifier(ident.into()),
                    span,
                });
            }
        }

        self.enter_state(LexerState::Scripting);
        None
    }

    fn looking_for_property(&mut self) -> Result<Token, SyntaxError> {
        let span = self.span;
        let kind = match self.peek_buf() {
            [b'-', b'>', ..] => {
                self.skip(2);
                TokenKind::Arrow
            }
            [b'?', b'-', b'>', ..] => {
                self.skip(3);
                TokenKind::NullsafeArrow
            }
            &[ident_start!(), ..] => {
                let buffer = self.consume_identifier();
                self.pop_state();
                TokenKind::Identifier(buffer.into())
            }
            // Should be impossible as we already looked ahead this far inside double_quote.
            _ => unreachable!(),
        };
        Ok(Token { kind, span })
    }

    fn var_offset(&mut self) -> Result<Token, SyntaxError> {
        let span = self.span;
        let kind = match self.peek_buf() {
            [b'$', ident_start!(), ..] => {
                self.next();
                self.tokenize_variable()
            }
            &[b'0'..=b'9', ..] => {
                // TODO: all integer literals are allowed, but only decimal integers with no underscores
                // are actually treated as numbers. Others are treated as strings.
                // Float literals are not allowed, but that could be handled in the parser.
                self.tokenize_number()?
            }
            [b'[', ..] => {
                self.next();
                TokenKind::LeftBracket
            }
            [b'-', ..] => {
                self.next();
                TokenKind::Minus
            }
            [b']', ..] => {
                self.next();
                self.pop_state();
                TokenKind::RightBracket
            }
            &[ident_start!(), ..] => {
                let label = self.consume_identifier();
                TokenKind::Identifier(label.into())
            }
            &[b, ..] => unimplemented!(
                "<var offset> char: {}, line: {}, col: {}",
                b as char,
                self.span.0,
                self.span.1
            ),
            [] => return Err(SyntaxError::UnexpectedEndOfFile(self.span)),
        };
        Ok(Token { kind, span })
    }

    fn tokenize_single_quote_string(&mut self) -> Result<TokenKind, SyntaxError> {
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
                [] => return Err(SyntaxError::UnexpectedEndOfFile(self.span)),
            }
        }

        Ok(TokenKind::LiteralString(buffer.into()))
    }

    fn tokenize_double_quote_string(&mut self) -> Result<TokenKind, SyntaxError> {
        let mut buffer = Vec::new();

        let constant = loop {
            match self.peek_buf() {
                [b'"', ..] => {
                    self.next();
                    break true;
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
                        return Err(SyntaxError::InvalidUnicodeEscape(self.span));
                    }
                    self.next();

                    let c = if let Ok(c) = u32::from_str_radix(&code_point, 16) {
                        c
                    } else {
                        return Err(SyntaxError::InvalidUnicodeEscape(self.span));
                    };

                    if let Some(c) = char::from_u32(c) {
                        let mut tmp = [0; 4];
                        let bytes = c.encode_utf8(&mut tmp);
                        buffer.extend(bytes.as_bytes());
                    } else {
                        return Err(SyntaxError::InvalidUnicodeEscape(self.span));
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
                        return Err(SyntaxError::InvalidOctalEscape(self.span));
                    }
                }
                [b'$', ident_start!(), ..] | [b'{', b'$', ..] | [b'$', b'{', ..] => {
                    break false;
                }
                &[b, ..] => {
                    self.next();
                    buffer.push(b);
                }
                [] => return Err(SyntaxError::UnexpectedEndOfFile(self.span)),
            }
        };

        Ok(if constant {
            TokenKind::LiteralString(buffer.into())
        } else {
            self.enter_state(LexerState::DoubleQuote);
            TokenKind::StringPart(buffer.into())
        })
    }

    fn peek_identifier(&self) -> Option<&[u8]> {
        let mut cursor = self.cursor;
        if let Some(ident_start!()) = self.chars.get(cursor) {
            cursor += 1;
            while let Some(ident!()) = self.chars.get(cursor) {
                cursor += 1;
            }
            Some(&self.chars[self.cursor..cursor])
        } else {
            None
        }
    }

    fn consume_identifier(&mut self) -> Vec<u8> {
        let ident = self.peek_identifier().unwrap().to_vec();
        self.skip(ident.len());

        ident
    }

    fn tokenize_variable(&mut self) -> TokenKind {
        TokenKind::Variable(self.consume_identifier().into())
    }

    fn tokenize_number(&mut self) -> Result<TokenKind, SyntaxError> {
        let mut buffer = String::new();

        let (base, kind) = match self.peek_buf() {
            [b'0', b'B' | b'b', ..] => {
                self.skip(2);
                (2, NumberKind::Int)
            }
            [b'0', b'O' | b'o', ..] => {
                self.skip(2);
                (8, NumberKind::Int)
            }
            [b'0', b'X' | b'x', ..] => {
                self.skip(2);
                (16, NumberKind::Int)
            }
            [b'0', ..] => (10, NumberKind::OctalOrFloat),
            [b'.', ..] => (10, NumberKind::Float),
            _ => (10, NumberKind::IntOrFloat),
        };

        if kind != NumberKind::Float {
            self.read_digits(&mut buffer, base);
            if kind == NumberKind::Int {
                return parse_int(&buffer, base as u32, self.span);
            }
        }

        // Remaining cases: decimal integer, legacy octal integer, or float.
        let is_float = matches!(
            self.peek_buf(),
            [b'.', ..]
                | [b'e' | b'E', b'-' | b'+', b'0'..=b'9', ..]
                | [b'e' | b'E', b'0'..=b'9', ..]
        );
        if !is_float {
            let base = if kind == NumberKind::OctalOrFloat {
                8
            } else {
                10
            };
            return parse_int(&buffer, base as u32, self.span);
        }

        if self.current == Some(b'.') {
            buffer.push('.');
            self.next();
            self.read_digits(&mut buffer, 10);
        }

        if let Some(b'e' | b'E') = self.current {
            buffer.push('e');
            self.next();
            if let Some(b @ (b'-' | b'+')) = self.current {
                buffer.push(b as char);
                self.next();
            }
            self.read_digits(&mut buffer, 10);
        }

        Ok(TokenKind::LiteralFloat(buffer.parse().unwrap()))
    }

    fn read_digits(&mut self, buffer: &mut String, base: usize) {
        if base == 16 {
            self.read_digits_fn(buffer, u8::is_ascii_hexdigit);
        } else {
            let max = b'0' + base as u8;
            self.read_digits_fn(buffer, |b| (b'0'..max).contains(b));
        };
    }

    fn read_digits_fn<F: Fn(&u8) -> bool>(&mut self, buffer: &mut String, is_digit: F) {
        if let Some(b) = self.current {
            if is_digit(&b) {
                self.next();
                buffer.push(b as char);
            } else {
                return;
            }
        }
        loop {
            match *self.peek_buf() {
                [b, ..] if is_digit(&b) => {
                    self.next();
                    buffer.push(b as char);
                }
                [b'_', b, ..] if is_digit(&b) => {
                    self.next();
                    self.next();
                    buffer.push(b as char);
                }
                _ => {
                    break;
                }
            }
        }
    }

    fn enter_state(&mut self, state: LexerState) {
        *self.state_stack.last_mut().unwrap() = state;
    }

    fn push_state(&mut self, state: LexerState) {
        self.state_stack.push(state);
    }

    fn pop_state(&mut self) {
        self.state_stack.pop();
    }

    fn peek_buf(&self) -> &[u8] {
        &self.chars[self.cursor..]
    }

    fn peek_byte(&self, delta: usize) -> Option<u8> {
        self.chars.get(self.cursor + delta).copied()
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
                self.span.0 += 1;
                self.span.1 = 1;
            }
            Some(_) => self.span.1 += 1,
            _ => {}
        }
        self.cursor += 1;
        self.current = self.chars.get(self.cursor).copied();
    }
}

// Parses an integer literal in the given base and converts errors to SyntaxError.
// It returns a float token instead on overflow.
fn parse_int(buffer: &str, base: u32, span: Span) -> Result<TokenKind, SyntaxError> {
    match i64::from_str_radix(buffer, base) {
        Ok(i) => Ok(TokenKind::LiteralInteger(i)),
        Err(err) if err.kind() == &IntErrorKind::InvalidDigit => {
            // The InvalidDigit error is only possible for legacy octal literals.
            Err(SyntaxError::InvalidOctalLiteral(span))
        }
        Err(err) if err.kind() == &IntErrorKind::PosOverflow => {
            // Parse as i128 so we can handle other bases.
            // This means there's an upper limit on how large the literal can be.
            let i = i128::from_str_radix(buffer, base).unwrap();
            Ok(TokenKind::LiteralFloat(i as f64))
        }
        _ => Err(SyntaxError::UnexpectedError(span)),
    }
}

fn identifier_to_keyword(ident: &[u8]) -> Option<TokenKind> {
    Some(match ident {
        b"enddeclare" => TokenKind::EndDeclare,
        b"endswitch" => TokenKind::EndSwitch,
        b"endfor" => TokenKind::EndFor,
        b"endwhile" => TokenKind::EndWhile,
        b"endforeach" => TokenKind::EndForeach,
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
        b"insteadof" => TokenKind::Insteadof,
        _ => return None,
    })
}

#[derive(Debug, Eq, PartialEq)]
enum NumberKind {
    Int,
    Float,
    IntOrFloat,
    OctalOrFloat,
}
