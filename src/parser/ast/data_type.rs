use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use std::fmt::Display;

use crate::lexer::byte_string::ByteString;
use crate::lexer::token::Span;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum Type {
    Named(Span, ByteString),
    Nullable(Box<Type>),
    Union(Vec<Type>),
    Intersection(Vec<Type>),
    Void(Span),
    Null(Span),
    True(Span),
    False(Span),
    Never(Span),
    Float(Span),
    Boolean(Span),
    Integer(Span),
    String(Span),
    Array(Span),
    Object(Span),
    Mixed(Span),
    Callable(Span),
    Iterable(Span),
    StaticReference(Span),
    SelfReference(Span),
    ParentReference(Span),
}

impl Type {
    pub fn standalone(&self) -> bool {
        matches!(self, Type::Mixed(_) | Type::Never(_) | Type::Void(_))
    }

    pub fn nullable(&self) -> bool {
        matches!(self, Type::Nullable(_))
    }

    pub fn includes_callable(&self) -> bool {
        match &self {
            Self::Callable(_) => true,
            Self::Union(types) | Self::Intersection(types) => {
                types.iter().any(|x| x.includes_callable())
            }
            _ => false,
        }
    }

    pub fn includes_class_scoped(&self) -> bool {
        match &self {
            Self::StaticReference(_) | Self::SelfReference(_) | Self::ParentReference(_) => true,
            Self::Union(types) | Self::Intersection(types) => {
                types.iter().any(|x| x.includes_class_scoped())
            }
            _ => false,
        }
    }

    pub fn is_bottom(&self) -> bool {
        matches!(self, Type::Never(_) | Type::Void(_))
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Type::Named(_, inner) => write!(f, "{}", inner),
            Type::Nullable(inner) => write!(f, "{}", inner),
            Type::Union(inner) => write!(
                f,
                "{}",
                inner
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<String>>()
                    .join("|")
            ),
            Type::Intersection(inner) => write!(
                f,
                "{}",
                inner
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<String>>()
                    .join("&")
            ),
            Type::Void(_) => write!(f, "void"),
            Type::Null(_) => write!(f, "null"),
            Type::True(_) => write!(f, "true"),
            Type::False(_) => write!(f, "false"),
            Type::Never(_) => write!(f, "never"),
            Type::Float(_) => write!(f, "float"),
            Type::Boolean(_) => write!(f, "bool"),
            Type::Integer(_) => write!(f, "int"),
            Type::String(_) => write!(f, "string"),
            Type::Array(_) => write!(f, "array"),
            Type::Object(_) => write!(f, "object"),
            Type::Mixed(_) => write!(f, "mixed"),
            Type::Callable(_) => write!(f, "callable"),
            Type::Iterable(_) => write!(f, "iterable"),
            Type::StaticReference(_) => write!(f, "static"),
            Type::SelfReference(_) => write!(f, "self"),
            Type::ParentReference(_) => write!(f, "parent"),
        }
    }
}
