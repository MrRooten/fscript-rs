use std::{
    borrow::Cow, cell::RefCell, collections::hash_map::Keys, rc::Rc, sync::atomic::{AtomicU64, Ordering}
};

use crate::{
    backend::{
        types::fn_def::FSRnE,
        vm::{
            runtime::{FALSE_OBJECT, FSRVM, NONE_OBJECT, TRUE_OBJECT},
            thread::FSRThreadRuntime,
        },
    },
    utils::error::{FSRErrCode, FSRError},
};

use super::{
    class::FSRClass, class_inst::FSRClassInst, fn_def::FSRFn, list::FSRList, string::FSRString,
};

pub enum FSRGlobalObjId {
    None = 0,
    True = 1,
    False = 2,
}

#[derive(Debug, Clone)]
pub enum FSRValue<'a> {
    Integer(i64),
    Float(f64),
    String(Cow<'a, str>),
    Class(FSRClass<'a>),
    ClassInst(FSRClassInst<'a>),
    Function(FSRFn<'a>),
    Bool(bool),
    List(FSRList),
    None,
}

#[derive(Debug)]
pub enum FSRRetValue<'a> {
    Value(FSRObject<'a>),
    GlobalId(u64),
}

impl<'a> FSRValue<'a> {
    fn to_string(&self, self_id: u64, thread: &mut FSRThreadRuntime<'a>) -> Option<Cow<str>> {
        let s = match self {
            FSRValue::Integer(e) => Some(Cow::Owned(e.to_string())),
            FSRValue::Float(e) => Some(Cow::Owned(e.to_string())),
            FSRValue::String(e) => Some(e.clone()),
            FSRValue::Class(_) => None,
            FSRValue::ClassInst(inst) => {
                let vm = thread.get_vm();
                let cls = match vm.get_global_obj_by_name(inst.get_cls_name()) {
                    Some(s) => s,
                    None => {
                        return None;
                    }
                };
                let cls = FSRObject::id_to_obj(*cls);
                let cls = cls.as_class();
                let v = cls.get_attr("__str__");
                if let Some(obj_id) = v {
                    let obj = FSRObject::id_to_obj(obj_id);
                    let ret = obj.call(vec![self_id], thread);
                    let ret_value = match ret {
                        Ok(o) => o,
                        Err(_) => {
                            return None;
                        }
                    };

                    if let FSRRetValue::Value(v) = ret_value {
                        return Some(Cow::Owned(v.as_string().to_string()));
                    }

                    if let FSRRetValue::GlobalId(id) = ret_value {
                        let obj = FSRObject::id_to_obj(id);
                        if let FSRValue::String(s) = &obj.value {
                            return Some(Cow::Borrowed(s));
                        }
                    }
                }
                None
            }
            FSRValue::Function(_) => None,
            FSRValue::None => None,
            FSRValue::Bool(e) => Some(Cow::Owned(e.to_string())),
            FSRValue::List(_) => {
                let res = FSRObject::invoke_method("__str__", vec![self_id], thread).unwrap();
                match res {
                    FSRRetValue::Value(v) => {
                        if let FSRValue::String(s) = v.value {
                            return Some(s)
                        }
                        return None
                    },
                    FSRRetValue::GlobalId(id) => {
                        let obj = FSRObject::id_to_obj(id);
                        if let FSRValue::String(s) = &obj.value {
                            return Some(s.clone());
                        }

                        return None
                    },
                }

            },
        };

        s
    }
}

#[derive(Debug)]
pub struct FSRObject<'a> {
    pub(crate) obj_id: u64,
    pub(crate) value: FSRValue<'a>,
    pub(crate) ref_count: AtomicU64,
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

    pub fn as_string(&self) -> &str {
        if let FSRValue::String(s) = &self.value {
            return s;
        }
        unimplemented!()
    }

    pub fn as_class(&self) -> &FSRClass {
        if let FSRValue::Class(cls) = &self.value {
            return cls;
        }

        unimplemented!()
    }

    pub fn get_cls_attr(&self, name: &str, vm: &FSRVM<'a>) -> Option<u64> {
        if let Some(btype) = vm.get_cls(self.cls) {
            return btype.get_attr(name);
        }
        let cls = vm.get_global_obj_by_name(self.cls);
        let cls_id = match cls {
            Some(s) => *s,
            None => return None,
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
                        ref_count: AtomicU64::new(0),
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
        }

        unimplemented!()
    }

    pub fn sp_object(id: u64) -> &'static FSRObject<'static> {
        if id == 0 {
            return unsafe { NONE_OBJECT.as_ref().unwrap() };
        }
        if id == 1 {
            return unsafe { TRUE_OBJECT.as_ref().unwrap() };
        }
        if id == 2 {
            return unsafe { FALSE_OBJECT.as_ref().unwrap() };
        }

        panic!()
    }

    #[inline]
    pub fn ref_add(&self) {
        self.ref_count.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn ref_dec(&self) {
        self.ref_count.fetch_sub(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn id_to_obj(id: u64) -> &'a FSRObject<'a> {
        if id < 1000 {
            return Self::sp_object(id);
        }
        unsafe {
            let ptr = id as *const FSRObject;
            &*ptr
        }
    }

    #[inline]
    pub fn id_to_mut_obj(id: u64) -> &'a mut FSRObject<'a> {
        unsafe {
            let ptr = id as *mut FSRObject;
            &mut *ptr
        }
    }

    pub fn invoke_method(
        name: &str,
        args: Vec<u64>,
        thread: &mut FSRThreadRuntime<'a>,
    ) -> Result<FSRRetValue<'a>, FSRError> {
        let self_object = Self::id_to_obj(args[0]);
        let self_method = match self_object.get_cls_attr(name, thread.get_vm()) {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    format!("no such a method `{}`", name),
                    FSRErrCode::NoSuchMethod,
                ))
            }
        };
        let method_object = Self::id_to_obj(self_method);
        let v = method_object.call(args, thread)?;
        Ok(v)
    }

    #[inline]
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

    pub fn list_attrs(&self) -> Keys<&'a str, u64> {
        if let FSRValue::ClassInst(inst) = &self.value {
            return inst.list_attrs();
        }

        unimplemented!()
    }

    #[inline]
    pub fn is_true(&self) -> bool {
        self.obj_id == FSRGlobalObjId::True as u64
    }

    #[inline]
    pub fn is_false(&self) -> bool {
        self.obj_id == FSRGlobalObjId::False as u64
    }

    #[inline]
    pub fn is_none(&self) -> bool {
        self.obj_id == FSRGlobalObjId::None as u64
    }

    pub fn call(
        &'a self,
        args: Vec<u64>,
        thread: &mut FSRThreadRuntime<'a>,
    ) -> Result<FSRRetValue<'a>, FSRError> {
        if let FSRValue::Function(fn_def) = &self.value {
            return fn_def.invoke(args, thread);
        }
        unimplemented!()
    }

    pub fn get_self_id(&self) -> u64 {
        self as *const Self as u64
    }

    pub fn to_string(&'a self, thread: &mut FSRThreadRuntime<'a>) -> FSRObject<'a> {
        let s = self.value.to_string(self.obj_id, thread);
        if let Some(s) = s {
            return FSRString::new_inst(s);
        }

        return FSRString::new_inst(Cow::Owned(format!(
            "<`{}` Object at {:?}>",
            self.cls, self as *const Self
        )));
        //return self.invoke("__str__", vec![]);
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

    pub fn get_fsr_offset(&self) -> (Rc<String>, (usize, usize)) {
        if let FSRValue::Function(fn_def) = &self.value {
            if let FSRnE::FSRFn(f) = &fn_def.get_def() {
                return (f.get_name(), f.get_ip());
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
            return cls.get_name();
        }

        unimplemented!()
    }
}
