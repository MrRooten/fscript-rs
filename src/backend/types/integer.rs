#![allow(unused)]

use crate::{
    backend::{compiler::bytecode::BinaryOffset, vm::{runtime::FSRVM, thread::{CallState, FSRThreadRuntime}}},
    utils::error::FSRError,
};

use super::{
    base::{FSRGlobalObjId, FSRObject, FSRRetValue, FSRValue},
    class::FSRClass,
    fn_def::FSRFn, module::FSRModule,
};

pub struct FSRInteger {}

fn add<'a>(
    args: &[u64],
    thread: &mut FSRThreadRuntime<'a>,
    module: Option<&FSRModule>
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(
    
    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            return Ok(FSRRetValue::Value(Box::new(FSRInteger::new_inst(
                self_int + other_int,
            ))));
        }
    }

    unimplemented!()
}

fn sub<'a>(
    args: &[u64],
    thread: &mut FSRThreadRuntime<'a>,
    module: Option<&FSRModule>
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            return Ok(FSRRetValue::Value(Box::new(FSRInteger::new_inst(
                self_int - other_int,
            ))));
        }
    }

    unimplemented!()
}

fn mul<'a>(
    args: &[u64],
    thread: &mut FSRThreadRuntime<'a>,
    module: Option<&FSRModule>
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            return Ok(FSRRetValue::Value(Box::new(FSRInteger::new_inst(
                self_int * other_int,
            ))));
        }
    }

    unimplemented!()
}

fn div<'a>(
    _args: &[u64],
    _stack: &'a mut CallState,
    _thread: &mut FSRThreadRuntime<'a>
) -> Result<FSRRetValue<'a>, FSRError> {
    // let self_id = args[0];
    // let other_id = args[1];
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    // if let FSRValue::Integer(self_int) = self_object.value {
    //     if let FSRValue::Integer(other_int) = other_object.value {
    //         return Ok(FSRInteger::new_inst(self_int * other_int));
    //     }
    // }

    unimplemented!()
}


fn left_shift<'a>(
    args: &[u64],
    thread: &mut FSRThreadRuntime<'a>,
    module: Option<&FSRModule>
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            return Ok(FSRRetValue::Value(Box::new(FSRInteger::new_inst(
                self_int << other_int,
            ))));
        }
    }

    unimplemented!()
}


fn right_shift<'a>(
    args: &[u64],
    thread: &mut FSRThreadRuntime<'a>,
    module: Option<&FSRModule>
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            return Ok(FSRRetValue::Value(Box::new(FSRInteger::new_inst(
                self_int >> other_int,
            ))));
        }
    }
    unimplemented!()
}

fn greater<'a>(
    args: &[u64],
    thread: &mut FSRThreadRuntime<'a>,
    module: Option<&FSRModule>
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            if self_int > other_int {
                return Ok(FSRRetValue::GlobalId(1));
            } else {
                return Ok(FSRRetValue::GlobalId(2));
            }
        }
    }
    unimplemented!()
}

fn less<'a>(
    args: &[u64],
    thread: &mut FSRThreadRuntime<'a>,
    module: Option<&FSRModule>
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            if self_int < other_int {
                return Ok(FSRRetValue::GlobalId(1));
            } else {
                return Ok(FSRRetValue::GlobalId(2));
            }
        }
    }
    unimplemented!()
}

fn greater_equal<'a>(
    args: &[u64],
    thread: &mut FSRThreadRuntime<'a>,
    module: Option<&FSRModule>
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            if self_int >= other_int {
                return Ok(FSRRetValue::GlobalId(1));
            } else {
                return Ok(FSRRetValue::GlobalId(2));
            }
        }
    }
    unimplemented!()
}

fn less_equal<'a>(
    args: &[u64],
    thread: &mut FSRThreadRuntime<'a>,
    module: Option<&FSRModule>
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            if self_int <= other_int {
                return Ok(FSRRetValue::GlobalId(1));
            } else {
                return Ok(FSRRetValue::GlobalId(2));
            }
        }
    }
    unimplemented!()
}

fn equal<'a>(
    args: &[u64],
    thread: &mut FSRThreadRuntime<'a>,
    module: Option<&FSRModule>
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            if self_int == other_int {
                return Ok(FSRRetValue::GlobalId(1));
            } else {
                return Ok(FSRRetValue::GlobalId(2));
            }
        }
    }
    unimplemented!()
}

impl<'a> FSRInteger {
    pub fn get_class() -> FSRClass<'a> {
        let mut cls = FSRClass::new("Integer");
        let add_fn = FSRFn::from_rust_fn(add);
        //cls.insert_attr("__add__", add_fn);
        cls.insert_offset_attr(BinaryOffset::Add, add_fn);
        let sub_fn = FSRFn::from_rust_fn(sub);
        //cls.insert_attr("__sub__", sub_fn);
        cls.insert_offset_attr(BinaryOffset::Sub, sub_fn);
        let mul_fn = FSRFn::from_rust_fn(mul);
        //cls.insert_attr("__mul__", mul_fn);
        cls.insert_offset_attr(BinaryOffset::Mul, mul_fn);
        let gt_fn = FSRFn::from_rust_fn(greater);
        //cls.insert_attr("__gt__", gt_fn);
        cls.insert_offset_attr(BinaryOffset::Greater, gt_fn);
        let gte_fn = FSRFn::from_rust_fn(greater_equal);
        //cls.insert_attr("__gte__", gte_fn);
        cls.insert_offset_attr(BinaryOffset::GreatEqual, gte_fn);
        let lt_fn = FSRFn::from_rust_fn(less);
        //cls.insert_attr("__lt__", lt_fn);
        cls.insert_offset_attr(BinaryOffset::Less, lt_fn);
        let lte_fn = FSRFn::from_rust_fn(less_equal);
        //cls.insert_attr("__lte__", lte_fn);
        cls.insert_offset_attr(BinaryOffset::LessEqual, lte_fn);
        let eq = FSRFn::from_rust_fn(equal);
        cls.insert_offset_attr(BinaryOffset::Equal, eq);
        cls
    }

    pub fn new_inst(i: i64) -> FSRObject<'a> {
        let mut object = FSRObject::new();
        object.set_cls(FSRGlobalObjId::IntegerCls as u64);
        object.set_value(FSRValue::Integer(i));
        object
    }
}
