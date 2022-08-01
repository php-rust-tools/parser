use std::{fmt::Display, io::{Write, BufRead}};
use rand::Rng;

#[derive(Clone)]
pub enum PhpValue {
    String(String),
    Int(i64),
}

impl PhpValue {
    pub fn eq(&self, other: Self) -> bool {
        match (self, &other) {
            (Self::Int(a), Self::String(b)) | (Self::String(b), Self::Int(a)) => match b.parse::<i64>() {
                Ok(b) => *a == b,
                _ => false,
            },
            _ => todo!(),
        }
    }
}

impl Into<i64> for PhpValue {
    fn into(self) -> i64 {
        match self {
            Self::Int(i) => i,
            _ => todo!(),
        }
    }
}

impl From<i64> for PhpValue {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<String> for PhpValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for PhpValue {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

impl Display for PhpValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(string) => write!(f, "{}", string),
            Self::Int(i) => write!(f, "{}", i),
            _ => todo!(),
        }
    }
}

pub fn _php_echo(value: PhpValue) {
    print!("{value}");
}

pub fn _php_concat(left: PhpValue, right: PhpValue) -> PhpValue {
    format!("{}{}", left, right).into()
}

// TODO: Make the `prompt` argument optional.
pub fn readline(prompt: PhpValue) -> PhpValue {
    print!("{}", prompt);

    std::io::stdout().flush().unwrap();

    let mut result = String::new();
    std::io::stdin().lock().read_line(&mut result).unwrap();

    PhpValue::from(result.trim_end())
}

pub fn rand(from: PhpValue, to: PhpValue) -> PhpValue {
    let from: i64 = from.into();
    let to: i64 = to.into();

    let mut rng = rand::thread_rng();

    PhpValue::from(rng.gen_range(from..to))
}