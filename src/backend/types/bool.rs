use crate::{backend::{compiler::bytecode::BinaryOffset, types::{base::FSRRetValue, fn_def::FSRFn}, vm::{thread::FSRThreadRuntime, virtual_machine::get_object_by_global_id}}, utils::error::FSRError};

use super::{base::{GlobalObj, FSRObject, FSRValue, ObjId}, class::FSRClass};

pub struct FSRBool {

}


pub fn equal(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    if args[0] == args[1] {
        Ok(FSRRetValue::GlobalId(FSRObject::true_id()))
    } else {
        Ok(FSRRetValue::GlobalId(FSRObject::false_id()))
    }
}

impl<'a> FSRBool {
    pub fn get_class() -> FSRClass<'a> {
        let mut cls = FSRClass::new("Bool");
        let eq = FSRFn::from_rust_fn_static(equal, "bool_eq");
        cls.insert_offset_attr(BinaryOffset::Equal, eq);
        cls
    }

    pub fn new_inst(i: bool) -> FSRObject<'a> {
        let mut object = FSRObject::new();
        object.set_cls(get_object_by_global_id(GlobalObj::BoolCls) as ObjId);
        object.set_value(FSRValue::Bool(i));
        object
    }
}