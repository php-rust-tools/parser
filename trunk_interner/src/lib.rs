use std::collections::HashMap;

/// A wrapper-type pointing to a unique string in the [`Interner`].
#[derive(Debug, PartialEq)]
#[repr(transparent)]
pub struct Symbol(u32);

/// The unoptimized singleton used for interning strings.
/// 
/// The structure uses a standard `HashMap` to map strings to indexes,
/// where those indexes point to a `String` inside of a `Vec<String>`.
/// 
/// The design of the interner isn't optimal right now since each
/// intern results into 2 allocations.
#[derive(Debug, Default)]
pub struct Interner {
    map: HashMap<String, u32>,
    storage: Vec<String>,
}

impl Interner {
    /// Intern a `&str` and retrieve a unique [`Symbol`].
    pub fn intern(&mut self, name: &str) -> Symbol {
        if let Some(&index) = self.map.get(name) {
            return Symbol(index);
        }

        let index = self.map.len() as u32;

        self.map.insert(name.to_string(), index);
        self.storage.push(name.to_string());

        Symbol(index)
    }

    /// Retrieve tyhe `&str` for a given [`Symbol`].
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