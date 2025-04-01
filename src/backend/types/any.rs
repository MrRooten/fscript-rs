use std::any::Any;

pub struct AnyType {
    pub value: Box<dyn Any + Send>,
}