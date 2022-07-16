use std::{iter::Peekable, str::Chars};

use crate::{Token, TokenKind, OpenTagKind};

#[derive(Debug)]
pub enum LexerState {
    Initial,
    Scripting,
}

pub struct LexerConfig {
    short_tags: bool,
}

impl Default for LexerConfig {
    fn default() -> Self {
        Self { short_tags: false }
    }
}

pub struct Lexer {
    config: LexerConfig,
    state: LexerState,
}

impl Lexer {
    pub fn new(config: Option<LexerConfig>) -> Self {
        Self {
            config: config.unwrap_or(LexerConfig::default()),
            state: LexerState::Initial,
        }
    }

    pub fn tokenize(&mut self, input: &str) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();
        let mut it = input.chars().peekable();

        while let Some(_) = it.peek() {
            match self.state {
                // The "Initial" state is used to parse inline HTML. It is essentially a catch-all
                // state that will build up a single token buffer until it encounters an open tag
                // of some description.
                LexerState::Initial => {
                    tokens.append(&mut self.initial(&mut it)?);
                },
                // The scripting state is entered when an open tag is encountered in the source code.
                // This tells the lexer to start analysing characters at PHP tokens instead of inline HTML.
                LexerState::Scripting => {
                    while let Some(c) = it.peek() {
                        if ! c.is_whitespace() {
                            break;
                        }
                
                        it.next();
                    }

                    // If we have consumed whitespace and then reached the end of the file, we should break.
                    if let None = it.peek() {
                        break;
                    }

                    tokens.push(self.scripting(&mut it)?);
                },
            }
        }

        Ok(tokens)
    }

    #[allow(dead_code)]
    fn initial(&mut self, it: &mut Peekable<Chars>) -> Result<Vec<Token>, LexerError> {
        let mut buffer = String::new();
        while let Some(char) = it.next() {
            match char {
                '<' => {
                    // This is disgusting and can most definitely be tidied up with a multi-peek iterator.
                    if let Some('?') = it.peek() {
                        it.next();

                        if let Some('p') = it.peek() {
                            it.next();

                            if let Some('h') = it.peek() {
                                it.next();

                                if let Some('p') = it.peek() {
                                    it.next();

                                    self.enter_state(LexerState::Scripting);

                                    let mut tokens = vec!();

                                    if !buffer.is_empty() {
                                        tokens.push(Token {
                                            kind: TokenKind::InlineHtml(buffer),
                                            span: (0, 0),
                                        });
                                    }
                                    
                                    tokens.push(Token {
                                        kind: TokenKind::OpenTag(OpenTagKind::Full),
                                        span: (0, 0)
                                    });

                                    return Ok(tokens);
                                }
                            } else {
                                buffer.push('h');
                            }
                        } else {
                            buffer.push('?');
                        }
                    } else {
                        buffer.push(char);
                    }
                },
                _ => {
                    buffer.push(char);
                },
            }
        }

        Ok(vec![
            Token {
                kind: TokenKind::InlineHtml(buffer),
                span: (0, 0) // TODO: Actually track spans.
            }
        ])
    }

    fn scripting(&mut self, it: &mut Peekable<Chars>) -> Result<Token, LexerError> {
        // We should never reach this point since we have the empty checks surrounding
        // the call to this function, but it's better to be safe than sorry.
        if let None = it.peek() {
            return Err(LexerError::UnexpectedEndOfFile);
        }

        // Since we have the check above, we can safely unwrap the result of `.next()`
        // to help reduce the amount of indentation.
        let char = it.next().unwrap();

        let kind = match char {
            '?' => {
                // This is a close tag, we can enter "Initial" mode again.
                if let Some('>') = it.peek() {
                    it.next();

                    self.enter_state(LexerState::Initial);

                    TokenKind::CloseTag
                } else {
                    todo!();
                }
            },
            '$' => {
                let mut buffer = String::new();

                while let Some(n) = it.peek() {
                    match n {
                        'a'..='z' | 'A'..='Z' | '\u{80}'..='\u{ff}' | '_' => {
                            buffer.push(*n);
                            it.next();
                        }
                        _ => break,
                    }
                }

                TokenKind::Variable(buffer)
            },
            _ if char.is_alphabetic() => {
                let mut buffer = String::from(char);

                while let Some(n) = it.peek() {
                    if n.is_alphanumeric() || n == &'_' {
                        buffer.push(*n);
                        it.next();
                    } else {
                        break;
                    }
                }

                identifier_to_keyword(&buffer).unwrap_or(TokenKind::Identifier(buffer))
            },
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,
            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            ';' => TokenKind::SemiColon,
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '<' => TokenKind::LessThan,
            _ => unimplemented!("<scripting> char: {}", char),
        };

        Ok(Token {
            kind,
            span: (0, 0)
        })
    }

    fn enter_state(&mut self, state: LexerState) {
        self.state = state;
    }
}

#[allow(dead_code)]
fn identifier_to_keyword(ident: &str) -> Option<TokenKind> {
    Some(match ident {
        "function" => TokenKind::Function,
        "if" => TokenKind::If,
        "echo" => TokenKind::Echo,
        _ => return None,
    })
}

#[derive(Debug)]
pub enum LexerError {
    UnexpectedEndOfFile,
}

#[cfg(test)]
mod tests {
    use crate::{TokenKind, OpenTagKind};
    use super::Lexer;

    macro_rules! open {
        () => {
            TokenKind::OpenTag(OpenTagKind::Full)
        };
        ($kind:expr) => {
            TokenKind::OpenTag($kind)
        }
    }
    macro_rules! var {
        ($v:expr) => {
            TokenKind::Variable($v.into())
        };
    }

    #[test]
    fn basic_tokens() {
        assert_tokens("<?php ?>", &[
            open!(),
            TokenKind::CloseTag,
        ]);
    }

    #[test]
    fn inline_html() {
        assert_tokens("Hello, world!\n<?php", &[
            TokenKind::InlineHtml("Hello, world!\n".into()),
            open!(),
        ]);
    }

    #[test]
    fn keywords() {
        assert_tokens("<?php function if echo", &[
            open!(),
            TokenKind::Function,
            TokenKind::If,
            TokenKind::Echo,
        ]);
    }

    #[test]
    fn vars() {
        assert_tokens("<?php $one $_one $One $one_one", &[
            open!(),
            var!("one"),
            var!("_one"),
            var!("One"),
            var!("one_one"),
        ]);
    }

    #[test]
    fn punct() {
        assert_tokens("<?php {}();", &[
            open!(),
            TokenKind::LeftBrace,
            TokenKind::RightBrace,
            TokenKind::LeftParen,
            TokenKind::RightParen,
            TokenKind::SemiColon, 
        ]);
    }

    #[test]
    fn math() {
        assert_tokens("<?php + - <", &[
            open!(),
            TokenKind::Plus,
            TokenKind::Minus,
            TokenKind::LessThan,
        ]);
    }

    fn assert_tokens(source: &str, expected: &[TokenKind]) {
        let mut lexer = Lexer::new(None);
        let mut kinds = vec!();

        for token in lexer.tokenize(source).unwrap() {
            kinds.push(token.kind);
        }

        assert_eq!(kinds, expected);
    }
}