use std::cell::{Ref, RefCell};

use crate::backend::{types::{base::FSRValue, integer::FSRInteger}, vm::{runtime::FSRVM, thread::CallState}};

use super::{base::FSRObject, class::FSRClass, fn_def::FSRFn};

pub struct FSRString {

}

fn string_len<'a>(args: Vec<Ref<FSRObject>>, stack: &mut CallState, vm: &FSRVM<'a>) -> Result<FSRObject<'a>, ()> {
    let self_object = &args[0];

    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(

    if let FSRValue::String(self_s) = &self_object.value {
        return Ok(FSRInteger::new_inst(self_s.len() as i64));
    }

    unimplemented!()
}

impl FSRString {
    pub fn get_class<'a>(vm: & mut FSRVM<'a>) -> FSRClass<'a> {
        let mut cls = FSRClass::new("String");
        let len_m = FSRFn::from_rust_fn(string_len);
        cls.insert_attr("len", len_m, vm);
        cls
    }

    pub fn new_inst<'a>(s: String, vm: &'a mut FSRVM<'a>) -> FSRObject<'a> {
        let mut object = FSRObject::new();
        object.set_cls("String");

        return object
    }
}