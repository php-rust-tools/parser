#[derive(Debug, Eq, PartialEq)]
pub enum TypeError {
    UnsupportedOperandTypes {
        lhs: String,
        op: &'static str,
        rhs: String,
    }
}