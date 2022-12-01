use crate::lexer::token::Span;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StackState {
    Initial,
    Scripting,
    Halted,
    DoubleQuote,
    LookingForVarname,
    LookingForProperty,
    VarOffset,
}

// TODO(azjezz): make `chars` a `[u8, N]`, and `State`, `State<const N: usize>`
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct State {
    pub stack: Vec<StackState>,
    pub chars: Vec<u8>,
    pub cursor: usize,
    pub current: Option<u8>,
    pub span: Span,
}

impl State {
    pub fn new<B: ?Sized + AsRef<[u8]>>(input: &B) -> Self {
        let chars = input.as_ref().to_vec();
        let current = chars.first().copied();

        Self {
            stack: vec![StackState::Initial],
            chars,
            current,
            cursor: 0,
            span: (1, 1),
        }
    }

    pub fn enter_state(&mut self, state: StackState) {
        *self.stack.last_mut().unwrap() = state;
    }

    pub fn push_state(&mut self, state: StackState) {
        self.stack.push(state);
    }

    pub fn pop_state(&mut self) {
        self.stack.pop();
    }

    pub fn peek_buf(&self) -> &[u8] {
        &self.chars[self.cursor..]
    }

    pub fn peek_byte(&self, delta: usize) -> Option<u8> {
        self.chars.get(self.cursor + delta).copied()
    }

    pub fn try_read(&self, search: &'static [u8]) -> bool {
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
