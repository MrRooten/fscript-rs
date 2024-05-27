use std::collections::HashMap;

use crate::backend::vm::runtime::FSRVM;

use super::base::FSRObject;

#[derive(Debug, Clone)]
pub struct FSRClass<'a> {
    pub(crate) name: &'a str,
    pub(crate) attrs: HashMap<&'a str, u64>,
    offset_attrs    : Vec<u64>
}

impl<'a> FSRClass<'a> {
    pub fn new(name: &'a str) -> FSRClass<'a> {
        FSRClass {
            name,
            attrs: HashMap::new(),
            offset_attrs: vec![0;30],
        }
    }

    pub fn insert_attr(&mut self, name: &'a str, object: FSRObject<'a>) {
        let obj_id = FSRVM::register_object(object);
        self.attrs.insert(name, obj_id);
    }

    pub fn insert_offset_attr(&mut self, offset: usize, object: FSRObject<'a>) {
        let obj_id = FSRVM::register_object(object);
        self.offset_attrs[offset] = obj_id;
    }

    pub fn insert_attr_id(&mut self, name: &'a str, obj_id: u64) {
        self.attrs.insert(name, obj_id);
    }

    pub fn get_attr(&self, name: &str) -> Option<u64> {
        return self.attrs.get(name).copied();
    }

    pub fn get_offset_attr(&self, offset: usize) -> Option<u64> {
        if let Some(s) = self.offset_attrs.get(offset) {
            if s == &0 {
                return None
            }

            return Some(*s);
        }

        None
    }

    pub fn get_name(&self) -> &str {
        self.name
    }
}
