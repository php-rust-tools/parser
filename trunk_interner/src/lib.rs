use std::collections::HashMap;

#[derive(Debug, PartialEq)]
#[repr(transparent)]
pub struct Symbol(u32);

#[derive(Debug, Default)]
pub struct Interner {
    map: HashMap<String, u32>,
    storage: Vec<String>,
}

impl Interner {
    pub fn intern(&mut self, name: &str) -> Symbol {
        if let Some(&index) = self.map.get(name) {
            return Symbol(index);
        }

        let index = self.map.len() as u32;

        self.map.insert(name.to_string(), index);
        self.storage.push(name.to_string());

        Symbol(index)
    }

    pub fn get(&self, symbol: Symbol) -> &str {
        self.storage[symbol.0 as usize].as_str()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Interner, Symbol};


    #[test]
    fn it_can_intern_a_string() {
        let mut interner = Interner::default();

        assert_eq!(interner.intern("Hello, world!"), Symbol(0));
        assert_eq!(interner.intern("Hello, world!"), Symbol(0));
    }

    #[test]
    fn it_can_retrieve_an_interned_string() {
        let mut interner = Interner::default();
        let symbol = interner.intern("Hello, world!");

        assert_eq!(interner.get(symbol), "Hello, world!");
    }
}