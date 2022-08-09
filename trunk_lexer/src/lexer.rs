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
    chars: Vec<char>,
    cursor: usize,
    current: Option<char>,
    peek: Option<char>,
    col: usize,
    line: usize,
}

impl Lexer {
    pub fn new(config: Option<LexerConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            state: LexerState::Initial,
            chars: "".chars().collect(),
            cursor: 0,
            current: None,
            peek: None,
            line: 1,
            col: 0,
        }
    }

    pub fn tokenize(&mut self, input: &str) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();
        self.chars = input.chars().collect();

        self.next();
        self.next();

        while self.peek.is_some() {
            match self.state {
                // The "Initial" state is used to parse inline HTML. It is essentially a catch-all
                // state that will build up a single token buffer until it encounters an open tag
                // of some description.
                LexerState::Initial => {
                    tokens.append(&mut self.initial()?);
                },
                // The scripting state is entered when an open tag is encountered in the source code.
                // This tells the lexer to start analysing characters at PHP tokens instead of inline HTML.
                LexerState::Scripting => {
                    while let Some(c) = self.peek {
                        if ! c.is_whitespace() && ! ['\n', '\t', '\r'].contains(&c) {
                            break;
                        }
                
                        if c == '\n' {
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
                },
            }
        }

        Ok(tokens)
    }

    #[allow(dead_code)]
    fn initial(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut buffer = String::new();
        while let Some(char) = self.current {
            match char {
                '<' => {
                    // This is disgusting and can most definitely be tidied up with a multi-peek iterator.
                    if let Some('?') = self.peek {
                        self.next();

                        if let Some('p') = self.peek {
                            self.next();

                            if let Some('h') = self.peek {
                                self.next();

                                if let Some('p') = self.peek {
                                    self.next();

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
                    self.next();
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
            '!' => {
                self.col += 1;

                if let Some('=') = self.peek {
                    self.col += 1;

                    self.next();

                    if let Some('=') = self.peek {
                        self.col += 1;

                        self.next();

                        TokenKind::BangDoubleEquals
                    } else {
                        TokenKind::BangEquals
                    }
                } else {
                    TokenKind::Bang
                }
            },
            '&' => {
                self.col += 1;

                if let Some('&') = self.peek {
                    self.col += 1;

                    self.next();

                    TokenKind::BooleanAnd
                } else {
                    TokenKind::Ampersand
                }
            },
            '?' => {
                // This is a close tag, we can enter "Initial" mode again.
                if let Some('>') = self.peek {
                    self.next();

                    self.col += 2;

                    self.enter_state(LexerState::Initial);

                    TokenKind::CloseTag
                } else if let Some('?') = self.peek {
                    self.col += 1;

                    self.next();

                    if let Some('=') = self.peek {
                        self.col += 1;

                        self.next();

                        TokenKind::CoalesceEqual
                    } else {
                        TokenKind::Coalesce
                    }
                } else if let Some(':') = self.peek {
                    self.col += 1;
                    self.next();
                    TokenKind::QuestionColon
                } else if self.try_read("->") {
                    self.col += 1;
                    self.skip(3);
                    TokenKind::NullsafeArrow
                } else {
                    TokenKind::Question
                }
            },
            '=' => {
                if let Some('=') = self.peek {
                    self.next();

                    if let Some('=') = self.peek {
                        self.next();

                        self.col += 3;

                        TokenKind::TripleEquals
                    } else {
                        self.col += 2;

                        TokenKind::DoubleEquals
                    }
                } else if let Some('>') = self.peek {
                    self.next();
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

                while let Some(n) = self.peek {
                    if ! escaping && n == '\'' {
                        self.next();

                        break;
                    }

                    if n == '\\' && !escaping {
                        escaping = true;
                        self.next();
                        continue;
                    }

                    if escaping && ['\\', '\''].contains(&n) {
                        escaping = false;
                        buffer.push(n);
                        self.next();
                        continue;
                    }

                    if n == '\n' {
                        self.line += 1;
                        self.col = 0;
                    } else {
                        self.col += 1;
                    }

                    escaping = false;

                    buffer.push(n);
                    self.next();
                }

                TokenKind::ConstantString(buffer)
            },
            '"' => {
                self.col += 1;

                let mut buffer = String::new();
                let mut escaping = false;

                while let Some(n) = self.peek {
                    if ! escaping && n == '"' {
                        self.next();

                        break;
                    }

                    if n == '\\' && !escaping {
                        escaping = true;
                        self.next();
                        continue;
                    }

                    if escaping && ['\\', '"'].contains(&n) {
                        escaping = false;
                        buffer.push(n);
                        self.next();
                        continue;
                    }

                    if n == '\n' {
                        self.line += 1;
                        self.col = 0;
                    } else {
                        self.col += 1;
                    }

                    escaping = false;

                    buffer.push(n);
                    self.next();
                }

                TokenKind::ConstantString(buffer)
            },
            '$' => {
                let mut buffer = String::new();

                self.col += 1;

                while let Some(n) = self.peek {
                    match n {
                        '0'..='9' if buffer.len() > 1 => {
                            self.col += 1;
                            buffer.push(n);
                            self.next();
                        },
                        'a'..='z' | 'A'..='Z' | '\u{80}'..='\u{ff}' | '_'  => {
                            self.col += 1;

                            buffer.push(n);
                            self.next();
                        }
                        _ => break,
                    }
                }

                TokenKind::Variable(buffer)
            },
            '.' => {
                self.col += 1;

                if let Some('0'..='9') = self.peek {
                    let mut buffer = String::from("0.");
                    let mut underscore = false;

                    while let Some(n) = self.peek {
                        match n {
                            '0'..='9' => {
                                underscore = false;
                                buffer.push(n);
                                self.next();
    
                                self.col += 1;
                            },
                            '_' => {
                                if underscore {
                                    return Err(LexerError::UnexpectedCharacter(n));
                                }
    
                                underscore = true;
                                self.next();
    
                                self.col += 1;
                            },
                            _ => break,
                        }
                    }

                    TokenKind::Float(buffer.parse().unwrap())
                } else if let Some('.') = self.peek {
                    self.next();

                    self.col += 1;

                    if let Some('.') = self.peek {
                        self.next();

                        self.col += 1;

                        TokenKind::Ellipsis
                    } else {
                        todo!("don't know how to handle this case yet, it should just be 2 Dot tokens...")
                    }
                } else if let Some('=') = self.peek {
                    self.next();
                    self.col += 1;
                    TokenKind::DotEquals
                } else {
                    TokenKind::Dot
                }
            },
            '0'..='9' => {
                let mut buffer = String::from(char);
                let mut underscore = false;
                let mut is_float = false;

                self.col += 1;

                while let Some(n) = self.peek {
                    match n {
                        '0'..='9' => {
                            underscore = false;
                            buffer.push(n);
                            self.next();

                            self.col += 1;
                        },
                        '.' => {
                            if is_float {
                                return Err(LexerError::UnexpectedCharacter(n));
                            }

                            is_float = true;
                            buffer.push(n);
                            self.next();
                            self.col += 1;
                        },
                        '_' => {
                            if underscore {
                                return Err(LexerError::UnexpectedCharacter(n));
                            }

                            underscore = true;
                            self.next();

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
                self.col += 1;

                if let Some(n) = self.peek && (n.is_alphabetic() || n == '_') {
                    match self.scripting()? {
                        Token { kind: TokenKind::Identifier(i) | TokenKind::QualifiedIdentifier(i), .. } => {
                            TokenKind::FullyQualifiedIdentifier(format!("\\{}", i))
                        },
                        s => unreachable!("{:?}", s)
                    }
                } else {
                    TokenKind::NamespaceSeparator
                }
            },
            _ if char.is_alphabetic() || char == '_' => {
                self.col += 1;

                let mut qualified = false;
                let mut last_was_slash = false;

                let mut buffer = String::from(char);
                while let Some(next) = self.peek {
                    if next.is_alphanumeric() || next == '_' {
                        buffer.push(next);
                        self.next();
                        self.col += 1;
                        last_was_slash = false;
                        continue;
                    }

                    if next == '\\' && ! last_was_slash {
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
                    TokenKind::QualifiedIdentifier(buffer)
                } else {
                    identifier_to_keyword(&buffer).unwrap_or(TokenKind::Identifier(buffer))
                }
            },
            '/' | '#' => {
                self.col += 1;

                fn read_till_end_of_line(s: &mut Lexer) -> String {
                    s.col += 1;

                    let mut buffer = String::new();

                    while let Some(c) = s.peek {
                        if c == '\n' {
                            break;
                        }

                        buffer.push(c);
                        s.next();
                    }

                    buffer
                }

                if char == '/' && let Some(t) = self.peek && t == '*' {
                    let mut buffer = String::from(char);

                    while self.peek.is_some() {
                        self.next();

                        let t = self.current.unwrap();

                        match t {
                            '*' => {                     
                                if let Some('/') = self.peek {
                                    self.col += 2;
                                    buffer.push_str("*/");
                                    self.next();
                                    break;
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

                    if buffer.starts_with("/**") {
                        TokenKind::DocComment(buffer)
                    } else {
                        TokenKind::Comment(buffer)
                    }
                } else if char == '/' && let Some(t) = self.peek && t != '/' {
                    TokenKind::Slash
                } else if char == '#' && let Some(t) = self.peek && t == '[' {
                    TokenKind::Attribute
                } else {
                    self.next();
                    let buffer = format!("{}{}{}", char, &self.current.unwrap(), read_till_end_of_line(self));

                    TokenKind::Comment(buffer)
                }
            },
            '*' => {
                self.col += 1;

                if let Some('*') = self.peek {
                    self.col += 1;
                    self.next();
                    TokenKind::Pow
                } else if let Some('=') = self.peek {
                    self.col += 1;
                    self.next();
                    TokenKind::AsteriskEqual
                } else {
                    TokenKind::Asterisk
                }
            },
            '|' => {
                self.col += 1;
                
                if let Some('|') = self.peek {
                    self.col += 1;

                    self.next();

                    TokenKind::BooleanOr
                } else {
                    TokenKind::Pipe
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

                if self.try_read("string)") {
                    self.col += 7;
                    self.skip(8);
                    
                    TokenKind::StringCast 
                } else if self.try_read("object)") {
                    self.col += 7;
                    self.skip(8);

                    TokenKind::ObjectCast
                } else if self.try_read("bool)") {
                    self.col += 5;
                    self.skip(6);
                    TokenKind::BoolCast
                } else if self.try_read("int)") {
                    self.col += 4;
                    self.skip(5);
                    TokenKind::IntCast
                } else {
                    TokenKind::LeftParen
                }
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

                if let Some('=') = self.peek {
                    self.col += 1;

                    self.next();
                    
                    TokenKind::PlusEquals
                } else if let Some('+') = self.peek {
                    self.col += 1;

                    self.next();

                    TokenKind::Increment
                } else {
                    TokenKind::Plus
                }
            },
            '-' => {
                self.col += 1;
                
                if let Some('>') = self.peek {
                    self.col += 1;

                    self.next();

                    TokenKind::Arrow
                } else if let Some('=') = self.peek {
                    self.col += 1;
                    self.next();
                    TokenKind::MinusEquals
                } else {
                    TokenKind::Minus
                }
            },
            '<' => {
                self.col += 1;

                if let Some('=') = self.peek {
                    self.next();

                    self.col += 1;

                    TokenKind::LessThanEquals
                } else if let Some('<') = self.peek {
                    self.next();

                    if let Some('<') = self.peek {
                        // TODO: Handle both heredocs and nowdocs.
                        self.next();

                        todo!("heredocs & nowdocs");
                    } else {
                        TokenKind::LeftShift
                    }                 
                } else {
                    TokenKind::LessThan
                }
            },
            '>' => {
                self.col += 1;

                if let Some('=') = self.peek {
                    self.next();

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

                if let Some(':') = self.peek {
                    self.col += 1;
                    
                    self.next();
                    TokenKind::DoubleColon
                } else {
                    TokenKind::Colon
                }
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

    fn char_at(&self, idx: usize) -> Option<&char> {
        self.chars.get(idx)
    }

    fn try_read(&self, search: &'static str) -> bool {
        if self.current.is_none() || self.peek.is_none() {
            return false;
        }

        let start = self.cursor.saturating_sub(1);
        let mut buffer = String::new();

        for i in 0..search.len() {
            match self.char_at(start + i) {
                Some(char) => buffer.push(*char),
                _ => return false,
            };
        }

        buffer.as_str() == search
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
fn identifier_to_keyword(ident: &str) -> Option<TokenKind> {
    Some(match ident {
        "match" => TokenKind::Match,
        "abstract" => TokenKind::Abstract,
        "array" => TokenKind::Array,
        "as" => TokenKind::As,
        "break" => TokenKind::Break,
        "case" => TokenKind::Case,
        "catch" => TokenKind::Catch,
        "class" => TokenKind::Class,
        "clone" => TokenKind::Clone,
        "continue" => TokenKind::Continue,
        "const" => TokenKind::Const,
        "declare" => TokenKind::Declare,
        "default" => TokenKind::Default,
        "echo" => TokenKind::Echo,
        "else" => TokenKind::Else,
        "elseif" => TokenKind::ElseIf,
        "enum" => TokenKind::Enum,
        "extends" => TokenKind::Extends,
        "false" | "FALSE" => TokenKind::False,
        "final" => TokenKind::Final,
        "finally" => TokenKind::Finally,
        "fn" => TokenKind::Fn,
        "for" => TokenKind::For,
        "foreach" => TokenKind::Foreach,
        "function" => TokenKind::Function,
        "if" => TokenKind::If,
        "implements" => TokenKind::Implements,
        "interface" => TokenKind::Interface,
        "instanceof" => TokenKind::Instanceof,
        "namespace" => TokenKind::Namespace,
        "new" => TokenKind::New,
        "null" | "NULL" => TokenKind::Null,
        "private" => TokenKind::Private,
        "protected" => TokenKind::Protected,
        "public" => TokenKind::Public,
        "require" => TokenKind::Require,
        "require_once" => TokenKind::RequireOnce,
        "return" => TokenKind::Return,
        "static" => TokenKind::Static,
        "switch" => TokenKind::Switch,
        "throw" => TokenKind::Throw,
        "trait" => TokenKind::Trait,
        "true" | "TRUE" => TokenKind::True,
        "try" => TokenKind::Try,
        "use" => TokenKind::Use,
        "var" => TokenKind::Var,
        "yield" => TokenKind::Yield,
        "__DIR__" => TokenKind::DirConstant,
        "while" => TokenKind::While,
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
        assert_tokens("<?php function if else elseif echo return class extends implements public protected private static null NULL true TRUE false FALSE use const namespace interface new foreach instanceof", &[
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
        ]);
    }

    #[test]
    fn casts() {
        assert_tokens("<?php (object) (string)", &[
            open!(),
            TokenKind::ObjectCast,
            TokenKind::StringCast,
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
    fn multi_line_comments_before_structure() {
        assert_tokens(r#"<?php
/*
Hello
*/
function"#, &[
            open!(),
            TokenKind::Comment("/*\nHello\n*/".into()),
            TokenKind::Function,
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
        assert_tokens("<?php {}();, :: :", &[
            open!(),
            TokenKind::LeftBrace,
            TokenKind::RightBrace,
            TokenKind::LeftParen,
            TokenKind::RightParen,
            TokenKind::SemiColon, 
            TokenKind::Comma,
            TokenKind::DoubleColon,
            TokenKind::Colon,
        ]);
    }

    #[test]
    fn sigils() {
        assert_tokens("<?php ->", &[
            open!(),
            TokenKind::Arrow,
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