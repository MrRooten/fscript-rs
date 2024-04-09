use std::{borrow::Borrow, cell::{Cell, RefCell}, collections::HashMap};

use crate::backend::{types::string::FSRString, vm::{runtime::FSRVM, thread::CallState}};

use super::{class::FSRClass, class_inst::FSRClassInst, fn_def::FSRFn};

#[derive(Debug)]
pub enum FSRValue<'a> {
    Integer(i64),
    Float(f64),
    String(String),
    Class(FSRClass<'a>),
    ClassInst(FSRClassInst<'a>),
    Function(FSRFn),
    None
}

impl<'a> FSRValue<'a> {
    pub fn to_string(&self) -> String {
        match self {
            FSRValue::Integer(e) => e.to_string(),
            FSRValue::Float(e) => e.to_string(),
            FSRValue::String(e) => e.to_string(),
            FSRValue::Class(_) => todo!(),
            FSRValue::ClassInst(_) => todo!(),
            FSRValue::Function(_) => todo!(),
            FSRValue::None => todo!(),
        }
    }
}

#[derive(Debug)]
pub struct FSRObject<'a> {
    pub(crate) obj_id                   : u64,
    pub(crate) value                    : FSRValue<'a>,
    pub(crate) cls                      : &'a str,
    pub(crate) attrs                    : HashMap<&'a str, u64>
}


impl<'a> FSRObject<'a> {
    pub fn new() -> FSRObject<'a> {
        FSRObject {
            obj_id: 0,
            value: FSRValue::None,
            cls: "",
            attrs: HashMap::new(),
        }
    }

    pub fn set_value(&mut self, value: FSRValue<'a>) {
        self.value = value;
    }

    pub fn set_cls(&mut self, cls: &'a str) {
        self.cls = cls
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
                        cls: "Integer",
                        attrs: HashMap::new(),
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
                        cls: "Integer",
                        attrs: HashMap::new(),
                    }
                }
            }
        }
        else if method.eq("__str__") {
            return Self {
                obj_id: 0,
                value: FSRValue::String(self.value.to_string()),
                cls: "String",
                attrs: HashMap::new(),
            }
        }
        unimplemented!()
    }

    pub fn call(&self, args: Vec<u64>, stack: &'a mut CallState, vm: &'a mut FSRVM<'a>) -> Result<u64,()> {
        if let FSRValue::Function(fn_def) = &self.value {
            return fn_def.invoke(args, stack, vm);
        }
        unimplemented!()
    }

    pub fn to_string(&self) -> FSRObject<'a> {
        return self.invoke("__str__", vec![]);
    }
}

