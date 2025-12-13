use std::{
    collections::{hash_map::Keys, HashMap},
    fmt::Debug, sync::Arc,
};

use ahash::AHashMap;

use super::base::{AtomicObjId, FSRObject, FSRValue, ObjId};

pub struct FSRClassInst<'a> {
    #[allow(unused)]
    name: Arc<String>,
    attrs: AHashMap<&'a str, AtomicObjId>,
}

impl Debug for FSRClassInst<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut new_hash = HashMap::new();
        for kv in &self.attrs {
            let obj = FSRObject::id_to_obj(kv.1.load(std::sync::atomic::Ordering::Relaxed));
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
    pub fn new(name: Arc<String>) -> FSRClassInst<'a> {
        Self {
            name,
            attrs: AHashMap::new(),
        }
    }

    pub fn get_attr(&self, name: &str) -> Option<&AtomicObjId> {
        self.attrs.get(name)
    }

    pub fn set_attr(&mut self, name: &'a str, value: ObjId) {
        if let Some(v) = self.attrs.get_mut(name) {
            v.store(value, std::sync::atomic::Ordering::Relaxed);
        } else {
            self.attrs.insert(name, AtomicObjId::new(value));
        }
    }

    pub fn list_attrs(&self) -> Keys<'_, &'a str, AtomicObjId> {
        self.attrs.keys()
    }

    pub fn iter_values(&self) -> impl Iterator<Item = &AtomicObjId> {
        self.attrs.values()
    }

    pub fn get_cls_name(&self) -> &str {
        self.name.as_str()
    }
}
