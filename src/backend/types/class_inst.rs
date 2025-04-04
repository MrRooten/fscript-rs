use std::{
    borrow::Cow, collections::{btree_map::Values, hash_map::Keys, HashMap}, fmt::Debug
};

use ahash::AHashMap;

use crate::backend::memory::size_alloc::FSRObjectAllocator;

use super::base::{DropObject, FSRObject, FSRValue, ObjId};

#[derive(Clone)]
pub struct FSRClassInst<'a> {
    #[allow(unused)]
    name: &'a str,
    attrs: AHashMap<&'a str, ObjId>,
}

impl Debug for FSRClassInst<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut new_hash = HashMap::new();
        for kv in &self.attrs {
            let obj = FSRObject::id_to_obj(*kv.1);
            if let FSRValue::Function(_) = &obj.value {
                continue;
            }
            new_hash.insert(kv.0, obj);
        }
        f.debug_struct("FSRClassInst")
            .field("name", &self.name)
            .field("attrs", &new_hash)
            .finish()
    }
}

impl<'a> FSRClassInst<'a> {
    pub fn new(name: &'a str) -> FSRClassInst<'a> {
        Self {
            name,
            attrs: AHashMap::new(),
        }
    }

    pub fn get_attr(&self, name: &str) -> Option<&ObjId> {
        self.attrs.get(name)
    }

    pub fn set_attr(&mut self, name: &'a str, value: ObjId) {
        if let Some(v) = self.attrs.get_mut(name) {
            let obj = FSRObject::id_to_obj(*v);
            *v = value;
        } else {
            let obj = FSRObject::id_to_obj(value);
            obj.ref_add();
            self.attrs.insert(name, value);
        }
    }

    pub fn list_attrs(&self) -> Keys<&'a str, ObjId> {
        self.attrs.keys()
    }

    pub fn iter_values(&self) -> impl Iterator<Item = &ObjId> {
        self.attrs.values()
    }

    pub fn get_cls_name(&self) -> &str {
        self.name
    }
}

impl<'a> DropObject<'a> for FSRClassInst<'a> {
    fn drop(&self, allocator: &mut FSRObjectAllocator<'a>) {
        let mut stack = vec![self];

        while let Some(s) = stack.pop() {
            for key_value in &s.attrs {
                let obj = FSRObject::id_to_obj(*key_value.1);
    
                if obj.count_ref() == 0 {
                    allocator.free(*key_value.1);
                }

                if let FSRValue::ClassInst(i) = &obj.value {
                    stack.push(i);
                }
            }
        }
    }
}
