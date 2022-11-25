use hashbrown::HashMap;
use std::cell::RefCell;

use super::value::Value;

#[derive(Clone, Debug)]
pub struct Environment {
    entries: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.entries.get(name) {
            return Some(value.clone());
        }

        return None;
    }

    pub fn set(&mut self, name: &str, value: Value) {
        if let Some(current) = self.entries.get_mut(name) {
            *current = value;
        } else {
            self.entries.insert(name.to_owned(), value);
        }
    }
}
