#![allow(unused)]

use crate::{
    backend::{compiler::bytecode::BinaryOffset, types::float::FSRFloat, vm::{virtual_machine::FSRVM, thread::{CallFrame, FSRThreadRuntime}}},
    utils::error::FSRError,
};

use super::{
    base::{FSRGlobalObjId, FSRObject, FSRRetValue, FSRValue, ObjId},
    class::FSRClass,
    fn_def::FSRFn, module::FSRModule,
};

pub struct FSRInteger {}

fn add<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: Option<ObjId>
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(
    
    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            return Ok(FSRRetValue::Value(thread.thread_allocator.new_object(FSRValue::Integer(self_int + other_int), self_object.cls)));
        }
    }

    unimplemented!()
}

fn sub<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: Option<ObjId>
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            return Ok(FSRRetValue::Value(thread.thread_allocator.new_object(FSRValue::Integer(self_int - other_int), self_object.cls)));
        }
    }

    unimplemented!()
}

fn mul<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: Option<ObjId>
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            return Ok(FSRRetValue::Value(thread.thread_allocator.new_object(FSRValue::Integer(self_int * other_int), self_object.cls)));
        }
    }

    unimplemented!()
}

fn div<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: Option<ObjId>
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            return Ok(FSRRetValue::Value(Box::new(FSRFloat::new_inst(self_int as f64 / other_int as f64))));
        }
    }

    unimplemented!()
}


fn left_shift<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: Option<ObjId>
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            return Ok(FSRRetValue::Value(thread.get_vm().lock().unwrap().allocator.new_object(FSRValue::Integer(self_int << other_int), self_object.cls)));
        }
    }

    unimplemented!()
}


fn right_shift<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: Option<ObjId>
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
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: Option<ObjId>
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
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: Option<ObjId>
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
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: Option<ObjId>
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
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: Option<ObjId>
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
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: Option<ObjId>
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

fn not_equal<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: Option<ObjId>
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            if self_int != other_int {
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
        cls
    }

    pub fn new_inst(i: i64) -> FSRObject<'a> {
        let mut object = FSRObject::new();
        object.set_cls(FSRGlobalObjId::IntegerCls as ObjId);
        object.set_value(FSRValue::Integer(i));
        object
    }
}
