use std::{cell::RefCell, collections::HashMap};

use crate::backend::vm::runtime::FSRVM;

use super::base::FSRObject;

#[derive(Debug, Clone)]
pub struct FSRClass<'a> {
    pub(crate) name: &'a str,
    pub(crate) attrs: HashMap<&'a str, u64>,
}

impl<'a> FSRClass<'a> {
    pub fn new(name: &'a str) -> FSRClass<'a> {
        FSRClass {
            name,
            attrs: HashMap::new(),
        }
    }

    pub fn insert_attr(&mut self, name: &'a str, object: FSRObject<'a>, vm: &mut FSRVM<'a>) {
        let obj_id = vm.register_object(object);
        self.attrs.insert(name, obj_id);
    }

    pub fn insert_attr_id(&mut self, name: &'a str, obj_id: u64) {
        self.attrs.insert(name, obj_id);
    }

    pub fn get_attr(&self, name: &str) -> Option<u64> {
        return self.attrs.get(name).copied();
    }

    pub fn get_name(&self) -> &str {
        self.name
    }
}
