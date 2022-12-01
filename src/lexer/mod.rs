pub mod byte_string;
pub mod error;
pub mod token;

mod macros;
mod state;

use std::num::IntErrorKind;

use crate::lexer::byte_string::ByteString;
use crate::lexer::error::SyntaxError;
use crate::lexer::state::StackState;
use crate::lexer::state::State;
use crate::lexer::token::OpenTagKind;
use crate::lexer::token::Span;
use crate::lexer::token::Token;
use crate::lexer::token::TokenKind;

use crate::ident;
use crate::ident_start;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Lexer;

impl Lexer {
    pub const fn new() -> Self {
        Self {}
    }

    pub fn tokenize<B: ?Sized + AsRef<[u8]>>(&self, input: &B) -> Result<Vec<Token>, SyntaxError> {
        let mut state = State::new(input);
        let mut tokens = Vec::new();

        while state.current.is_some() {
            match state.stack.last().unwrap() {
                // The "Initial" state is used to parse inline HTML. It is essentially a catch-all
                // state that will build up a single token buffer until it encounters an open tag
                // of some description.
                StackState::Initial => {
                    tokens.append(&mut self.initial(&mut state)?);
                }
                // The scripting state is entered when an open tag is encountered in the source code.
                // This tells the lexer to start analysing characters at PHP tokens instead of inline HTML.
                StackState::Scripting => {
                    self.skip_whitespace(&mut state);

                    // If we have consumed whitespace and then reached the end of the file, we should break.
                    if state.current.is_none() {
                        break;
                    }

                    tokens.push(self.scripting(&mut state)?);
                }
                // The "Halted" state is entered when the `__halt_compiler` token is encountered.
                // In this state, all the text that follows is no longer parsed as PHP as is collected
                // into a single "InlineHtml" token (kind of cheating, oh well).
                StackState::Halted => {
                    tokens.push(Token {
                        kind: TokenKind::InlineHtml(state.chars[state.cursor..].into()),
                        span: state.span,
                    });
                    break;
                }
                // The double quote state is entered when inside a double-quoted string that
                // contains variables.
                StackState::DoubleQuote => tokens.extend(self.double_quote(&mut state)?),
                // LookingForProperty is entered inside double quotes,
                // backticks, or a heredoc, expecting a variable name.
                // If one isn't found, it switches to scripting.
                StackState::LookingForVarname => {
                    if let Some(token) = self.looking_for_varname(&mut state) {
                        tokens.push(token);
                    }
                }
                // LookingForProperty is entered inside double quotes,
                // backticks, or a heredoc, expecting an arrow followed by a
                // property name.
                StackState::LookingForProperty => {
                    tokens.push(self.looking_for_property(&mut state)?);
                }
                StackState::VarOffset => {
                    if state.current.is_none() {
                        break;
                    }

                    tokens.push(self.var_offset(&mut state)?);
                }
            }
        }

        Ok(tokens)
    }

    fn skip_whitespace(&self, state: &mut State) {
        while let Some(b' ' | b'\n' | b'\r' | b'\t') = state.current {
            state.next();
        }
    }

    fn initial(&self, state: &mut State) -> Result<Vec<Token>, SyntaxError> {
        let inline_span = state.span;
        let mut buffer = Vec::new();
        while let Some(char) = state.current {
            if state.try_read(b"<?php") {
                let tag_span = state.span;
                state.skip(5);

                state.enter_state(StackState::Scripting);

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

            state.next();
            buffer.push(char);
        }

        Ok(vec![Token {
            kind: TokenKind::InlineHtml(buffer.into()),
            span: inline_span,
        }])
    }

    fn scripting(&self, state: &mut State) -> Result<Token, SyntaxError> {
        let span = state.span;
        let kind = match state.peek_buf() {
            [b'@', ..] => {
                state.next();

                TokenKind::At
            }
            [b'!', b'=', b'=', ..] => {
                state.skip(3);
                TokenKind::BangDoubleEquals
            }
            [b'!', b'=', ..] => {
                state.skip(2);
                TokenKind::BangEquals
            }
            [b'!', ..] => {
                state.next();
                TokenKind::Bang
            }
            [b'&', b'&', ..] => {
                state.skip(2);
                TokenKind::BooleanAnd
            }
            [b'&', b'=', ..] => {
                state.skip(2);
                TokenKind::AmpersandEquals
            }
            [b'&', ..] => {
                state.next();
                TokenKind::Ampersand
            }
            [b'?', b'>', ..] => {
                // This is a close tag, we can enter "Initial" mode again.
                state.skip(2);

                state.enter_state(StackState::Initial);

                TokenKind::CloseTag
            }
            [b'?', b'?', b'=', ..] => {
                state.skip(3);
                TokenKind::CoalesceEqual
            }
            [b'?', b'?', ..] => {
                state.skip(2);
                TokenKind::Coalesce
            }
            [b'?', b':', ..] => {
                state.skip(2);
                TokenKind::QuestionColon
            }
            [b'?', b'-', b'>', ..] => {
                state.skip(3);
                TokenKind::NullsafeArrow
            }
            [b'?', ..] => {
                state.next();
                TokenKind::Question
            }
            [b'=', b'>', ..] => {
                state.skip(2);
                TokenKind::DoubleArrow
            }
            [b'=', b'=', b'=', ..] => {
                state.skip(3);
                TokenKind::TripleEquals
            }
            [b'=', b'=', ..] => {
                state.skip(2);
                TokenKind::DoubleEquals
            }
            [b'=', ..] => {
                state.next();
                TokenKind::Equals
            }
            // Single quoted string.
            [b'\'', ..] => {
                state.next();
                self.tokenize_single_quote_string(state)?
            }
            [b'b' | b'B', b'\'', ..] => {
                state.skip(2);
                self.tokenize_single_quote_string(state)?
            }
            [b'"', ..] => {
                state.next();
                self.tokenize_double_quote_string(state)?
            }
            [b'b' | b'B', b'"', ..] => {
                state.skip(2);
                self.tokenize_double_quote_string(state)?
            }
            [b'$', ident_start!(), ..] => {
                state.next();
                self.tokenize_variable(state)
            }
            [b'$', ..] => {
                state.next();
                TokenKind::Dollar
            }
            [b'.', b'=', ..] => {
                state.skip(2);
                TokenKind::DotEquals
            }
            [b'.', b'0'..=b'9', ..] => self.tokenize_number(state)?,
            [b'.', b'.', b'.', ..] => {
                state.skip(3);
                TokenKind::Ellipsis
            }
            [b'.', ..] => {
                state.next();
                TokenKind::Dot
            }
            &[b'0'..=b'9', ..] => self.tokenize_number(state)?,
            &[b'\\', ident_start!(), ..] => {
                state.next();

                match self.scripting(state)? {
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
                state.next();
                TokenKind::NamespaceSeparator
            }
            &[b @ ident_start!(), ..] => {
                state.next();
                let mut qualified = false;
                let mut last_was_slash = false;

                let mut buffer = vec![b];
                while let Some(next) = state.current {
                    if next.is_ascii_alphanumeric() || next == b'_' {
                        buffer.push(next);
                        state.next();
                        last_was_slash = false;
                        continue;
                    }

                    if next == b'\\' && !last_was_slash {
                        qualified = true;
                        last_was_slash = true;
                        buffer.push(next);
                        state.next();
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
                        match state.peek_buf() {
                            [b'(', b')', b';', ..] => {
                                state.skip(3);
                                state.enter_state(StackState::Halted);
                            }
                            _ => return Err(SyntaxError::InvalidHaltCompiler(state.span)),
                        }
                    }

                    kind
                }
            }
            [b'/', b'*', ..] => {
                state.next();
                let mut buffer = vec![b'/'];

                while state.current.is_some() {
                    match state.peek_buf() {
                        [b'*', b'/', ..] => {
                            state.skip(2);
                            buffer.extend_from_slice(b"*/");
                            break;
                        }
                        &[t, ..] => {
                            state.next();
                            buffer.push(t);
                        }
                        [] => unreachable!(),
                    }
                }
                state.next();

                if buffer.starts_with(b"/**") {
                    TokenKind::DocComment(buffer.into())
                } else {
                    TokenKind::Comment(buffer.into())
                }
            }
            [b'#', b'[', ..] => {
                state.skip(2);
                TokenKind::Attribute
            }
            &[ch @ b'/', b'/', ..] | &[ch @ b'#', ..] => {
                let mut buffer = if ch == b'/' {
                    state.skip(2);
                    b"//".to_vec()
                } else {
                    state.next();
                    b"#".to_vec()
                };

                while let Some(c) = state.current {
                    if c == b'\n' {
                        break;
                    }

                    buffer.push(c);
                    state.next();
                }

                state.next();

                TokenKind::Comment(buffer.into())
            }
            [b'/', b'=', ..] => {
                state.skip(2);
                TokenKind::SlashEquals
            }
            [b'/', ..] => {
                state.next();
                TokenKind::Slash
            }
            [b'*', b'*', b'=', ..] => {
                state.skip(3);
                TokenKind::PowEquals
            }
            [b'*', b'*', ..] => {
                state.skip(2);
                TokenKind::Pow
            }
            [b'*', b'=', ..] => {
                state.skip(2);
                TokenKind::AsteriskEqual
            }
            [b'*', ..] => {
                state.next();
                TokenKind::Asterisk
            }
            [b'|', b'|', ..] => {
                state.skip(2);
                TokenKind::Pipe
            }
            [b'|', b'=', ..] => {
                state.skip(2);
                TokenKind::PipeEquals
            }
            [b'|', ..] => {
                state.next();
                TokenKind::Pipe
            }
            [b'^', b'=', ..] => {
                state.skip(2);
                TokenKind::CaretEquals
            }
            [b'^', ..] => {
                state.next();
                TokenKind::Caret
            }
            [b'{', ..] => {
                state.next();
                state.push_state(StackState::Scripting);
                TokenKind::LeftBrace
            }
            [b'}', ..] => {
                state.next();
                state.pop_state();
                TokenKind::RightBrace
            }
            [b'(', ..] => {
                state.next();

                if state.try_read(b"int)") {
                    state.skip(4);
                    TokenKind::IntCast
                } else if state.try_read(b"integer)") {
                    state.skip(8);
                    TokenKind::IntegerCast
                } else if state.try_read(b"bool)") {
                    state.skip(5);
                    TokenKind::BoolCast
                } else if state.try_read(b"boolean)") {
                    state.skip(8);
                    TokenKind::BooleanCast
                } else if state.try_read(b"float)") {
                    state.skip(6);
                    TokenKind::FloatCast
                } else if state.try_read(b"double)") {
                    state.skip(7);
                    TokenKind::DoubleCast
                } else if state.try_read(b"real)") {
                    state.skip(5);
                    TokenKind::RealCast
                } else if state.try_read(b"string)") {
                    state.skip(7);
                    TokenKind::StringCast
                } else if state.try_read(b"binary)") {
                    state.skip(7);
                    TokenKind::BinaryCast
                } else if state.try_read(b"array)") {
                    state.skip(6);
                    TokenKind::ArrayCast
                } else if state.try_read(b"object)") {
                    state.skip(7);
                    TokenKind::ObjectCast
                } else if state.try_read(b"unset)") {
                    state.skip(6);
                    TokenKind::UnsetCast
                } else {
                    TokenKind::LeftParen
                }
            }
            [b')', ..] => {
                state.next();
                TokenKind::RightParen
            }
            [b';', ..] => {
                state.next();
                TokenKind::SemiColon
            }
            [b'+', b'+', ..] => {
                state.skip(2);
                TokenKind::Increment
            }
            [b'+', b'=', ..] => {
                state.skip(2);
                TokenKind::PlusEquals
            }
            [b'+', ..] => {
                state.next();
                TokenKind::Plus
            }
            [b'%', b'=', ..] => {
                state.skip(2);
                TokenKind::PercentEquals
            }
            [b'%', ..] => {
                state.next();
                TokenKind::Percent
            }
            [b'-', b'-', ..] => {
                state.skip(2);
                TokenKind::Decrement
            }
            [b'-', b'>', ..] => {
                state.skip(2);
                TokenKind::Arrow
            }
            [b'-', b'=', ..] => {
                state.skip(2);
                TokenKind::MinusEquals
            }
            [b'-', ..] => {
                state.next();
                TokenKind::Minus
            }
            [b'<', b'<', b'<', ..] => {
                // TODO: Handle both heredocs and nowdocs.
                state.skip(3);

                todo!("heredocs & nowdocs");
            }
            [b'<', b'<', b'=', ..] => {
                state.skip(3);

                TokenKind::LeftShiftEquals
            }
            [b'<', b'<', ..] => {
                state.skip(2);
                TokenKind::LeftShift
            }
            [b'<', b'=', b'>', ..] => {
                state.skip(3);
                TokenKind::Spaceship
            }
            [b'<', b'=', ..] => {
                state.skip(2);
                TokenKind::LessThanEquals
            }
            [b'<', b'>', ..] => {
                state.skip(2);
                TokenKind::AngledLeftRight
            }
            [b'<', ..] => {
                state.next();
                TokenKind::LessThan
            }
            [b'>', b'>', b'=', ..] => {
                state.skip(3);
                TokenKind::RightShiftEquals
            }
            [b'>', b'>', ..] => {
                state.skip(2);
                TokenKind::RightShift
            }
            [b'>', b'=', ..] => {
                state.skip(2);
                TokenKind::GreaterThanEquals
            }
            [b'>', ..] => {
                state.next();
                TokenKind::GreaterThan
            }
            [b',', ..] => {
                state.next();
                TokenKind::Comma
            }
            [b'[', ..] => {
                state.next();
                TokenKind::LeftBracket
            }
            [b']', ..] => {
                state.next();
                TokenKind::RightBracket
            }
            [b':', b':', ..] => {
                state.skip(2);
                TokenKind::DoubleColon
            }
            [b':', ..] => {
                state.next();
                TokenKind::Colon
            }
            &[b'~', ..] => {
                state.next();
                TokenKind::BitwiseNot
            }
            &[b, ..] => unimplemented!(
                "<scripting> char: {}, line: {}, col: {}",
                b as char,
                state.span.0,
                state.span.1
            ),
            // We should never reach this point since we have the empty checks surrounding
            // the call to this function, but it's better to be safe than sorry.
            [] => return Err(SyntaxError::UnexpectedEndOfFile(state.span)),
        };

        Ok(Token { kind, span })
    }

    fn double_quote(&self, state: &mut State) -> Result<Vec<Token>, SyntaxError> {
        let span = state.span;
        let mut buffer = Vec::new();
        let kind = loop {
            match state.peek_buf() {
                [b'$', b'{', ..] => {
                    state.skip(2);
                    state.push_state(StackState::LookingForVarname);
                    break TokenKind::DollarLeftBrace;
                }
                [b'{', b'$', ..] => {
                    // Intentionally only consume the left brace.
                    state.next();
                    state.push_state(StackState::Scripting);
                    break TokenKind::LeftBrace;
                }
                [b'"', ..] => {
                    state.next();
                    state.enter_state(StackState::Scripting);
                    break TokenKind::DoubleQuote;
                }
                [b'$', ident_start!(), ..] => {
                    state.next();
                    let ident = self.consume_identifier(state);

                    match state.peek_buf() {
                        [b'[', ..] => state.push_state(StackState::VarOffset),
                        [b'-', b'>', ident_start!(), ..]
                        | [b'?', b'-', b'>', ident_start!(), ..] => {
                            state.push_state(StackState::LookingForProperty)
                        }
                        _ => {}
                    }

                    break TokenKind::Variable(ident.into());
                }
                &[b, ..] => {
                    state.next();
                    buffer.push(b);
                }
                [] => return Err(SyntaxError::UnexpectedEndOfFile(state.span)),
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

    fn looking_for_varname(&self, state: &mut State) -> Option<Token> {
        let identifier = self.peek_identifier(state);

        if let Some(ident) = identifier {
            if let Some(b'[' | b'}') = state.peek_byte(ident.len()) {
                let ident = ident.to_vec();
                let span = state.span;
                state.skip(ident.len());
                state.enter_state(StackState::Scripting);
                return Some(Token {
                    kind: TokenKind::Identifier(ident.into()),
                    span,
                });
            }
        }

        state.enter_state(StackState::Scripting);
        None
    }

    fn looking_for_property(&self, state: &mut State) -> Result<Token, SyntaxError> {
        let span = state.span;
        let kind = match state.peek_buf() {
            [b'-', b'>', ..] => {
                state.skip(2);
                TokenKind::Arrow
            }
            [b'?', b'-', b'>', ..] => {
                state.skip(3);
                TokenKind::NullsafeArrow
            }
            &[ident_start!(), ..] => {
                let buffer = self.consume_identifier(state);
                state.pop_state();
                TokenKind::Identifier(buffer.into())
            }
            // Should be impossible as we already looked ahead this far inside double_quote.
            _ => unreachable!(),
        };
        Ok(Token { kind, span })
    }

    fn var_offset(&self, state: &mut State) -> Result<Token, SyntaxError> {
        let span = state.span;
        let kind = match state.peek_buf() {
            [b'$', ident_start!(), ..] => {
                state.next();
                self.tokenize_variable(state)
            }
            &[b'0'..=b'9', ..] => {
                // TODO: all integer literals are allowed, but only decimal integers with no underscores
                // are actually treated as numbers. Others are treated as strings.
                // Float literals are not allowed, but that could be handled in the parser.
                self.tokenize_number(state)?
            }
            [b'[', ..] => {
                state.next();
                TokenKind::LeftBracket
            }
            [b'-', ..] => {
                state.next();
                TokenKind::Minus
            }
            [b']', ..] => {
                state.next();
                state.pop_state();
                TokenKind::RightBracket
            }
            &[ident_start!(), ..] => {
                let label = self.consume_identifier(state);
                TokenKind::Identifier(label.into())
            }
            &[b, ..] => unimplemented!(
                "<var offset> char: {}, line: {}, col: {}",
                b as char,
                state.span.0,
                state.span.1
            ),
            [] => return Err(SyntaxError::UnexpectedEndOfFile(state.span)),
        };
        Ok(Token { kind, span })
    }

    fn tokenize_single_quote_string(&self, state: &mut State) -> Result<TokenKind, SyntaxError> {
        let mut buffer = Vec::new();

        loop {
            match state.peek_buf() {
                [b'\'', ..] => {
                    state.next();
                    break;
                }
                &[b'\\', b @ b'\'' | b @ b'\\', ..] => {
                    state.skip(2);
                    buffer.push(b);
                }
                &[b, ..] => {
                    state.next();
                    buffer.push(b);
                }
                [] => return Err(SyntaxError::UnexpectedEndOfFile(state.span)),
            }
        }

        Ok(TokenKind::LiteralString(buffer.into()))
    }

    fn tokenize_double_quote_string(&self, state: &mut State) -> Result<TokenKind, SyntaxError> {
        let mut buffer = Vec::new();

        let constant = loop {
            match state.peek_buf() {
                [b'"', ..] => {
                    state.next();
                    break true;
                }
                &[b'\\', b @ (b'"' | b'\\' | b'$'), ..] => {
                    state.skip(2);
                    buffer.push(b);
                }
                &[b'\\', b'n', ..] => {
                    state.skip(2);
                    buffer.push(b'\n');
                }
                &[b'\\', b'r', ..] => {
                    state.skip(2);
                    buffer.push(b'\r');
                }
                &[b'\\', b't', ..] => {
                    state.skip(2);
                    buffer.push(b'\t');
                }
                &[b'\\', b'v', ..] => {
                    state.skip(2);
                    buffer.push(b'\x0b');
                }
                &[b'\\', b'e', ..] => {
                    state.skip(2);
                    buffer.push(b'\x1b');
                }
                &[b'\\', b'f', ..] => {
                    state.skip(2);
                    buffer.push(b'\x0c');
                }
                &[b'\\', b'x', b @ (b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F'), ..] => {
                    state.skip(3);

                    let mut hex = String::from(b as char);
                    if let Some(b @ (b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F')) = state.current {
                        state.next();
                        hex.push(b as char);
                    }

                    let b = u8::from_str_radix(&hex, 16).unwrap();
                    buffer.push(b);
                }
                &[b'\\', b'u', b'{', ..] => {
                    state.skip(3);

                    let mut code_point = String::new();
                    while let Some(b @ (b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F')) = state.current {
                        state.next();
                        code_point.push(b as char);
                    }

                    if code_point.is_empty() || state.current != Some(b'}') {
                        return Err(SyntaxError::InvalidUnicodeEscape(state.span));
                    }
                    state.next();

                    let c = if let Ok(c) = u32::from_str_radix(&code_point, 16) {
                        c
                    } else {
                        return Err(SyntaxError::InvalidUnicodeEscape(state.span));
                    };

                    if let Some(c) = char::from_u32(c) {
                        let mut tmp = [0; 4];
                        let bytes = c.encode_utf8(&mut tmp);
                        buffer.extend(bytes.as_bytes());
                    } else {
                        return Err(SyntaxError::InvalidUnicodeEscape(state.span));
                    }
                }
                &[b'\\', b @ b'0'..=b'7', ..] => {
                    state.skip(2);

                    let mut octal = String::from(b as char);
                    if let Some(b @ b'0'..=b'7') = state.current {
                        state.next();
                        octal.push(b as char);
                    }
                    if let Some(b @ b'0'..=b'7') = state.current {
                        state.next();
                        octal.push(b as char);
                    }

                    if let Ok(b) = u8::from_str_radix(&octal, 8) {
                        buffer.push(b);
                    } else {
                        return Err(SyntaxError::InvalidOctalEscape(state.span));
                    }
                }
                [b'$', ident_start!(), ..] | [b'{', b'$', ..] | [b'$', b'{', ..] => {
                    break false;
                }
                &[b, ..] => {
                    state.next();
                    buffer.push(b);
                }
                [] => return Err(SyntaxError::UnexpectedEndOfFile(state.span)),
            }
        };

        Ok(if constant {
            TokenKind::LiteralString(buffer.into())
        } else {
            state.enter_state(StackState::DoubleQuote);
            TokenKind::StringPart(buffer.into())
        })
    }

    fn peek_identifier<'a>(&'a self, state: &'a State) -> Option<&[u8]> {
        let mut cursor = state.cursor;
        if let Some(ident_start!()) = state.chars.get(cursor) {
            cursor += 1;
            while let Some(ident!()) = state.chars.get(cursor) {
                cursor += 1;
            }
            Some(&state.chars[state.cursor..cursor])
        } else {
            None
        }
    }

    fn consume_identifier(&self, state: &mut State) -> Vec<u8> {
        let ident = self.peek_identifier(state).unwrap().to_vec();
        state.skip(ident.len());

        ident
    }

    fn tokenize_variable(&self, state: &mut State) -> TokenKind {
        TokenKind::Variable(self.consume_identifier(state).into())
    }

    fn tokenize_number(&self, state: &mut State) -> Result<TokenKind, SyntaxError> {
        let mut buffer = String::new();

        let (base, kind) = match state.peek_buf() {
            [b'0', b'B' | b'b', ..] => {
                state.skip(2);
                (2, NumberKind::Int)
            }
            [b'0', b'O' | b'o', ..] => {
                state.skip(2);
                (8, NumberKind::Int)
            }
            [b'0', b'X' | b'x', ..] => {
                state.skip(2);
                (16, NumberKind::Int)
            }
            [b'0', ..] => (10, NumberKind::OctalOrFloat),
            [b'.', ..] => (10, NumberKind::Float),
            _ => (10, NumberKind::IntOrFloat),
        };

        if kind != NumberKind::Float {
            self.read_digits(state, &mut buffer, base);
            if kind == NumberKind::Int {
                return parse_int(&buffer, base as u32, state.span);
            }
        }

        // Remaining cases: decimal integer, legacy octal integer, or float.
        let is_float = matches!(
            state.peek_buf(),
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
            return parse_int(&buffer, base as u32, state.span);
        }

        if state.current == Some(b'.') {
            buffer.push('.');
            state.next();
            self.read_digits(state, &mut buffer, 10);
        }

        if let Some(b'e' | b'E') = state.current {
            buffer.push('e');
            state.next();
            if let Some(b @ (b'-' | b'+')) = state.current {
                buffer.push(b as char);
                state.next();
            }
            self.read_digits(state, &mut buffer, 10);
        }

        Ok(TokenKind::LiteralFloat(buffer.parse().unwrap()))
    }

    fn read_digits(&self, state: &mut State, buffer: &mut String, base: usize) {
        if base == 16 {
            self.read_digits_fn(state, buffer, u8::is_ascii_hexdigit);
        } else {
            let max = b'0' + base as u8;
            self.read_digits_fn(state, buffer, |b| (b'0'..max).contains(b));
        };
    }

    fn read_digits_fn<F: Fn(&u8) -> bool>(
        &self,
        state: &mut State,
        buffer: &mut String,
        is_digit: F,
    ) {
        if let Some(b) = state.current {
            if is_digit(&b) {
                state.next();
                buffer.push(b as char);
            } else {
                return;
            }
        }
        loop {
            match *state.peek_buf() {
                [b, ..] if is_digit(&b) => {
                    state.next();
                    buffer.push(b as char);
                }
                [b'_', b, ..] if is_digit(&b) => {
                    state.next();
                    state.next();
                    buffer.push(b as char);
                }
                _ => {
                    break;
                }
            }
        }
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
