use std::collections::VecDeque;

use crate::lexer::byte_string::ByteString;
use crate::lexer::error::SyntaxError;
use crate::lexer::error::SyntaxResult;
use crate::lexer::token::Span;

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, Copy)]
pub enum DocStringKind {
    Heredoc,
    Nowdoc,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StackFrame {
    Initial,
    Scripting,
    Halted,
    DoubleQuote,
    ShellExec,
    DocString(DocStringKind, ByteString),
    LookingForVarname,
    LookingForProperty,
    VarOffset,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct State {
    pub stack: VecDeque<StackFrame>,
    pub chars: Vec<u8>,
    pub cursor: usize,
    pub current: Option<u8>,
    pub span: Span,
}

impl State {
    pub fn new<B: ?Sized + AsRef<[u8]>>(input: &B) -> Self {
        let chars = input.as_ref().to_vec();
        let current = chars.first().copied();

        let mut stack = VecDeque::with_capacity(32);
        stack.push_back(StackFrame::Initial);

        Self {
            stack,
            chars,
            current,
            cursor: 0,
            span: (1, 1),
        }
    }

    pub fn frame(&self) -> SyntaxResult<&StackFrame> {
        self.stack
            .back()
            .ok_or(SyntaxError::UnpredictableState(self.span))
    }

    pub fn replace(&mut self, state: StackFrame) {
        let i = self.stack.len() - 1;

        self.stack[i] = state;
    }

    pub fn enter(&mut self, state: StackFrame) {
        self.stack.push_back(state);
    }

    pub fn exit(&mut self) {
        self.stack.pop_back();
    }

    pub fn peek_buf(&self) -> &[u8] {
        &self.chars[self.cursor..]
    }

    pub fn peek_byte(&self, delta: usize) -> Option<u8> {
        self.chars.get(self.cursor + delta).copied()
    }

    pub fn try_read(&self, search: &[u8]) -> bool {
        self.peek_buf().starts_with(search)
    }

    pub fn skip(&mut self, count: usize) {
        for _ in 0..count {
            self.next();
        }
    }

    pub fn next(&mut self) {
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
