# Interner

This crate provides a simple string interner through the `Interner` structure. This structure allows you to retrieve a unique `Symbol` to a `&str`.

## Usage

```rust
use trunk_interner::{Interner, Symbol};

fn main() {
    let mut interner = Interner::default();

    let my_string_symbol: Symbol = interner.intern("Hello, world!");
    let my_original_string: &str = interner.get(my_string_symbol);
}
```

If a `&str` is interned multiple times, the same `Symbol` will be returned, in theory minimizing the amount of memory used by your program due to a reduced number of `String` allocations.