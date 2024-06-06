use std::{
    borrow::Cow,
    collections::hash_map::Keys,
    fmt::Debug,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{
    backend::{
        compiler::bytecode::BinaryOffset,
        types::fn_def::FSRnE,
        vm::{
            runtime::{FSRVM, OBJECTS},
            thread::FSRThreadRuntime,
        },
    },
    utils::error::{FSRErrCode, FSRError},
};

use super::{
    class::FSRClass, class_inst::FSRClassInst, fn_def::FSRFn, iterator::FSRInnerIterator,
    list::FSRList, string::FSRString,
};

pub enum FSRGlobalObjId {
    None = 0,
    True = 1,
    False = 2,
    IntegerCls = 3,
    FnCls = 4,
    InnerIterator = 5,
    ListCls = 6,
    StringCls = 7,
    ClassCls = 8,
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
    Iterator(FSRInnerIterator),
    None,
}

#[derive(Debug)]
pub enum FSRRetValue<'a> {
    Value(Box<FSRObject<'a>>),
    GlobalId(u64),
    GlobalIdTemp(u64)
}

impl<'a> FSRValue<'a> {
    fn inst_to_string(
        inst: &FSRClassInst,
        self_id: u64,
        thread: &mut FSRThreadRuntime<'a>,
    ) -> Option<Cow<'a, str>> {
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
            let ret = obj.call(&vec![self_id], thread);
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

    fn to_string(&self, self_id: u64, thread: &mut FSRThreadRuntime<'a>) -> Option<Cow<str>> {
        let s = match self {
            FSRValue::Integer(e) => Some(Cow::Owned(e.to_string())),
            FSRValue::Float(e) => Some(Cow::Owned(e.to_string())),
            FSRValue::String(e) => Some(e.clone()),
            FSRValue::Class(_) => None,
            FSRValue::ClassInst(inst) => Self::inst_to_string(inst, self_id, thread),
            FSRValue::Function(_) => None,
            FSRValue::None => Some(Cow::Borrowed("None")),
            FSRValue::Bool(e) => Some(Cow::Owned(e.to_string())),
            FSRValue::List(_) => {
                let res = FSRObject::invoke_method("__str__", &vec![self_id], thread).unwrap();
                match res {
                    FSRRetValue::Value(v) => {
                        if let FSRValue::String(s) = v.value {
                            return Some(s);
                        }
                        return None;
                    }
                    FSRRetValue::GlobalId(id) => {
                        let obj = FSRObject::id_to_obj(id);
                        if let FSRValue::String(s) = &obj.value {
                            return Some(s.clone());
                        }

                        return None;
                    }
                    FSRRetValue::GlobalIdTemp(id) => {
                        let obj = FSRObject::id_to_obj(id);
                        if let FSRValue::String(s) = &obj.value {
                            return Some(s.clone());
                        }

                        FSRObject::drop_object(id);
                        return None;
                    },
                }
            }
            FSRValue::Iterator(_) => None,
        };

        s
    }
}


pub struct FSRObject<'a> {
    pub(crate) obj_id: u64,
    pub(crate) value: FSRValue<'a>,
    pub(crate) ref_count: AtomicU64,
    pub(crate) cls: u64,
}

impl Debug for FSRObject<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cls = self.cls;
        let obj = FSRObject::id_to_obj(cls);
        let cls = match &obj.value {
            FSRValue::Class(c) => c,
            _ => {
                unimplemented!()
            }
        };
        f.debug_struct("FSRObject")
            .field("obj_id", &self.obj_id)
            .field("value", &self.value)
            .field("ref_count", &self.ref_count)
            .field("cls", cls)
            .finish()
    }
}

impl Clone for FSRObject<'_> {
    fn clone(&self) -> Self {
        // Only use for SValue like tempory value
        Self {
            obj_id: 0,
            value: self.value.clone(),
            ref_count: AtomicU64::new(0),
            cls: self.cls,
        }
    }
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
            cls: 0,
            ref_count: AtomicU64::new(0),
        }
    }

    pub fn is_true_id(&self) -> u64 {
        if let FSRValue::None = self.value {
            return 2
        }

        if let FSRValue::Bool(b) = self.value {
            if b {
                return 1
            } else {
                return 2
            }
        }

        1
    }

    pub fn set_value(&mut self, value: FSRValue<'a>) {
        self.value = value;
    }

    pub fn set_cls(&mut self, cls: u64) {
        self.cls = cls
    }

    pub fn as_string(&self) -> &str {
        if let FSRValue::String(s) = &self.value {
            return s;
        }
        unimplemented!()
    }

    pub fn set_attr(&mut self, name: &'a str, obj_id: u64) {
        if let FSRValue::ClassInst(inst) = &mut self.value {
            inst.set_attr(name, obj_id);
            return;
        }

        unimplemented!()
    }

    pub fn as_class(&self) -> &FSRClass {
        if let FSRValue::Class(cls) = &self.value {
            return cls;
        }

        unimplemented!()
    }

    #[inline]
    pub fn obj_to_id(obj: &FSRObject<'a>) -> u64 {
        obj as *const Self as u64
    }

    pub fn get_cls_attr(&self, name: &str, vm: &FSRVM<'a>) -> Option<u64> {
        if let Some(btype) = vm.get_base_cls(self.cls) {
            return btype.get_attr(name);
        }

        let cls_obj = FSRObject::id_to_obj(self.cls);
        if let FSRValue::Class(cls) = &cls_obj.value {
            return cls.get_attr(name);
        }

        None
    }

    #[inline]
    pub fn get_cls_offset_attr(&self, offset: BinaryOffset, vm: &FSRVM<'a>) -> Option<u64> {
        if let Some(btype) = vm.get_base_cls(self.cls) {
            return btype.get_offset_attr(offset);
        }

        let cls_obj = FSRObject::id_to_obj(self.cls);
        if let FSRValue::Class(cls) = &cls_obj.value {
            return cls.get_offset_attr(offset);
        }

        None
    }

    #[inline(always)]
    fn sp_object(id: u64) -> &'static FSRObject<'static> {
        unsafe {
            if let Some(obj) = OBJECTS.get(id as usize) {
                return obj;
            }
        }

        panic!()
    }

    #[inline(always)]
    pub fn is_sp_object(id: u64) -> bool {
        id < 1000
    }

    #[inline]
    pub fn ref_add(&self) {
        if Self::is_sp_object(self.obj_id) {
            return;
        }
        self.ref_count.fetch_add(1, Ordering::AcqRel);
    }

    #[inline]
    pub fn ref_dec(&self) {
        if Self::is_sp_object(self.obj_id) {
            return;
        }
        self.ref_count.fetch_sub(1, Ordering::AcqRel);

        if self.count_ref() == 0 {
            // Self::drop_object(self.obj_id)
            // println!("Drop self: {:?}", self);
        }
    }

    pub fn drop_object(id: u64) {
        let _cleanup = unsafe { Box::from_raw(id as *mut Self) };
    }

    #[inline(always)]
    pub fn count_ref(&self) -> u64 {
        unsafe { *self.ref_count.as_ptr() }
    }

    #[inline(always)]
    pub fn id_to_obj(id: u64) -> &'a FSRObject<'a> {
        if id < 1000 {
            return Self::sp_object(id);
        }
        unsafe {
            let ptr = id as *const FSRObject;
            &*ptr
        }
    }

    #[inline(always)]
    pub fn id_to_mut_obj(id: u64) -> &'a mut FSRObject<'a> {
        unsafe {
            let ptr = id as *mut FSRObject;
            &mut *ptr
        }
    }

    pub fn invoke_method(
        name: &str,
        args: &Vec<u64>,
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
    pub fn invoke_offset_method(
        offset: BinaryOffset,
        args: &Vec<u64>,
        thread: &mut FSRThreadRuntime<'a>,
    ) -> Result<FSRRetValue<'a>, FSRError> {
        let self_object = Self::id_to_obj(args[0]);
        if let Some(self_method) = self_object.get_cls_offset_attr(offset, thread.get_vm()) {
            let method_object = Self::id_to_obj(self_method);
            let v = method_object.call(args, thread)?;
            return Ok(v);
        }

        let self_method = match self_object.get_cls_attr(offset.alias_name(), thread.get_vm()) {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    format!("no such a method `{:?}`", offset),
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

    #[inline(always)]
    pub fn is_true(&self) -> bool {
        self.obj_id == FSRGlobalObjId::True as u64
    }

    #[inline(always)]
    pub fn is_false(&self) -> bool {
        self.obj_id == FSRGlobalObjId::False as u64
    }

    #[inline(always)]
    pub fn is_none(&self) -> bool {
        self.obj_id == FSRGlobalObjId::None as u64
    }

    pub fn call(
        &'a self,
        args: &Vec<u64>,
        thread: &mut FSRThreadRuntime<'a>,
    ) -> Result<FSRRetValue, FSRError> {
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
        let v = FSRObject::id_to_obj(self.cls);
        if let FSRValue::Class(c) = &v.value {
            return FSRString::new_inst(Cow::Owned(format!(
                "<`{}` Object at {:?}>",
                c.get_name(),
                self as *const Self
            )));
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

    pub fn get_fsr_offset(&self) -> (&Cow<str>, (usize, usize)) {
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
