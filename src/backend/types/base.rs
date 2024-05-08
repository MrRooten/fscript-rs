use std::{
    cell::RefCell, rc::Rc, sync::atomic::{AtomicU64, Ordering}
};

use crate::{backend::{
    types::fn_def::FSRnE,
    vm::{runtime::{FALSE_OBJECT, FSRVM, NONE_OBJECT, TRUE_OBJECT}, thread::CallState},
}, utils::error::{FSRErrCode, FSRError}};

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

#[derive(Debug)]
pub enum FSRRetValue<'a> {
    Value(FSRObject<'a>),
    GlobalId(u64)
}

impl<'a> std::fmt::Display for FSRValue<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            FSRValue::Integer(e) => e.to_string(),
            FSRValue::Float(e) => e.to_string(),
            FSRValue::String(e) => e.to_string(),
            FSRValue::Class(_) => todo!(),
            FSRValue::ClassInst(_) => todo!(),
            FSRValue::Function(_) => todo!(),
            FSRValue::None => todo!(),
            FSRValue::Bool(e) => e.to_string(),
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug)]
pub struct FSRObject<'a> {
    pub(crate) obj_id: u64,
    pub(crate) value: FSRValue<'a>,
    pub(crate) ref_count       : AtomicU64,
    pub(crate) cls: &'a str,
}

impl<'a> Default for FSRObject<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> FSRObject<'a> {
    pub fn new() -> FSRObject<'a> {
        FSRObject {
            obj_id: 0,
            value: FSRValue::None,
            cls: "",
            ref_count: AtomicU64::new(0),
        }
    }

    pub fn set_value(&mut self, value: FSRValue<'a>) {
        self.value = value;
    }

    pub fn set_cls(&mut self, cls: &'a str) {
        self.cls = cls
    }


    pub fn get_cls_attr(&self, name: &str, vm: &FSRVM<'a>) -> Option<u64> {
        if let Some(btype) = vm.get_cls(self.cls) {
            return btype.get_attr(name);
        }
        let cls = vm.get_global_obj_by_name(self.cls);
        let cls_id = match cls {
            Some(s) => *s,
            None => return None
        };

        let cls_obj = FSRObject::id_to_obj(cls_id);
        if let FSRValue::Class(cls) = &cls_obj.value {
            return cls.get_attr(name);
        }

        None
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
                        ref_count: AtomicU64::new(0)
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
                        ref_count: AtomicU64::new(0),
                    };
                }
            }
        } else if method.eq("__str__") {
            return Self {
                obj_id: 0,
                value: FSRValue::String(self.value.to_string()),
                cls: "String",
                ref_count: AtomicU64::new(0),
            };
        }

        unimplemented!()
    }

    pub fn sp_object(id: u64) -> &'static FSRObject<'static> {
        if id == 0 {
            return unsafe { NONE_OBJECT.as_ref().unwrap() }
        }
        if id == 1 {
            return unsafe { TRUE_OBJECT.as_ref().unwrap() }
        }
        if id == 2 {
            return unsafe { FALSE_OBJECT.as_ref().unwrap() }
        }

        panic!()
    }

    pub fn ref_add(&self) {
        self.ref_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn ref_dec(&self) {
        self.ref_count.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn id_to_obj(id: u64) -> &'a FSRObject<'a> {
        if id < 1000 {
            return Self::sp_object(id);
        }
        unsafe {
            let ptr = id as *const FSRObject;
            &*ptr
        }
    }

    pub fn id_to_mut_obj(id: u64) -> &'a mut FSRObject<'a> {
        unsafe {
            let ptr = id as *mut FSRObject;
            &mut *ptr
        }
    }

    pub fn invoke_method(
        name: &str,
        args: Vec<u64>,
        stack: &mut CallState,
        vm: &FSRVM<'a>,
    ) -> Result<FSRRetValue<'a>, FSRError> {
        let self_object = Self::id_to_obj(args[0]);
        let self_method =  match self_object.get_cls_attr(name, vm) {
            Some(s) => s,
            None => {
                return Err(FSRError::new(format!("no such a method `{}`", name), FSRErrCode::NoSuchMethod))
            }
        };
        let method_object = Self::id_to_obj(self_method);
        let v = method_object.call(args, stack, vm)?;
        Ok(v)
    }

    pub fn get_attr(&self, name: &str, vm: &FSRVM<'a>) -> Option<u64> {
        if let Some(s) = self.get_cls_attr(name, vm) {
            return Some(s);
        }

        if let FSRValue::ClassInst(inst) = &self.value {
            let v = match inst.get_attr(name) {
                Some(s) => s,
                None => {
                    return None;
                }
            };
            return Some(*v);
        }

        None
    }

    pub fn call(
        &self,
        args: Vec<u64>,
        stack: &mut CallState,
        vm: &FSRVM<'a>,
    ) -> Result<FSRRetValue<'a>, FSRError> {
        if let FSRValue::Function(fn_def) = &self.value {
            return fn_def.invoke(args, stack, vm);
        }
        unimplemented!()
    }

    pub fn to_string(&self) -> FSRObject<'a> {
        return self.invoke("__str__", vec![]);
    }

    pub fn is_fsr_function(&self) -> bool {
        if let FSRValue::Function(fn_def) = &self.value {
            if let FSRnE::FSRFn(_) = &fn_def.get_def() {
                return true;
            }
        }
        
        false
    }

    pub fn is_fsr_cls(&self) -> bool {
        if let FSRValue::Class(_) = &self.value {
            return true;
        }
        
        false
    }

    pub fn get_fsr_offset(&self) -> (Rc<String>, (u64, u64)) {
        if let FSRValue::Function(fn_def) = &self.value {
            if let FSRnE::FSRFn(f) = &fn_def.get_def() {
                return (f.0.clone(), f.1);
            }
        }
        
        panic!()
    }

    pub fn get_fsr_args(&self) -> &Vec<String> {
        if let FSRValue::Function(fn_def) = &self.value {
            return fn_def.get_args();
        }

        unimplemented!()
    }

    pub fn get_fsr_class_name(&self) -> &str {
        if let FSRValue::Class(cls) = &self.value {
            return cls.get_name()
        }

        unimplemented!()
    }

}
