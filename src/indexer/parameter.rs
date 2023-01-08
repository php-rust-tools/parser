use crate::{lexer::byte_string::ByteString, parser::ast::data_type::Type};

pub struct Parameter {
    pub name: ByteString,
    pub r#type: Option<Type>,
}
