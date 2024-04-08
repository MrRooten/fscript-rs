use super::{class::FSRClass, class_inst::FSRClassInst, fn_def::FSRFn};

pub enum FSRValue<'a> {
    Integer(i64),
    Float(f64),
    String(String),
    Class(FSRClass),
    ClassInst(FSRClassInst<'a>),
    Function(FSRFn),
    None
}

pub struct FSRObject<'a> {
    pub(crate) obj_id      : u64,
    pub(crate) value       : FSRValue<'a>
}

impl<'a> FSRObject<'a> {
    pub fn set_value(&mut self, value: FSRValue<'a>) {
        self.value = value;
    }
}

