use crate::{lexer::byte_string::ByteString, parser::ast::data_type::Type};

use super::function::Function;

#[derive(Debug)]
pub struct Index {
    pub functions: Vec<Function>,
}

impl Index {
    pub(crate) fn new() -> Self {
        Self {
            functions: Vec::new(),
        }
    }

    pub(crate) fn add_function(&mut self, function: Function) {
        self.functions.push(function);
    }
}
