use std::fmt::Display;

enum PhpValue {
    String(String),
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
            _ => todo!(),
        }
    }
}

fn _php_echo(value: PhpValue) {
    print!("{value}");
}