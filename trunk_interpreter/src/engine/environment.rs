use std::{cell::RefCell, collections::HashMap};

use super::value::Value;

#[derive(Clone, Debug)]
pub struct Environment {
    entries: RefCell<HashMap<String, Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            entries: RefCell::new(HashMap::new()),
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        let entries = self.entries.borrow();

        if let Some(value) = entries.get(name) {
            return Some(value.clone());
        }

        return None;
    }

    pub fn set(&mut self, name: &str, value: Value) {
        let mut entries = self.entries.borrow_mut();

        if let Some(current) = entries.get_mut(name) {
            *current = value;
        } else {
            entries.insert(name.to_owned(), value);
        }
    }
}