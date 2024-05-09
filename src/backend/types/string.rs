use crate::{
    backend::{
        types::{base::FSRValue, integer::FSRInteger},
        vm::{runtime::FSRVM, thread::CallState},
    },
    utils::error::FSRError,
};

use super::{
    base::{FSRObject, FSRRetValue},
    class::FSRClass,
    fn_def::FSRFn,
};

pub struct FSRString {}

fn string_len<'a>(
    args: Vec<u64>,
    _: &mut CallState,
    _: &FSRVM<'a>,
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);

    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(

    if let FSRValue::String(self_s) = &self_object.value {
        return Ok(FSRRetValue::Value(
            FSRInteger::new_inst(self_s.len() as i64),
        ));
    }

    unimplemented!()
}

impl FSRString {
    pub fn get_class<'a>(vm: &mut FSRVM<'a>) -> FSRClass<'a> {
        let mut cls = FSRClass::new("String");
        let len_m = FSRFn::from_rust_fn(string_len);
        cls.insert_attr("len", len_m, vm);
        cls
    }

    pub fn new_inst<'a>(s: String) -> FSRObject<'a> {
        let mut object = FSRObject::new();
        object.set_cls("String");
        object.set_value(FSRValue::String(s));
        object
    }
}
