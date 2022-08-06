mod value;
mod add;
mod sub;
mod mul;
mod div;
mod rem;
mod type_error;

pub use value::*;
pub use type_error::TypeError;

#[cfg(test)]
mod tests {
    use crate::PhpValue;

    #[test]
    fn values_can_be_added() {
        assert_eq!(PhpValue::Int(1) + PhpValue::Int(2), Ok(PhpValue::Int(3)));
        assert_eq!(PhpValue::Float(1.5) + PhpValue::Float(2.5), Ok(PhpValue::Float(4.0)));
        assert_eq!(PhpValue::Float(1.5) + PhpValue::Int(1), Ok(PhpValue::Float(2.5)));
        assert_eq!(PhpValue::Int(1) + PhpValue::Float(1.5), Ok(PhpValue::Float(2.5)));
        assert_eq!(PhpValue::String("1".into()) + PhpValue::Int(1), Ok(PhpValue::Int(2)));
        assert_eq!(PhpValue::String("1".into()) + PhpValue::Float(1.0), Ok(PhpValue::Float(2.0)));
    }
}