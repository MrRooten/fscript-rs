use std::borrow::Cow;

use crate::{
    backend::{
        compiler::bytecode::BinaryOffset,
        types::{base::FSRValue, integer::FSRInteger},
        vm::thread::FSRThreadRuntime,
    },
    utils::error::FSRError,
};

use super::{
    base::{FSRGlobalObjId, FSRObject, FSRRetValue, ObjId},
    class::FSRClass,
    fn_def::FSRFn,
};

pub struct FSRString {}

fn string_len<'a>(
    args: &[ObjId],
    _thread: &mut FSRThreadRuntime<'a>,
    module: Option<ObjId>,
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);

    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(

    if let FSRValue::String(self_s) = &self_object.value {
        return Ok(FSRRetValue::Value(Box::new(FSRInteger::new_inst(
            self_s.len() as i64,
        ))));
    }

    unimplemented!()
}

fn add<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: Option<ObjId>,
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(

    if let FSRValue::String(self_str) = &self_object.value {
        if let FSRValue::String(other_str) = &other_object.value {
            return Ok(FSRRetValue::Value(
                thread
                    .get_vm()
                    .allocator
                    .new_string(Cow::Owned(format!("{}{}", self_str, other_str))),
            ));
        } else {
            return Err(FSRError::new(
                "right value is not a string",
                crate::utils::error::FSRErrCode::NotValidArgs,
            ));
        }
    } else {
        return Err(FSRError::new(
            "left value is not a string",
            crate::utils::error::FSRErrCode::NotValidArgs,
        ));
    }

    unimplemented!()
}

fn eq<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: Option<ObjId>,
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(

    if let FSRValue::String(self_str) = &self_object.value {
        if let FSRValue::String(other_str) = &other_object.value {
            if self_str.eq(other_str) {
                return Ok(FSRRetValue::GlobalId(FSRObject::true_id()));
            } else {
                return Ok(FSRRetValue::GlobalId(FSRObject::false_id()));
            }
        } else {
            return Err(FSRError::new(
                "right value is not a string",
                crate::utils::error::FSRErrCode::NotValidArgs,
            ));
        }
    } else {
        return Err(FSRError::new(
            "left value is not a string",
            crate::utils::error::FSRErrCode::NotValidArgs,
        ));
    }

    unimplemented!()
}

impl FSRString {
    pub fn get_class<'a>() -> FSRClass<'a> {
        let mut cls = FSRClass::new("String");
        let len_m = FSRFn::from_rust_fn(string_len);
        cls.insert_attr("len", len_m);
        let add_fn = FSRFn::from_rust_fn(add);
        //cls.insert_attr("__add__", add_fn);
        cls.insert_offset_attr(BinaryOffset::Add, add_fn);

        let eq_fn = FSRFn::from_rust_fn(eq);
        //cls.insert_attr("__add__", add_fn);
        cls.insert_offset_attr(BinaryOffset::Equal, eq_fn);
        cls
    }

    pub fn new_inst(s: Cow<'_, str>) -> FSRObject<'_> {
        let mut object = FSRObject::new();
        object.set_cls(FSRGlobalObjId::StringCls as ObjId);
        object.set_value(FSRValue::String(s));
        object
    }
}
