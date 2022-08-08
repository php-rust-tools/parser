# Notes

```rust
pub enum PhpValue {
    Int(i64),
    Array(Vec<GBox<PhpValue>>)
}

impl Trace<Self> for PhpValue {
    fn trace(&self, tracer: Tracer) {
        match self {
            Self::Array(items) => {
                for item in items {
                    item.trace(tracer);
                }
            },
            _ => {},
        }
    }
}

let mut gc: Gc<PhpValue> = Gc::new();

let one = gc.hold(PhpValue::Int(1));

gc.root(PhpValue::Array(vec![
    one,
]));

gc.collect();
```