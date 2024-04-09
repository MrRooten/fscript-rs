use std::collections::HashMap;

use crate::backend::vm::runtime::FSRVM;

use super::{base::{FSRObject, FSRValue}, class::FSRClass};

pub struct FSRInteger {

}

impl<'a> FSRInteger {
    pub fn get_class() -> FSRClass<'static> {
        let mut cls = FSRClass::new("Integer");

        cls
    }

    pub fn new_inst(i: i64, vm: &'a FSRVM<'a>) -> FSRObject<'a> {
        let mut object = FSRObject::new();
        object.set_cls("Integer");
        object.set_value(FSRValue::Integer(i));
        return object
    }
}