mod function;
mod index;
mod parameter;
mod source;

use std::path::PathBuf;

pub use index::Index;

use crate::{
    lexer::{byte_string::ByteString, token::Span},
    node::downcast,
    parser::ast::{
        functions::{FunctionStatement, ReturnType},
        identifiers::SimpleIdentifier,
        namespaces::{BracedNamespace, UnbracedNamespace},
        Statement,
    },
    traverser::Visitor,
};

use self::{function::Function, parameter::Parameter, source::Source};

pub struct Indexer {
    file: Option<PathBuf>,
    namespace: Option<ByteString>,
    index: Index,
}

impl Indexer {
    pub fn new() -> Self {
        Self {
            file: None,
            namespace: None,
            index: Index::new(),
        }
    }

    pub fn index(&mut self, file: PathBuf, program: &[Statement]) -> Result<(), IndexerError> {
        self.file = Some(file);

        for statement in program {
            self.visit_node(statement)?;
        }

        Ok(())
    }

    fn join_names(&self, name: &ByteString) -> ByteString {
        if let Some(namespace) = &self.namespace {
            let mut joined = namespace.clone();
            joined.push(b'\\');
            joined.extend(&mut name.iter());
            joined
        } else {
            name.clone()
        }
    }

    fn source(&self, span: &Span) -> Source {
        Source {
            file: self.file.clone().unwrap(),
            line: span.line,
            column: span.column,
            position: span.position,
        }
    }
}

impl Visitor<IndexerError> for Indexer {
    fn visit(&mut self, node: &dyn crate::node::Node) -> Result<(), IndexerError> {
        if let Some(UnbracedNamespace { name, .. }) = downcast::<UnbracedNamespace>(node) {
            self.namespace = Some(name.value.clone());
        } else if let Some(BracedNamespace { name, .. }) = downcast::<BracedNamespace>(node) {
            if let Some(SimpleIdentifier { value, .. }) = name {
                self.namespace = Some(value.clone());
            }
        } else if let Some(FunctionStatement {
            name: SimpleIdentifier { span, value },
            parameters,
            return_type,
            ..
        }) = downcast::<FunctionStatement>(node)
        {
            let name = self.join_names(&value);
            let parameters = parameters
                .parameters
                .inner
                .iter()
                .map(|p| Parameter {
                    name: p.name.name.clone(),
                    r#type: p.data_type.clone(),
                })
                .collect::<Vec<Parameter>>();
            let return_type = if let Some(ReturnType { data_type, .. }) = return_type {
                Some(data_type.clone())
            } else {
                None
            };

            self.index.add_function(Function {
                name,
                parameters,
                return_type,
                source: self.source(&span),
            });
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum IndexerError {}
