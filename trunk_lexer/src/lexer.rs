use std::{iter::Peekable, str::Chars, char};

use crate::{Token, TokenKind, OpenTagKind};

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
    col: usize,
    line: usize,
}

impl Lexer {
    pub fn new(config: Option<LexerConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            state: LexerState::Initial,
            line: 1,
            col: 0,
        }
    }

    pub fn tokenize(&mut self, input: &str) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();
        let mut it = input[..].chars().peekable();

        while it.peek().is_some() {
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
                        if ! c.is_whitespace() && ! ['\n', '\t', '\r'].contains(c) {
                            break;
                        }
                
                        if *c == '\n' {
                            self.line += 1;
                            self.col = 0;
                        } else {
                            self.col += 1;
                        }

                        it.next();
                    }

                    // If we have consumed whitespace and then reached the end of the file, we should break.
                    if it.peek().is_none() {
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

                                    self.col += 4;

                                    self.enter_state(LexerState::Scripting);

                                    let mut tokens = vec!();

                                    if !buffer.is_empty() {
                                        tokens.push(Token {
                                            kind: TokenKind::InlineHtml(buffer),
                                            span: (self.line, self.col.saturating_sub(5)),
                                        });
                                    }
                                    
                                    tokens.push(Token {
                                        kind: TokenKind::OpenTag(OpenTagKind::Full),
                                        span: (self.line, self.col)
                                    });

                                    return Ok(tokens);
                                }
                            } else {
                                self.col += 3;

                                buffer.push('h');
                            }
                        } else {
                            self.col += 2;

                            buffer.push('?');
                        }
                    } else {
                        self.col += 1;

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
                span: (self.line, self.col)
            }
        ])
    }

    fn scripting(&mut self, it: &mut Peekable<Chars>) -> Result<Token, LexerError> {
        // We should never reach this point since we have the empty checks surrounding
        // the call to this function, but it's better to be safe than sorry.
        if it.peek().is_none() {
            return Err(LexerError::UnexpectedEndOfFile);
        }

        // Since we have the check above, we can safely unwrap the result of `.next()`
        // to help reduce the amount of indentation.
        let char = it.next().unwrap();

        let kind = match char {
            '!' => {
                self.col += 1;
                TokenKind::Bang
            },
            '&' => {
                self.col += 1;

                if let Some('&') = it.peek() {
                    self.col += 1;

                    it.next();

                    TokenKind::BooleanAnd
                } else {
                    TokenKind::BitAnd
                }
            },
            '?' => {
                // This is a close tag, we can enter "Initial" mode again.
                if let Some('>') = it.peek() {
                    it.next();

                    self.col += 2;

                    self.enter_state(LexerState::Initial);

                    TokenKind::CloseTag
                } else {
                    TokenKind::Question
                }
            },
            '=' => {
                if let Some('=') = it.peek() {
                    it.next();

                    if let Some('=') = it.peek() {
                        it.next();

                        self.col += 3;

                        TokenKind::TripleEquals
                    } else {
                        self.col += 2;

                        TokenKind::DoubleEquals
                    }
                } else if let Some('>') = it.peek() {
                    it.next();
                    self.col += 1;
                    TokenKind::DoubleArrow
                } else {
                    self.col += 1;

                    TokenKind::Equals
                }
            },
            // Single quoted string.
            '\'' => {
                self.col += 1;

                let mut buffer = String::new();
                let mut escaping = false;

                while let Some(n) = it.peek() {
                    if ! escaping && *n == '\'' {
                        it.next();

                        break;
                    }

                    if *n == '\\' && !escaping {
                        escaping = true;
                        it.next();
                        continue;
                    }

                    if escaping && ['\\', '\''].contains(n) {
                        escaping = false;
                        buffer.push(*n);
                        it.next();
                        continue;
                    }

                    if *n == '\n' {
                        self.line += 1;
                        self.col = 0;
                    } else {
                        self.col += 1;
                    }

                    escaping = false;

                    buffer.push(*n);
                    it.next();
                }

                TokenKind::ConstantString(buffer)
            },
            '$' => {
                let mut buffer = String::new();

                self.col += 1;

                while let Some(n) = it.peek() {
                    match n {
                        'a'..='z' | 'A'..='Z' | '\u{80}'..='\u{ff}' | '_' => {
                            self.col += 1;

                            buffer.push(*n);
                            it.next();
                        }
                        _ => break,
                    }
                }

                TokenKind::Variable(buffer)
            },
            '.' => {
                self.col += 1;

                if let Some('0'..='9') = it.peek() {
                    let mut buffer = String::from("0.");
                    let mut underscore = false;

                    while let Some(n) = it.peek() {
                        match n {
                            '0'..='9' => {
                                underscore = false;
                                buffer.push(*n);
                                it.next();
    
                                self.col += 1;
                            },
                            '_' => {
                                if underscore {
                                    return Err(LexerError::UnexpectedCharacter(*n));
                                }
    
                                underscore = true;
                                it.next();
    
                                self.col += 1;
                            },
                            _ => break,
                        }
                    }

                    TokenKind::Float(buffer.parse().unwrap())
                } else {
                    TokenKind::Dot
                }
            },
            '0'..='9' => {
                let mut buffer = String::from(char);
                let mut underscore = false;
                let mut is_float = false;

                self.col += 1;

                while let Some(n) = it.peek() {
                    match n {
                        '0'..='9' => {
                            underscore = false;
                            buffer.push(*n);
                            it.next();

                            self.col += 1;
                        },
                        '.' => {
                            if is_float {
                                return Err(LexerError::UnexpectedCharacter(*n));
                            }

                            is_float = true;
                            buffer.push(*n);
                            it.next();
                            self.col += 1;
                        },
                        '_' => {
                            if underscore {
                                return Err(LexerError::UnexpectedCharacter(*n));
                            }

                            underscore = true;
                            it.next();

                            self.col += 1;
                        },
                        _ => break,
                    }
                }

                if is_float {
                    TokenKind::Float(buffer.parse().unwrap())
                } else {
                    TokenKind::Int(buffer.parse().unwrap())
                }
            },
            '\\' => {
                // TODO: Handle fully-qualified identifiers here.
                TokenKind::NamespaceSeparator
            },
            _ if char.is_alphabetic() || char == '_' => {
                self.col += 1;

                let mut qualified = false;
                let mut last_was_slash = false;

                let mut buffer = String::from(char);
                while let Some(next) = it.peek() {
                    if next.is_alphabetic() || *next == '_' {
                        buffer.push(*next);
                        it.next();
                        self.col += 1;
                        last_was_slash = false;
                        continue;
                    }

                    if *next == '\\' && ! last_was_slash {
                        qualified = true;
                        last_was_slash = true;
                        buffer.push(*next);
                        it.next();
                        self.col += 1;
                        continue;
                    }

                    break;
                }

                if qualified {
                    TokenKind::QualifiedIdentifier(buffer)
                } else {
                    identifier_to_keyword(&buffer).unwrap_or(TokenKind::Identifier(buffer))
                }
            },
            '/' | '#' => {
                self.col += 1;

                fn read_till_end_of_line(s: &mut Lexer, it: &mut Peekable<Chars>) -> String {
                    s.col += 1;

                    let mut buffer = String::new();

                    while let Some(c) = it.peek() {
                        if *c == '\n' {
                            break;
                        }

                        buffer.push(*c);
                        it.next();
                    }

                    buffer
                }

                if char == '/' && let Some(t) = it.peek() && *t == '*' {
                    let mut buffer = String::from(char);

                    while let Some(_) = it.peek() {
                        let t = it.next().unwrap();

                        match t {
                            '*' => {                                
                                if let Some('/') = it.peek() {
                                    self.col += 2;
                                    buffer.push_str("*/");
                                    it.next();
                                } else {
                                    self.col += 1;
                                    buffer.push(t);
                                }
                            },
                            '\n' => {
                                self.line += 1;
                                self.col = 0;

                                buffer.push('\n');
                            },
                            _ => {
                                self.col += 1;

                                buffer.push(t);
                            }
                        }
                    }

                    TokenKind::Comment(buffer)
                } else if char == '/' && let Some(t) = it.peek() && *t != '/' {
                    TokenKind::Slash
                } else if char == '#' && let Some(t) = it.peek() && *t == '[' {
                    TokenKind::Attribute
                } else {
                    let buffer = format!("{}{}{}", char, it.next().unwrap(), read_till_end_of_line(self, it));

                    TokenKind::Comment(buffer)
                }
            },
            '{' => {
                self.col += 1;
                TokenKind::LeftBrace
            },
            '}' => {
                self.col += 1;
                TokenKind::RightBrace
            },
            '(' => {
                self.col += 1;
                TokenKind::LeftParen
            },
            ')' => {
                self.col += 1;
                TokenKind::RightParen
            },
            ';' => {
                self.col += 1;
                TokenKind::SemiColon
            },
            '+' => {
                self.col += 1;
                TokenKind::Plus
            },
            '-' => {
                self.col += 1;
                TokenKind::Minus
            },
            '<' => {
                self.col += 1;

                if let Some('=') = it.peek() {
                    it.next();

                    self.col += 1;

                    TokenKind::LessThanEquals
                } else {
                    TokenKind::LessThan
                }
            },
            '>' => {
                self.col += 1;

                if let Some('=') = it.peek() {
                    it.next();

                    self.col += 1;

                    TokenKind::GreaterThanEquals
                } else {
                    TokenKind::GreaterThan
                }
            },
            ',' => {
                self.col += 1;
                TokenKind::Comma
            },
            '[' => {
                self.col += 1;
                TokenKind::LeftBracket
            },
            ']' => {
                self.col += 1;
                TokenKind::RightBracket
            },
            ':' => {
                self.col += 1;
                TokenKind::Colon
            },
            _ => unimplemented!("<scripting> char: {}, line: {}, col: {}", char, self.line, self.col),
        };

        Ok(Token {
            kind,
            span: (self.line, self.col)
        })
    }

    fn enter_state(&mut self, state: LexerState) {
        self.state = state;
    }
}

#[allow(dead_code)]
fn identifier_to_keyword(ident: &str) -> Option<TokenKind> {
    Some(match ident {
        "use" => TokenKind::Use,
        "null" | "NULL" => TokenKind::Null,
        "abstract" => TokenKind::Abstract,
        "class" => TokenKind::Class,
        "declare" => TokenKind::Declare,
        "echo" => TokenKind::Echo,
        "else" => TokenKind::Else,
        "elseif" => TokenKind::ElseIf,
        "enum" => TokenKind::Enum,
        "extends" => TokenKind::Extends,
        "final" => TokenKind::Final,
        "function" => TokenKind::Function,
        "if" => TokenKind::If,
        "implements" => TokenKind::Implements,
        "private" => TokenKind::Private,
        "protected" => TokenKind::Protected,
        "public" => TokenKind::Public,
        "return" => TokenKind::Return,
        "static" => TokenKind::Static,
        "var" => TokenKind::Var,
        "true" | "TRUE" => TokenKind::True,
        "false" | "FALSE" => TokenKind::False,
        "const" => TokenKind::Const,
        _ => return None,
    })
}

#[derive(Debug)]
pub enum LexerError {
    UnexpectedEndOfFile,
    UnexpectedCharacter(char),
}

#[cfg(test)]
mod tests {
    use crate::{TokenKind, OpenTagKind, Token};
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
    macro_rules! int {
        ($i:expr) => {
            TokenKind::Int($i)
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
        assert_tokens("<?php function if else elseif echo return class extends implements public protected private static null NULL true TRUE false FALSE use const", &[
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
        ]);
    }

    #[test]
    fn constant_single_quote_strings() {
        assert_tokens(r#"<?php 'Hello, world!' 'I\'m a developer.' 'This is a backslash \\.' 'This is a multi-line
string.'"#, &[
            open!(),
            TokenKind::ConstantString("Hello, world!".into()),
            TokenKind::ConstantString("I'm a developer.".into()),
            TokenKind::ConstantString("This is a backslash \\.".into()),
            TokenKind::ConstantString("This is a multi-line\nstring.".into()),
        ]);
    }

    #[test]
    fn single_line_comments() {
        assert_tokens(r#"<?php
        // Single line comment.
        # Another single line comment.
        "#, &[
            open!(),
            TokenKind::Comment("// Single line comment.".into()),
            TokenKind::Comment("# Another single line comment.".into()),
        ]);
    }

    #[test]
    fn multi_line_comments() {
        assert_tokens(r#"<?php
/*
Hello
*/"#, &[
            open!(),
            TokenKind::Comment("/*\nHello\n*/".into()),
        ])
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
    fn nums() {
        assert_tokens("<?php 1 1_000 1_000_000", &[
            open!(),
            int!(1),
            int!(1_000),
            int!(1_000_000),
        ]);
    }

    #[test]
    fn punct() {
        assert_tokens("<?php {}();,", &[
            open!(),
            TokenKind::LeftBrace,
            TokenKind::RightBrace,
            TokenKind::LeftParen,
            TokenKind::RightParen,
            TokenKind::SemiColon, 
            TokenKind::Comma,
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

    #[test]
    fn identifiers() {
        assert_tokens("<?php \\ Unqualified Is\\Qualified", &[
            open!(),
            TokenKind::NamespaceSeparator,
            TokenKind::Identifier("Unqualified".into()),
            TokenKind::QualifiedIdentifier("Is\\Qualified".into()),
        ]);
    }

    #[test]
    fn equals() {
        assert_tokens("<?php = == ===", &[
            open!(),
            TokenKind::Equals,
            TokenKind::DoubleEquals,
            TokenKind::TripleEquals,
        ]);
    }

    #[test]
    fn span_tracking() {
        let spans = get_spans("<?php hello_world()");

        assert_eq!(spans, &[
            (1, 4),
            (1, 16),
            (1, 17),
            (1, 18),
        ]);

        let spans = get_spans(r#"<?php
        
function hello_world() {

}"#);
        
        assert_eq!(spans, &[
            (1, 4),
            (3, 8),
            (3, 20),
            (3, 21),
            (3, 22),
            (3, 24),
            (5, 1),
        ]);
    }

    #[test]
    fn floats() {
        assert_tokens("<?php 200.5 .05", &[
            open!(),
            TokenKind::Float(200.5),
            TokenKind::Float(0.05),
        ]);
    }

    fn assert_tokens(source: &str, expected: &[TokenKind]) {
        let mut kinds = vec!();

        for token in get_tokens(source) {
            kinds.push(token.kind);
        }

        assert_eq!(kinds, expected);
    }

    fn get_spans(source: &str) -> Vec<(usize, usize)> {
        let tokens = get_tokens(source);
        let mut spans = vec!();
        
        for token in tokens {
            spans.push(token.span);
        }

        spans
    }

    fn get_tokens(source: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(None);
        lexer.tokenize(source).unwrap()
    }
}