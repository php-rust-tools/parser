use std::{ops::Add, ops::Sub, fmt::Display};

pub fn _internal_constant_to_string(constant: &str) -> String {
    match constant {
        "PHP_EOL" => "\n".into(),
        _ => unreachable!()
    }
}

#[derive(Clone, PartialEq)]
pub enum Value {
    Int(i64),
    String(String),
}

impl PartialOrd for Value {
    fn lt(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a < b,
            _ => todo!()
        }
    }

    fn ge(&self, other: &Self) -> bool {
        todo!()
    }

    fn le(&self, other: &Self) -> bool {
        todo!()
    }
    
    fn gt(&self, other: &Self) -> bool {
        todo!()
    }

    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        todo!()
    }
}

impl Sub for Value {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Int(a), Self::Int(b)) => Self::Int(a - b),
            _ => todo!(),
        }
    }
}

impl Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Int(a), Self::Int(b)) => Self::Int(a + b),
            _ => todo!()
        }
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            _ => todo!(),
        }
    }
}

pub fn _internal_echo(values: &[Value]) {
    for value in values {
        print!("{}", value)
    }
}

pub fn _internal_concat(left: Value, right: Value) -> Value {
    return format!("{}{}", left, right).into()
}