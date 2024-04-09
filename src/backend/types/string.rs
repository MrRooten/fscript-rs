use std::cell::RefCell;

use crate::backend::vm::runtime::FSRVM;

use super::{base::FSRObject, class::FSRClass};

pub struct FSRString {

}

impl FSRString {
    pub fn get_class<'a>(vm: &'a mut FSRVM<'a>) -> FSRClass<'a> {
        let mut cls = FSRClass::new("String");

        cls
    }

    pub fn new_inst<'a>(s: String, vm: &'a mut FSRVM<'a>) -> FSRObject<'a> {
        let mut object = FSRObject::new();
        object.set_cls("String");

        return object
    }
}