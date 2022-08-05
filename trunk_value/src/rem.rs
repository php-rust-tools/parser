use std::ops::Rem;

use crate::{PhpValue, TypeError};

impl Rem for PhpValue {
    type Output = Result<Self, TypeError>;

    fn rem(self, rhs: Self) -> Self::Output {
        Ok(match (&self, &rhs) {
            (Self::Int(a), Self::Int(b)) => Self::Int(a % b),
            (Self::Float(a), Self::Float(b)) => Self::Float(a % b),
            (Self::Float(a), Self::Int(b)) | (Self::Int(b), Self::Float(a)) => {
                Self::Float(a % *b as f64)
            },
            (Self::String(a), Self::Int(b)) | (Self::Int(b), Self::String(a)) => {
                if a.parse::<i64>().is_ok() {
                    Self::Int(a.parse::<i64>().unwrap() % b)
                } else {
                    return Err(TypeError::UnsupportedOperandTypes { lhs: self.inspect_type(), op: "%", rhs: rhs.inspect_type() })   
                }
            },
            (Self::String(a), Self::Float(b)) | (Self::Float(b), Self::String(a)) => {
                if a.parse::<f64>().is_ok() {
                    Self::Float(a.parse::<f64>().unwrap() % b)
                } else {
                    return Err(TypeError::UnsupportedOperandTypes { lhs: self.inspect_type(), op: "%", rhs: rhs.inspect_type() })   
                }
            },
            _ => return Err(TypeError::UnsupportedOperandTypes { lhs: self.inspect_type(), op: "%", rhs: rhs.inspect_type() })
        })
    }
}