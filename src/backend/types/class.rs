use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

use ahash::AHashMap;

use crate::backend::{compiler::bytecode::BinaryOffset, vm::virtual_machine::FSRVM};

use super::{
    base::{AtomicObjId, FSRObject, FSRValue, ObjId},
    fn_def::{FSRFn, FSRRustFn, FSRnE},
};
use std::fmt::Debug;

#[repr(C)]
pub struct FSRClass<'a> {
    pub(crate) offset_rust_fn: [Option<FSRRustFn>; 30],
    pub(crate) name: String,
    pub(crate) attrs: AHashMap<&'a str, AtomicObjId>,
    pub(crate) offset_attrs: Vec<Option<AtomicObjId>>,
}

impl PartialEq for FSRClass<'_> {
    fn eq(&self, other: &Self) -> bool {
        // pointer is same
        std::ptr::eq(self, other)
    }
}

impl Eq for FSRClass<'_> {
    
}

#[allow(unused)]
#[derive(Debug)]
enum TmpObject<'a> {
    Object(&'a FSRObject<'a>),
    String(String),
}

impl Debug for FSRClass<'_> {
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

impl<'a> FSRClass<'a> {
    pub fn new(name: &'a str) -> FSRClass<'a> {
        FSRClass {
            name: name.to_string(),
            attrs: AHashMap::new(),
            offset_attrs: vec![],
            offset_rust_fn: [None; 30],
        }
    }

    pub fn insert_attr(&mut self, name: &'a str, object: FSRObject<'a>) {
        let obj_id = FSRVM::register_object(object);
        self.attrs.insert(name, AtomicUsize::new(obj_id));
    }

    pub fn insert_offset_attr(&mut self, offset: BinaryOffset, object: FSRObject<'a>) {
        if self.offset_attrs.len() <= offset as usize {
            self.offset_attrs.resize_with(offset as usize + 1, || None);
        }

        if let FSRValue::Function(f) = &object.value {
            if let FSRnE::RustFn(rust_fn) = &f.fn_def {
                // self.offset_rust_fn.push(Some(rust_fn.1));
                // } else {
                //     self.offset_rust_fn.push(None);
                // }
                // self.offset_rust_fn
                //     .resize_with(offset as usize + 1, || None);
                self.offset_rust_fn[offset as usize] = Some(rust_fn.1);
            }
        }
        let obj_id = FSRVM::register_object(object);
        self.attrs
            .insert(offset.alias_name(), AtomicUsize::new(obj_id));
        self.offset_attrs[offset as usize] = Some(AtomicUsize::new(obj_id));
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

    pub fn insert_attr_id(&mut self, name: &'a str, obj_id: ObjId) {
        if let Some(v) = self.attrs.get_mut(name) {
            v.store(obj_id, Ordering::Relaxed);
        } else {
            self.attrs.insert(name, AtomicUsize::new(obj_id));
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
}
