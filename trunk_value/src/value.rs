use indexmap::IndexMap;

#[derive(Debug, Clone)]
pub enum PhpValue {
    Int(i64),
    Float(f64),
    String(String),
    Array(Array),
}

pub type Array = IndexMap<ArrayKey, PhpValue>;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ArrayKey {
    Int(i64),
    String(String),
}

impl PhpValue {
    pub fn inspect_type(&self) -> String {
        match self {
            Self::Int(_) => "int".into(),
            Self::Float(_) => "float".into(),
            Self::String(_) => "string".into(),
            _ => todo!(),
        }
    }
}