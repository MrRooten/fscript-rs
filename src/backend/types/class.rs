use std::{
    collections::HashMap,
    sync::{atomic::{AtomicUsize, Ordering}, Arc},
};

use ahash::AHashMap;

use crate::{backend::{compiler::bytecode::BinaryOffset, types::base::{FSRRetValue, GlobalObj}, vm::{thread::FSRThreadRuntime, virtual_machine::FSRVM}}, utils::error::FSRError};

use super::{
    base::{AtomicObjId, FSRObject, FSRValue, ObjId},
    fn_def::{FSRFn, FSRRustFn, FSRnE},
};
use std::fmt::Debug;

#[repr(C)]
pub struct FSRClass {
    /// This will be set after the class object is created
    object_id: Option<ObjId>,
    pub(crate) offset_rust_fn: [Option<FSRRustFn>; 30],
    pub(crate) name: Arc<String>,
    pub(crate) attrs: AHashMap<String, AtomicObjId>,
    pub(crate) offset_attrs: Vec<Option<AtomicObjId>>,
}

impl PartialEq for FSRClass {
    fn eq(&self, other: &Self) -> bool {
        // pointer is same
        std::ptr::eq(self, other)
    }
}

impl Eq for FSRClass {
    
}

#[allow(unused)]
#[derive(Debug)]
enum TmpObject<'a> {
    Object(&'a FSRObject<'a>),
    String(String),
}

impl Debug for FSRClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut new_hash = HashMap::new();
        for kv in &self.attrs {
            let obj = FSRObject::id_to_obj(kv.1.load(Ordering::Relaxed));
            if let FSRValue::Function(f) = &obj.value {
                if f.is_fsr_function() {
                    new_hash.insert(kv.0, TmpObject::String(format!("fn `{}`", kv.0)));
                } else {
                    new_hash.insert(kv.0, TmpObject::String(f.as_str()));
                }

                continue;
            }
            new_hash.insert(kv.0, TmpObject::Object(obj));
        }
        f.debug_struct("FSRClass")
            .field("name", &self.name)
            .field("attrs", &new_hash)
            .field("offset_attrs", &"")
            .finish()
    }
}

pub fn map_err(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    if len < 2 {
        return Err(FSRError::new(
            "map requires at least 2 arguments: function and iterable",
            crate::utils::error::FSRErrCode::NotValidArgs
        ));
    }
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let may_err_object = FSRObject::id_to_obj(args[0]);
    let fn_callback = FSRObject::id_to_obj(args[1]);
    if may_err_object.cls == FSRObject::id_to_obj(GlobalObj::Exception.get_id()).as_class() {
        return fn_callback.call(&[args[0]], thread, code, args[1]);
    }

    return Ok(FSRRetValue::GlobalId(args[0]));
}

pub fn is_err(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    if len < 1 {
        return Err(FSRError::new(
            "is_err requires at least 1 argument",
            crate::utils::error::FSRErrCode::NotValidArgs
        ));
    }
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let may_err_object = FSRObject::id_to_obj(args[0]);
    if may_err_object.cls == FSRObject::id_to_obj(GlobalObj::Exception.get_id()).as_class() {
        return Ok(FSRRetValue::GlobalId(FSRObject::true_id()));
    }

    return Ok(FSRRetValue::GlobalId(FSRObject::false_id()));
}

pub fn then(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    if len < 2 {
        return Err(FSRError::new(
            "map_ok requires at least 2 arguments: function and iterable",
            crate::utils::error::FSRErrCode::NotValidArgs
        ));
    }
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let may_err_object = FSRObject::id_to_obj(args[0]);
    let fn_callback = FSRObject::id_to_obj(args[1]);
    if may_err_object.cls == FSRObject::id_to_obj(GlobalObj::Exception.get_id()).as_class() {
        // return error
        return Ok(FSRRetValue::GlobalId(args[0]));
    }

    return fn_callback.call(&[args[0]], thread, code, args[1]);
}

pub fn is_none(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    if len < 1 {
        return Err(FSRError::new(
            "is_none requires at least 1 argument",
            crate::utils::error::FSRErrCode::NotValidArgs
        ));
    }
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    if args[0] == FSRObject::none_id() {
        return Ok(FSRRetValue::GlobalId(FSRObject::true_id()));
    }

    return Ok(FSRRetValue::GlobalId(FSRObject::false_id()));
}

pub fn unwrap(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    if len < 1 {
        return Err(FSRError::new(
            "unwrap requires at least 1 argument",
            crate::utils::error::FSRErrCode::NotValidArgs
        ));
    }
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let may_err_object = FSRObject::id_to_obj(args[0]);
    if may_err_object.cls == FSRObject::id_to_obj(GlobalObj::Exception.get_id()).as_class() {
        panic!("unwrap called on an error object: {:?}", may_err_object);
    }

    return Ok(FSRRetValue::GlobalId(args[0]));
}

pub fn expect(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    if len < 1 {
        return Err(FSRError::new(
            "expect requires at least 2 arguments: message and object",
            crate::utils::error::FSRErrCode::NotValidArgs
        ));
    }
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let message_object = FSRObject::id_to_obj(args[1]);
    if let FSRValue::String(message) = &message_object.value {
        panic!("expect called with message: {}", message);
    }

    return Ok(FSRRetValue::GlobalId(args[0]));
}

impl FSRClass {
    pub fn new(name: &str) -> FSRClass {
        let mut cls = FSRClass {
            object_id: None,
            name: Arc::new(name.to_string()),
            attrs: AHashMap::new(),
            offset_attrs: vec![],
            offset_rust_fn: [None; 30],
        };

        cls
    }

    pub fn init_method(&mut self) {
        let map_err = FSRFn::from_rust_fn_static(map_err, "object_map_err");
        self.insert_attr("map_err", map_err);
        let is_err = FSRFn::from_rust_fn_static(is_err, "object_is_err");
        self.insert_attr("is_err", is_err);
        let then = FSRFn::from_rust_fn_static(then, "object_map_ok");
        self.insert_attr("then", then);
        let is_none = FSRFn::from_rust_fn_static(is_none, "object_is_none");
        self.insert_attr("is_none", is_none);
        let unwrap = FSRFn::from_rust_fn_static(unwrap, "object_unwrap");
        self.insert_attr("unwrap", unwrap);
        let expect = FSRFn::from_rust_fn_static(expect, "object_expect");
        self.insert_attr("expect", expect);
    }

    pub fn new_without_method(name: &str) -> FSRClass {
        let mut cls = FSRClass {
            name: Arc::new(name.to_string()),
            attrs: AHashMap::new(),
            offset_attrs: vec![],
            offset_rust_fn: [None; 30],
            object_id: None,
        };

        cls
    }

    pub fn insert_attr<'a>(&mut self, name: &str, object: FSRObject<'a>) {
        let obj_id = FSRVM::register_object(object);
        self.attrs.insert(name.to_string(), AtomicUsize::new(obj_id));
    }

    pub fn insert_offset_attr<'a>(&mut self, offset: BinaryOffset, object: FSRObject<'a>) {
        if self.offset_attrs.len() <= offset as usize {
            self.offset_attrs.resize_with(offset as usize + 1, || None);
        }

        if let FSRValue::Function(f) = &object.value {
            if let FSRnE::RustFn(rust_fn) = &f.fn_def {
                self.offset_rust_fn[offset as usize] = Some(rust_fn.1);
            }
        }
        let obj_id = FSRVM::register_object(object);
        self.attrs
            .insert(offset.alias_name().to_string(), AtomicUsize::new(obj_id));
        self.offset_attrs[offset as usize] = Some(AtomicUsize::new(obj_id));
    }

    pub fn set_object_id(&mut self, id: ObjId) {
        self.object_id = Some(id);
    }

    #[inline(always)]
    pub fn get_rust_fn(&self, offset: BinaryOffset) -> Option<FSRRustFn> {
        // self.offset_rust_fn.get(offset as usize).and_then(|s| s.as_ref())
        self.offset_rust_fn[offset as usize]
    }

    pub fn insert_offset_attr_obj_id(&mut self, offset: BinaryOffset, id: ObjId) {
        if self.offset_attrs.len() <= offset as usize {
            self.offset_attrs.resize_with(offset as usize + 1, || None);
        }

        self.offset_attrs[offset as usize] = Some(AtomicUsize::new(id));
        self.insert_attr_id(offset.alias_name(), id);
    }

    pub fn insert_attr_id(&mut self, name: &str, obj_id: ObjId) {
        if let Some(v) = self.attrs.get_mut(name) {
            v.store(obj_id, Ordering::Relaxed);
        } else {
            self.attrs.insert(name.to_string(), AtomicUsize::new(obj_id));
        }
    }

    pub fn get_attr(&self, name: &str) -> Option<&AtomicObjId> {
        self.attrs.get(name)
    }

    pub fn iter_values(&self) -> impl Iterator<Item = &AtomicObjId> {
        self.attrs.values()
    }

    #[inline]
    pub fn get_offset_attr(&self, offset: BinaryOffset) -> Option<&AtomicObjId> {
        let s = self.offset_attrs.get(offset as usize)?;
        if s.is_none() {
            return None;
        }

        s.as_ref()
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn try_get_offset_attr(&self, offset: BinaryOffset) -> Option<&AtomicObjId> {
        match self.get_offset_attr(offset) {
            Some(s) => Some(s),
            None => self.get_attr(offset.alias_name()),
        }
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_arc_name(&self) -> Arc<String> {
        self.name.clone()
    }
}
