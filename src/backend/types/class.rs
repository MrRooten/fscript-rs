use std::collections::HashMap;

use ahash::AHashMap;

use crate::backend::{compiler::bytecode::BinaryOffset, vm::virtual_machine::FSRVM};

use super::base::{FSRObject, FSRValue, ObjId};
use std::fmt::Debug;

#[derive(Clone)]
pub struct FSRClass<'a> {
    pub(crate) name: &'a str,
    pub(crate) attrs: AHashMap<&'a str, ObjId>,
    pub(crate) offset_attrs: Vec<ObjId>,
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
            let obj = FSRObject::id_to_obj(*kv.1);
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
            name,
            attrs: AHashMap::new(),
            offset_attrs: vec![0; 30],
        }
    }

    pub fn insert_attr(&mut self, name: &'a str, object: FSRObject<'a>) {
        let obj_id = FSRVM::register_object(object);
        self.attrs.insert(name, obj_id);
    }

    pub fn insert_offset_attr(&mut self, offset: BinaryOffset, object: FSRObject<'a>) {
        let obj_id = FSRVM::register_object(object);
        self.offset_attrs[offset as usize] = obj_id;
    }

    pub fn insert_attr_id(&mut self, name: &'a str, obj_id: ObjId) {
        self.attrs.insert(name, obj_id);
    }

    pub fn get_attr(&self, name: &str) -> Option<ObjId> {
        self.attrs.get(name).copied()
    }

    pub fn iter_values(&self) -> impl Iterator<Item = &ObjId> {
        self.attrs.values()
    }

    #[inline]
    pub fn get_offset_attr(&self, offset: BinaryOffset) -> Option<ObjId> {
        let s = unsafe { self.offset_attrs.get_unchecked(offset as usize) };
        if s == &0 {
            return None;
        }

        return Some(*s);
    }

    #[inline(always)]
    pub fn try_get_offset_attr(&self, offset: BinaryOffset) -> Option<ObjId> {
        match self.get_offset_attr(offset) {
            Some(s) => Some(s),
            None => self.get_attr(offset.alias_name()),
        }
    }

    pub fn get_name(&self) -> &str {
        self.name
    }
}
