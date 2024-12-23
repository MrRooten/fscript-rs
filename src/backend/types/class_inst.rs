use std::{
    borrow::Cow, collections::{hash_map::Keys, HashMap}, fmt::Debug
};

use super::base::{FSRObject, FSRValue, ObjId};

#[derive(Clone)]
pub struct FSRClassInst<'a> {
    #[allow(unused)]
    name: &'a str,
    attrs: HashMap<&'a str, ObjId>,
}

impl Debug for FSRClassInst<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut new_hash = HashMap::new();
        for kv in &self.attrs {
            let obj = FSRObject::id_to_obj(*kv.1);
            if let FSRValue::Function(_) = &obj.value {
                continue;
            }
            new_hash.insert(kv.0, Cow::Borrowed(obj));
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
            attrs: HashMap::new(),
        }
    }

    pub fn get_attr(&self, name: &str) -> Option<&ObjId> {
        return self.attrs.get(name);
    }

    pub fn set_attr(&mut self, name: &'a str, value: ObjId) {
        if self.attrs.contains_key(name) {
            let v = self.attrs.get(name).unwrap();
            let obj = FSRObject::id_to_obj(*v);
            obj.ref_dec();
        }
        self.attrs.insert(name, value);
    }

    pub fn list_attrs(&self) -> Keys<&'a str, ObjId> {
        return self.attrs.keys();
    }

    pub fn get_cls_name(&self) -> &str {
        self.name
    }
}
