use super::{class::FSRClass, class_inst::FSRClassInst, fn_def::FSRFn};

pub enum FSRValue {
    Integer(i64),
    Float(f64),
    String(String),
    Class(FSRClass),
    ClassInst(FSRClassInst),
    Function(FSRFn)
}

pub struct FSRObject {
    obj_id      : u64,
    value       : FSRValue
}