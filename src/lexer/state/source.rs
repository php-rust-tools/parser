use crate::lexer::token::Span;

#[derive(Debug)]
pub struct Source<'a> {
    input: &'a [u8],
    length: usize,
    cursor: usize,
    span: Span,
}

impl<'a> Source<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        let input = input;
        let length = input.len();

        Self {
            input,
            length,
            cursor: 0,
            span: (1, 1),
        }
    }

    pub const fn span(&self) -> Span {
        self.span
    }

    pub const fn eof(&self) -> bool {
        self.cursor >= self.length
    }

    pub fn next(&mut self) {
        if !self.eof() {
            match self.input[self.cursor] {
                b'\n' => {
                    self.span.0 += 1;
                    self.span.1 = 1;
                }
                _ => self.span.1 += 1,
            }
        }

        self.cursor += 1;
    }

    pub fn skip(&mut self, count: usize) {
        for _ in 0..count {
            self.next();
        }
    }

    pub fn current(&self) -> Option<&'a u8> {
        if self.cursor >= self.length {
            None
        } else {
            Some(&self.input[self.cursor])
        }
    }

    pub fn read(&self, n: usize) -> &'a [u8] {
        let (from, until) = self.to_bound(n);

        &self.input[from..until]
    }

    #[inline(always)]
    pub fn read_remaining(&self) -> &'a [u8] {
        &self.input[(if self.cursor >= self.length {
            self.length
        } else {
            self.cursor
        })..]
    }

    pub fn at(&self, search: &[u8], len: usize) -> bool {
        self.read(len) == search
    }

    pub fn at_case_insensitive(&self, search: &[u8], len: usize) -> bool {
        let (from, until) = self.to_bound(len);

        let slice = &self.input[from..until];

        slice.eq_ignore_ascii_case(search)
    }

    pub fn peek(&self, i: usize, n: usize) -> &'a [u8] {
        let from = self.cursor + i;
        if from >= self.length {
            return &self.input[self.length..self.length];
        }

        let mut until = from + n;
        if until >= self.length {
            until = self.length;
        }

        &self.input[from..until]
    }

    pub fn peek_ignoring_whitespace(&self, i: usize, n: usize) -> &'a [u8] {
        let mut i = i;

        loop {
            let c = self.peek(i, 1);

            if c.is_empty() {
                return &[];
            }

            match c[0] {
                b' ' | b'\t' | b'\r' | b'\n' => i += 1,
                _ => break,
            }
        }

        self.peek(i, n)
    }

    const fn to_bound(&self, n: usize) -> (usize, usize) {
        if self.cursor >= self.length {
            return (self.length, self.length);
        }

        let mut until = self.cursor + n;

        if until >= self.length {
            until = self.length;
        }

        (self.cursor, until)
    }
}
