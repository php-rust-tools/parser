use std::any::Any;

pub trait Node: Any {
    fn children(&self) -> Vec<&dyn Node>;
}