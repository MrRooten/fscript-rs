use std::{
    borrow::Borrow,
    cell::{Cell, Ref, RefCell},
    collections::HashMap,
};

use crate::backend::{
    types::string::FSRString,
    vm::{runtime::FSRVM, thread::CallState},
};

use super::{class::FSRClass, class_inst::FSRClassInst, fn_def::FSRFn};

#[derive(Debug, Clone)]
pub enum FSRValue<'a> {
    Integer(i64),
    Float(f64),
    String(String),
    Class(FSRClass<'a>),
    ClassInst(FSRClassInst<'a>),
    Function(FSRFn),
    Bool(bool),
    None,
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
            FSRValue::Bool(e) => e.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FSRObject<'a> {
    pub(crate) obj_id: u64,
    pub(crate) value: FSRValue<'a>,
    pub(crate) cls: &'a str,
}

impl<'a> FSRObject<'a> {
    pub fn new() -> FSRObject<'a> {
        FSRObject {
            obj_id: 0,
            value: FSRValue::None,
            cls: "",
        }
    }

    pub fn set_value(&mut self, value: FSRValue<'a>) {
        self.value = value;
    }

    pub fn set_cls(&mut self, cls: &'a str) {
        self.cls = cls
    }

    pub fn get_cls_attr(&self, name: &str, vm: &FSRVM<'a>) -> Option<u64> {
        let cls = vm.get_cls(self.cls).unwrap();
        return cls.get_attr(name);
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
                    };
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
                    };
                }
            }
        } else if method.eq("__str__") {
            return Self {
                obj_id: 0,
                value: FSRValue::String(self.value.to_string()),
                cls: "String",
            };
        }

        unimplemented!()
    }

    pub fn invoke_method(
        name: &str,
        args: Vec<Ref<FSRObject<'a>>>,
        stack: &mut CallState,
        vm: &FSRVM<'a>,
    ) -> Result<FSRObject<'a>, ()> {
        let self_method = args[0].get_cls_attr(name, vm).unwrap();
        let method_object = vm.get_obj_by_id(&self_method).unwrap().borrow();
        let v = method_object.call(args, stack, vm)?;
        return Ok(v);
    }

    pub fn get_attr(&self, name: &str, vm: &FSRVM<'a>) -> Option<u64> {
        if let Some(s) = self.get_cls_attr(name, vm) {
            return Some(s);
        }

        if let FSRValue::ClassInst(inst) = &self.value {
            return Some(inst.get_attr(name).unwrap().clone());
        }

        unimplemented!()
    }

    pub fn call(
        &self,
        args: Vec<Ref<FSRObject<'a>>>,
        stack: &mut CallState,
        vm: &FSRVM<'a>,
    ) -> Result<FSRObject<'a>, ()> {
        if let FSRValue::Function(fn_def) = &self.value {
            return fn_def.invoke(args, stack, vm);
        }
        unimplemented!()
    }

    pub fn to_string(&self) -> FSRObject<'a> {
        return self.invoke("__str__", vec![]);
    }
}
