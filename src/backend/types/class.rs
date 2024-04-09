use std::{cell::RefCell, collections::HashMap};

use crate::backend::vm::runtime::FSRVM;

use super::base::FSRObject;

#[derive(Debug)]
pub struct FSRClass<'a> {
    pub(crate) name        : &'a str,
    pub(crate) attrs       : HashMap<&'a str, u64>
}

impl<'a> FSRClass<'a> {
    pub fn new(name: &'a str) -> FSRClass<'a> {
        let cls = FSRClass {
            name: name,
            attrs: HashMap::new(),
        };
        cls
    }

    pub fn insert_attr(&mut self, name: &'a str, object: FSRObject<'a>, vm: &mut FSRVM<'a>) {
        let obj_id = vm.register_object(object);
        self.attrs.insert(name, obj_id);
    }


}