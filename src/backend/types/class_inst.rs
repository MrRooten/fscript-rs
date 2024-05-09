use std::collections::{hash_map::Keys, HashMap};

use super::base::FSRObject;

#[derive(Debug, Clone)]
pub struct FSRClassInst<'a> {
    #[allow(unused)]
    name: &'a str,
    attrs: HashMap<&'a str, u64>,
}

impl<'a> FSRClassInst<'a> {
    pub fn new(name: &'a str) -> FSRClassInst<'a> {
        Self {
            name,
            attrs: HashMap::new(),
        }
    }

    pub fn get_attr(&self, name: &str) -> Option<&u64> {
        return self.attrs.get(name);
    }

    pub fn set_attr(&mut self, name: &'a str, value: u64) {
        if self.attrs.contains_key(name) {
            let v = self.attrs.get(name).unwrap();
            let obj = FSRObject::id_to_obj(*v);
            obj.ref_dec();
        }
        self.attrs.insert(name, value);
    }

    pub fn list_attrs(&self) -> Keys<&'a str, u64> {
        return self.attrs.keys();
    }

    pub fn get_cls_name(&self) -> &str {
        return &self.name
    }
}
