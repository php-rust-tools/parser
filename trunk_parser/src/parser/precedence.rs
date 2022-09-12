use trunk_lexer::TokenKind;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    CloneNew,
    Pow,
    Prefix,
    Instanceof,
    Bang,
    MulDivMod,
    AddSub,
    BitShift,
    Concat,
    LtGt,
    Equality,
    BitwiseAnd,
    BitwiseXor,
    BitwiseOr,
    And,
    Or,
    NullCoalesce,
    Ternary,
    Assignment,
    YieldFrom,
    Yield,
    Print,
    KeyAnd,
    KeyXor,
    KeyOr,
}

impl Precedence {
    pub fn prefix(kind: &TokenKind) -> Self {
        use TokenKind::*;

        match kind {
            Bang => Self::Bang,
            Clone | New => Self::CloneNew,
            _ => Self::Prefix,
        }
    }

    pub fn infix(kind: &TokenKind) -> Self {
        use TokenKind::*;

        match kind {
            Instanceof => Self::Instanceof,
            Asterisk | Slash | Percent => Self::MulDivMod,
            Plus | Minus => Self::AddSub,
            LeftShift | RightShift => Self::BitShift,
            Dot => Self::Concat,
            LessThan | LessThanEquals | GreaterThan | GreaterThanEquals => Self::LtGt,
            DoubleEquals | BangEquals | TripleEquals | BangDoubleEquals => Self::Equality,
            Ampersand => Self::BitwiseAnd,
            Caret => Self::BitwiseXor,
            Pipe => Self::BitwiseOr,
            BooleanAnd => Self::And,
            BooleanOr => Self::Or,
            Coalesce => Self::NullCoalesce,
            Question => Self::Ternary,
            Equals | PlusEquals | MinusEquals | AsteriskEqual | PowEquals | SlashEquals | DotEquals | AndEqual | CoalesceEqual => Self::Assignment,
            Yield => Self::Yield,
            _ => unimplemented!("precedence for op {:?}", kind)
        }
    }
}

pub enum Associativity {
    None,
    Left,
    Right,
}