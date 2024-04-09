use std::{borrow::Borrow, cell::{Cell, RefCell}};

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

    pub fn invoke(&self, method: &str, args: Vec<&RefCell<FSRObject<'a>>>) -> FSRObject<'a> {
        if method.eq("__add__") {
            let other = args[0];
            let v = self as *const FSRObject<'a> as *mut Self;
            if other.as_ptr() == v {
                if let FSRValue::Integer(i) = self.value {

                    let v = FSRValue::Integer(i + i);
                    return Self {
                        obj_id: 0,
                        value: v,
                    }
                    
                }
            }
            let other = other.borrow();
            if let FSRValue::Integer(i) = self.value {
                if let FSRValue::Integer(o_i) = other.value {
                    let v = FSRValue::Integer(i + o_i);
                    return Self {
                        obj_id: 0,
                        value: v,
                    }
                }
            }
        }
        unimplemented!()
    }

    pub fn call(&self, args: Vec<&RefCell<FSRObject<'a>>>) -> FSRObject<'a> {
        unimplemented!()
    }
}

