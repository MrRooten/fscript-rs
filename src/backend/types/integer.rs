#![allow(unused)]


use std::hash::{Hash, Hasher};

use ahash::AHasher;

use crate::{
    backend::{
        compiler::bytecode::BinaryOffset,
        memory::GarbageCollector,
        types::float::FSRFloat,
        vm::{
            thread::{CallFrame, FSRThreadRuntime},
            virtual_machine::{FSRVM, gid},
        },
    }, to_rs_list, utils::error::FSRError
};

use super::{
    base::{GlobalObj, FSRObject, FSRRetValue, FSRValue, ObjId},
    class::FSRClass,
    code::FSRCode,
    fn_def::FSRFn,
};

pub struct FSRInteger {}

pub fn add(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            let obj = thread.garbage_collect.new_object_in_place();
            obj.value = FSRValue::Integer(self_int + other_int);
            obj.cls = self_object.cls;
            return Ok(FSRRetValue::GlobalId(FSRObject::obj_to_id(obj)));
        }
    }

    unimplemented!()
}

pub fn sub(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            let obj = thread.garbage_collect.new_object_in_place();
            obj.value = FSRValue::Integer(self_int - other_int);
            obj.cls = self_object.cls;
            return Ok(FSRRetValue::GlobalId(FSRObject::obj_to_id(obj)));
        }
    }

    unimplemented!()
}

fn mul(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            let v = thread
                .garbage_collect
                .new_object(FSRValue::Integer(self_int * other_int), gid(GlobalObj::IntegerCls));

            return Ok(FSRRetValue::GlobalId(v));
        }
    }

    unimplemented!()
}

fn div(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    if len != 2 {
        return Err(FSRError::new(
            "div requires exactly 2 arguments",
            crate::utils::error::FSRErrCode::RuntimeError,
        ));
    }
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            let obj = thread.garbage_collect.new_object(
                FSRValue::Float(self_int as f64 / other_int as f64),
                gid(GlobalObj::FloatCls) as ObjId,
            );
            return Ok(FSRRetValue::GlobalId(obj));
        }
    }

    unimplemented!()
}


pub fn reminder(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    if len != 2 {
        return Err(FSRError::new(
            "reminder requires exactly 2 arguments",
            crate::utils::error::FSRErrCode::RuntimeError,
        ));
    }
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            let obj = thread.garbage_collect.new_object_in_place();
            obj.value = FSRValue::Integer(self_int % other_int);
            obj.cls = self_object.cls;
            return Ok(FSRRetValue::GlobalId(FSRObject::obj_to_id(obj)));
            //return Ok(FSRRetValue::GlobalId(v));
        }
    }

    unimplemented!()
}

fn left_shift(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            let v = thread
                .garbage_collect
                .new_object(FSRValue::Integer(self_int << other_int), gid(GlobalObj::IntegerCls));

            return Ok(FSRRetValue::GlobalId(v));
        }
    }

    unimplemented!()
}

fn right_shift(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            let v = thread
                .garbage_collect
                .new_object(FSRValue::Integer(self_int >> other_int), gid(GlobalObj::IntegerCls));

            return Ok(FSRRetValue::GlobalId(v));
        }
    }
    unimplemented!()
}

pub fn greater(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            if self_int > other_int {
                return Ok(FSRRetValue::GlobalId(FSRObject::true_id()));
            } else {
                return Ok(FSRRetValue::GlobalId(FSRObject::false_id()));
            }
        }
    }
    unimplemented!()
}

pub fn less(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            if self_int < other_int {
                return Ok(FSRRetValue::GlobalId(FSRObject::true_id()));
            } else {
                return Ok(FSRRetValue::GlobalId(FSRObject::false_id()));
            }
        }
    }
    unimplemented!()
}

pub fn greater_equal(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    if len != 2 {
        return Err(FSRError::new(
            "greater_equal requires at least 2 arguments",
            crate::utils::error::FSRErrCode::RuntimeError,
        ));
    }
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            if self_int >= other_int {
                return Ok(FSRRetValue::GlobalId(FSRObject::true_id()));
            } else {
                return Ok(FSRRetValue::GlobalId(FSRObject::false_id()));
            }
        }
    }
    unimplemented!()
}

pub fn less_equal(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            if self_int <= other_int {
                return Ok(FSRRetValue::GlobalId(FSRObject::true_id()));
            } else {
                return Ok(FSRRetValue::GlobalId(FSRObject::false_id()));
            }
        }
    }
    unimplemented!()
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
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    match (&self_object.value, &other_object.value) {
        (FSRValue::Integer(self_int), FSRValue::Integer(other_int)) => {
            if *self_int == *other_int {
                return Ok(FSRRetValue::GlobalId(FSRObject::true_id()));
            } else {
                return Ok(FSRRetValue::GlobalId(FSRObject::false_id()));
            }
        },
        _ => {
            return Ok(FSRRetValue::GlobalId(FSRObject::false_id()))
        }
    }

    Ok(FSRRetValue::GlobalId(FSRObject::false_id()))
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
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            if self_int != other_int {
                return Ok(FSRRetValue::GlobalId(FSRObject::true_id()));
            } else {
                return Ok(FSRRetValue::GlobalId(FSRObject::false_id()));
            }
        }
    }
    
    Ok(FSRRetValue::GlobalId(FSRObject::true_id()))
}

pub fn sorted_value(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    if len != 1 {
        return Err(FSRError::new(
            "sorted_value requires exactly 1 argument",
            crate::utils::error::FSRErrCode::RuntimeError,
        ));
    }
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);

    if let FSRValue::Integer(self_int) = self_object.value {
        return Ok(FSRRetValue::GlobalId(args[0]));
    }
    unimplemented!()
}

fn hash_integer(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    if len != 1 {
        return Err(FSRError::new(
            "hash_integer requires exactly 1 argument",
            crate::utils::error::FSRErrCode::RuntimeError,
        ));
    }
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);


    if let FSRValue::Integer(self_int) = &self_object.value {

        return Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
            FSRValue::Integer(*self_int),
            gid(GlobalObj::IntegerCls),
        )));
    }

    unimplemented!()
}

impl<'a> FSRInteger {
    pub fn get_class() -> FSRClass {
        let mut cls = FSRClass::new("Integer");
        let add_fn = FSRFn::from_rust_fn_static(add, "integer_add");
        //cls.insert_attr("__add__", add_fn);
        cls.insert_offset_attr(BinaryOffset::Add, add_fn);
        let sub_fn = FSRFn::from_rust_fn_static(sub, "integer_sub");
        //cls.insert_attr("__sub__", sub_fn);
        cls.insert_offset_attr(BinaryOffset::Sub, sub_fn);

        let div_fn = FSRFn::from_rust_fn_static(div, "integer_div");
        cls.insert_offset_attr(BinaryOffset::Div, div_fn);

        let mul_fn = FSRFn::from_rust_fn_static(mul, "integer_mul");
        //cls.insert_attr("__mul__", mul_fn);
        cls.insert_offset_attr(BinaryOffset::Mul, mul_fn);
        let gt_fn = FSRFn::from_rust_fn_static(greater, "integer_gt");
        //cls.insert_attr("__gt__", gt_fn);
        cls.insert_offset_attr(BinaryOffset::Greater, gt_fn);
        let gte_fn = FSRFn::from_rust_fn_static(greater_equal, "integer_gte");
        //cls.insert_attr("__gte__", gte_fn);
        cls.insert_offset_attr(BinaryOffset::GreatEqual, gte_fn);
        let lt_fn = FSRFn::from_rust_fn_static(less, "integer_lt");
        //cls.insert_attr("__lt__", lt_fn);
        cls.insert_offset_attr(BinaryOffset::Less, lt_fn);
        let lte_fn = FSRFn::from_rust_fn_static(less_equal, "integer_lte");
        //cls.insert_attr("__lte__", lte_fn);
        cls.insert_offset_attr(BinaryOffset::LessEqual, lte_fn);
        let eq = FSRFn::from_rust_fn_static(equal, "integer_eq");
        cls.insert_offset_attr(BinaryOffset::Equal, eq);

        let not_eq = FSRFn::from_rust_fn_static(not_equal, "integer_not_eq");
        cls.insert_offset_attr(BinaryOffset::NotEqual, not_eq);

        let hash_integer = FSRFn::from_rust_fn_static(hash_integer, "integer_not_eq");
        cls.insert_offset_attr(BinaryOffset::Hash, hash_integer);

        let reminder = FSRFn::from_rust_fn_static(reminder, "integer_reminder");
        cls.insert_offset_attr(BinaryOffset::Reminder, reminder);
        cls
    }

    pub fn new_inst(i: i64) -> FSRObject<'a> {
        let mut object = FSRObject::new();
        object.set_cls(gid(GlobalObj::IntegerCls));
        object.set_value(FSRValue::Integer(i));
        object
    }
}
