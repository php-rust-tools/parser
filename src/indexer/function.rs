use crate::{lexer::byte_string::ByteString, parser::ast::data_type::Type};

use super::{parameter::Parameter, source::Source};

#[derive(Debug)]
pub struct Function {
    pub name: ByteString,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub source: Source,
}
