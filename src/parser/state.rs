use std::collections::VecDeque;
use std::fmt::Display;

use crate::parser::ast::attributes::AttributeGroup;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::modifiers::ClassModifierGroup;
use crate::parser::ast::modifiers::MethodModifierGroup;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::stream::TokenStream;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NamespaceType {
    Braced,
    Unbraced,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Scope {
    Namespace(SimpleIdentifier),
    BracedNamespace(Option<SimpleIdentifier>),

    Interface(SimpleIdentifier),
    Class(SimpleIdentifier, ClassModifierGroup, bool),
    Trait(SimpleIdentifier),
    Enum(SimpleIdentifier, bool),
    AnonymousClass(bool),

    Function(SimpleIdentifier),
    Method(SimpleIdentifier, MethodModifierGroup),
    AnonymousFunction(bool),
    ArrowFunction(bool),
}

#[derive(Debug)]
pub struct State<'a> {
    pub stack: VecDeque<Scope>,
    pub stream: &'a mut TokenStream<'a>,
    pub attributes: Vec<AttributeGroup>,
    pub namespace_type: Option<NamespaceType>,
    pub has_class_scope: bool,
    pub has_class_parent_scope: bool,
}

impl<'a> State<'a> {
    pub fn new(tokens: &'a mut TokenStream<'a>) -> Self {
        Self {
            stack: VecDeque::with_capacity(32),
            stream: tokens,
            namespace_type: None,
            has_class_scope: false,
            has_class_parent_scope: false,
            attributes: vec![],
        }
    }

    pub fn attribute(&mut self, attr: AttributeGroup) {
        self.attributes.push(attr);
    }

    pub fn get_attributes(&mut self) -> Vec<AttributeGroup> {
        let mut attributes = vec![];

        std::mem::swap(&mut self.attributes, &mut attributes);

        attributes
    }

    /// Return the namespace type used in the current state
    ///
    /// The namespace type is retrieve from the last entered
    /// namespace scope.
    ///
    /// Note: even when a namespace scope is exited, the namespace type
    /// is retained, until the next namespace scope is entered.
    pub fn namespace_type(&self) -> Option<NamespaceType> {
        self.namespace_type.clone()
    }

    pub fn namespace(&self) -> Option<&Scope> {
        for scope in &self.stack {
            match scope {
                Scope::Namespace(_) | Scope::BracedNamespace(_) => {
                    return Some(scope);
                }
                _ => {}
            }
        }

        None
    }

    pub fn named<T: Display + ?Sized>(&self, name: &T) -> String {
        match self.namespace() {
            Some(Scope::Namespace(n)) | Some(Scope::BracedNamespace(Some(n))) => {
                format!("{}\\{}", n, name)
            }
            _ => name.to_string(),
        }
    }

    pub fn scope(&self) -> ParseResult<&Scope> {
        self.stack
            .back()
            .ok_or_else(|| ParseError::UnpredictableState(self.stream.current().span))
    }

    pub fn parent(&self) -> ParseResult<&Scope> {
        self.stack
            .get(self.stack.len() - 2)
            .ok_or_else(|| ParseError::UnpredictableState(self.stream.current().span))
    }

    pub fn enter(&mut self, scope: Scope) {
        match &scope {
            Scope::Namespace(_) => {
                self.namespace_type = Some(NamespaceType::Unbraced);
            }
            Scope::BracedNamespace(_) => {
                self.namespace_type = Some(NamespaceType::Braced);
            }
            _ => {}
        }

        self.stack.push_back(scope);
        self.update_scope();
    }

    pub fn exit(&mut self) {
        self.stack.pop_back();
        self.update_scope();
    }

    fn update_scope(&mut self) {
        self.has_class_scope = self.has_class_scope();
        self.has_class_parent_scope = if self.has_class_scope {
            self.has_class_parent_scope()
        } else {
            false
        };
    }

    fn has_class_scope(&self) -> bool {
        for scope in self.stack.iter().rev() {
            match &scope {
                // we can't determine this from here, wait until we reach the classish scope.
                Scope::ArrowFunction(_) | Scope::AnonymousFunction(_) => {}
                Scope::BracedNamespace(_) | Scope::Namespace(_) | Scope::Function(_) => {
                    return false;
                }
                _ => {
                    return true;
                }
            };
        }

        false
    }

    fn has_class_parent_scope(&self) -> bool {
        for scope in self.stack.iter().rev() {
            match &scope {
                Scope::BracedNamespace(_) | Scope::Namespace(_) | Scope::Function(_) => {
                    return false;
                }
                // we don't know if the trait has a parent at this point
                // the only time that we can determine if a trait has a parent
                // is when it's used in a class.
                Scope::Trait(_) => {
                    return true;
                }
                // interfaces and enums don't have a parent.
                Scope::Interface(_) | Scope::Enum(_, _) => {
                    return false;
                }
                Scope::Class(_, _, has_parent) | Scope::AnonymousClass(has_parent) => {
                    return *has_parent;
                }
                // we can't determine this from here, wait until we reach the classish scope.
                Scope::ArrowFunction(_) | Scope::AnonymousFunction(_) | Scope::Method(_, _) => {}
            };
        }

        false
    }
}
