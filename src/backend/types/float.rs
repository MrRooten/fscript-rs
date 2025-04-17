use crate::{
    backend::{compiler::bytecode::BinaryOffset, memory::GarbageCollector, vm::thread::FSRThreadRuntime},
    utils::error::FSRError,
};

use super::{
    base::{FSRGlobalObjId, FSRObject, FSRRetValue, FSRValue, ObjId},
    class::FSRClass,
    fn_def::FSRFn,
};

pub struct FSRFloat {
    pub(crate) value: f64,
}

pub fn add<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId
) -> Result<FSRRetValue<'a>, FSRError> {
    let _ = thread;
    let _ = module;
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(
    
    if let FSRValue::Float(self_int) = self_object.value {
        if let FSRValue::Float(other_int) = other_object.value {
            let obj = thread.garbage_collect.new_object(
                FSRValue::Float(self_int + other_int),
                FSRGlobalObjId::FloatCls as ObjId,
            );
            return Ok(FSRRetValue::GlobalId(obj));
        }
    }

    unimplemented!()
}

pub fn sub<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId
) -> Result<FSRRetValue<'a>, FSRError> {
    let _ = module;
    let _ = thread;
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(

    if let FSRValue::Float(self_int) = self_object.value {
        if let FSRValue::Float(other_int) = other_object.value {
            let obj = thread.garbage_collect.new_object(
                FSRValue::Float(self_int - other_int),
                FSRGlobalObjId::FloatCls as ObjId,
            );
            return Ok(FSRRetValue::GlobalId(obj));
        }
    }

    unimplemented!()
}

pub fn mul<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId
) -> Result<FSRRetValue<'a>, FSRError> {
    let _ = module;
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Float(self_int) = self_object.value {
        if let FSRValue::Float(other_int) = other_object.value {
            let obj = thread.garbage_collect.new_object(
                FSRValue::Float(self_int * other_int),
                FSRGlobalObjId::FloatCls as ObjId,
            );
            return Ok(FSRRetValue::GlobalId(obj));
        }
    }

    unimplemented!()
}

fn div<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId
) -> Result<FSRRetValue<'a>, FSRError> {
    let _ = module;
    let _ = thread;
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Float(self_int) = self_object.value {
        if let FSRValue::Float(other_int) = other_object.value {
            let obj = thread.garbage_collect.new_object(
                FSRValue::Float(self_int / other_int),
                FSRGlobalObjId::FloatCls as ObjId,
            );
            return Ok(FSRRetValue::GlobalId(obj));
        }
    }

    unimplemented!()
}



pub fn greater<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId
) -> Result<FSRRetValue<'a>, FSRError> {
    let _ = thread;
    let _ = module;
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Float(self_int) = self_object.value {
        if let FSRValue::Float(other_int) = other_object.value {
            if self_int > other_int {
                return Ok(FSRRetValue::GlobalId(1));
            } else {
                return Ok(FSRRetValue::GlobalId(2));
            }
        }
    }
    unimplemented!()
}

pub fn less<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId
) -> Result<FSRRetValue<'a>, FSRError> {
    let _ = module;
    let _ = thread;
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Float(self_int) = self_object.value {
        if let FSRValue::Float(other_int) = other_object.value {
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
    module: ObjId
) -> Result<FSRRetValue<'a>, FSRError> {
    let _ = module;
    let _ = thread;
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Float(self_int) = self_object.value {
        if let FSRValue::Float(other_int) = other_object.value {
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
    module: ObjId
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Float(self_int) = self_object.value {
        if let FSRValue::Float(other_int) = other_object.value {
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
    module: ObjId
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Float(self_int) = self_object.value {
        if let FSRValue::Float(other_int) = other_object.value {
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
    module: ObjId
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Float(self_int) = self_object.value {
        if let FSRValue::Float(other_int) = other_object.value {
            if self_int != other_int {
                return Ok(FSRRetValue::GlobalId(1));
            } else {
                return Ok(FSRRetValue::GlobalId(2));
            }
        }
    }
    unimplemented!()
}


impl<'a> FSRFloat {
    pub fn new(value: f64) -> Self {
        Self { value }
    }

    pub fn get_value(&self) -> f64 {
        self.value
    }

    pub fn new_inst(f: f64) -> FSRObject<'a> {
        let mut object = FSRObject::new();
        object.set_cls(FSRGlobalObjId::FloatCls as ObjId);
        object.set_value(FSRValue::Float(f));
        object
    }

    pub fn get_class() -> FSRClass<'a> {
        let mut cls = FSRClass::new("Float");
        let add_fn = FSRFn::from_rust_fn_static(add, "float_add");
        //cls.insert_attr("__add__", add_fn);
        cls.insert_offset_attr(BinaryOffset::Add, add_fn);
        let sub_fn = FSRFn::from_rust_fn_static(sub, "float_sub");
        //cls.insert_attr("__sub__", sub_fn);
        cls.insert_offset_attr(BinaryOffset::Sub, sub_fn);

        let div_fn = FSRFn::from_rust_fn_static(div, "float_div");
        cls.insert_offset_attr(BinaryOffset::Div, div_fn);

        let mul_fn = FSRFn::from_rust_fn_static(mul, "float_mul");
        //cls.insert_attr("__mul__", mul_fn);
        cls.insert_offset_attr(BinaryOffset::Mul, mul_fn);
        let gt_fn = FSRFn::from_rust_fn_static(greater, "float_gt");
        //cls.insert_attr("__gt__", gt_fn);
        cls.insert_offset_attr(BinaryOffset::Greater, gt_fn);
        let gte_fn = FSRFn::from_rust_fn_static(greater_equal, "float_gte");
        //cls.insert_attr("__gte__", gte_fn);
        cls.insert_offset_attr(BinaryOffset::GreatEqual, gte_fn);
        let lt_fn = FSRFn::from_rust_fn_static(less, "float_lt");
        //cls.insert_attr("__lt__", lt_fn);
        cls.insert_offset_attr(BinaryOffset::Less, lt_fn);
        let lte_fn = FSRFn::from_rust_fn_static(less_equal, "float_lte");
        //cls.insert_attr("__lte__", lte_fn);
        cls.insert_offset_attr(BinaryOffset::LessEqual, lte_fn);
        let eq = FSRFn::from_rust_fn_static(equal, "float_eq");
        cls.insert_offset_attr(BinaryOffset::Equal, eq);

        let not_eq = FSRFn::from_rust_fn_static(not_equal, "float_not_eq");
        cls.insert_offset_attr(BinaryOffset::NotEqual, not_eq);
        cls
    }
}

