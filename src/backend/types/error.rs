use crate::{backend::{types::base::{FSRObject, FSRValue}, vm::thread::FSRThreadRuntime}, utils::error::FSRError};

use super::{base::{FSRRetValue, ObjId}, class::FSRClass, fn_def::FSRFn};

#[derive(Debug, Clone)]
pub struct FSRException {
    
}

fn kind<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId
) -> Result<FSRRetValue, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(
    
    if let FSRValue::ClassInst(exception) = &self_object.value {
        let obj = match exception.get_attr("__kind__") {
            Some(s) => s.load(std::sync::atomic::Ordering::Relaxed),
            None => {
                panic!("not found __kind__")
            }
        };
        return Ok(FSRRetValue::GlobalId(obj))
    }

    unimplemented!()
}

fn message<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId
) -> Result<FSRRetValue, FSRError> {
    let message_object = FSRObject::id_to_obj(args[0]);
    let kind_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(
    
    if let FSRValue::ClassInst(exception) = &message_object.value {
        let obj = match exception.get_attr("__msg__") {
            Some(s) => s.load(std::sync::atomic::Ordering::Relaxed),
            None => {
                panic!("not found __msg__")
            }
        };
        return Ok(FSRRetValue::GlobalId(obj))
    }

    unimplemented!()
}

fn new<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId
) -> Result<FSRRetValue, FSRError> {
    
    unimplemented!()
}

impl FSRException {
    pub fn get_class() -> FSRClass<'static> {
        let mut cls = FSRClass::new("Exception");
        let kind_fn = FSRFn::from_rust_fn_static(kind, "kind");
        //cls.insert_attr("__add__", add_fn);
        cls.insert_attr("kind", kind_fn);

        let message_fn = FSRFn::from_rust_fn_static(kind, "message");
        //cls.insert_attr("__add__", add_fn);
        cls.insert_attr("message", message_fn);

        let new_fn = FSRFn::from_rust_fn_static(new, "__new__");
        cls.insert_attr("__new__", new_fn);
        cls
    }
}