use crate::{backend::{compiler::bytecode::BinaryOffset, types::{base::FSRRetValue, fn_def::FSRFn}, vm::{thread::FSRThreadRuntime, virtual_machine::get_object_by_global_id}}, utils::error::FSRError};

use super::{base::{GlobalObj, FSRObject, FSRValue, ObjId}, class::FSRClass};

pub struct FSRNone {

}

pub fn not_equal(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    if len != 2 {
        return Err(FSRError::new(
            "not_equal requires at least 2 arguments",
            crate::utils::error::FSRErrCode::RuntimeError,
        ));
    }
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    if args[0] == FSRObject::none_id() {
        return Ok(FSRRetValue::GlobalId(FSRObject::false_id()));
    }
    
    Ok(FSRRetValue::GlobalId(FSRObject::true_id()))
}

pub fn equal(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    if len != 2 {
        return Err(FSRError::new(
            "equal requires at least 2 arguments",
            crate::utils::error::FSRErrCode::RuntimeError,
        ));
    }
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    if args[1] == FSRObject::none_id() {
        return Ok(FSRRetValue::GlobalId(FSRObject::true_id()));
    }
    
    Ok(FSRRetValue::GlobalId(FSRObject::false_id()))
}

impl FSRNone {
    pub fn get_class() -> FSRClass {
        let mut cls = FSRClass::new("None");
        let not_eq = FSRFn::from_rust_fn_static(not_equal, "none_not_eq");
        cls.insert_offset_attr(BinaryOffset::NotEqual, not_eq);
        let eq = FSRFn::from_rust_fn_static(equal, "none_eq");
        cls.insert_offset_attr(BinaryOffset::Equal, eq);
        cls
    }
}