use std::borrow::Cow;

use crate::{
    backend::{
        types::{base::FSRValue, integer::FSRInteger},
        vm::thread::FSRThreadRuntime,
    },
    utils::error::FSRError,
};

use super::{
    base::{FSRGlobalObjId, FSRObject, FSRRetValue},
    class::FSRClass,
    fn_def::FSRFn, module::FSRModule,
};

pub struct FSRString {}

fn string_len<'a>(
    args: &[u64],
    _thread: &mut FSRThreadRuntime<'a>,
    _module: Option<&FSRModule>
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);

    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(

    if let FSRValue::String(self_s) = &self_object.value {
        return Ok(FSRRetValue::Value(
            Box::new(FSRInteger::new_inst(self_s.len() as i64)),
        ));
    }

    unimplemented!()
}

impl FSRString {
    pub fn get_class<'a>() -> FSRClass<'a> {
        let mut cls = FSRClass::new("String");
        let len_m = FSRFn::from_rust_fn(string_len);
        cls.insert_attr("len", len_m);
        cls
    }

    pub fn new_inst(s: Cow<'_, str>) -> FSRObject<'_> {
        let mut object = FSRObject::new();
        object.set_cls(FSRGlobalObjId::StringCls as u64);
        object.set_value(FSRValue::String(s));
        object
    }

}
