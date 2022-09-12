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

pub enum Associativity {
    None,
    Left,
    Right,
}