use std::{cell::Ref, collections::HashMap};

use crate::backend::vm::{runtime::FSRVM, thread::CallState};

use super::{base::{FSRObject, FSRValue}, class::FSRClass, fn_def::FSRFn};

pub struct FSRInteger {

}

fn add<'a>(args: Vec<Ref<FSRObject>>, stack: &mut CallState, vm: &FSRVM<'a>) -> Result<FSRObject<'a>, ()> {
    let self_object = &args[0];
    let other_object = &args[1];
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            return Ok(FSRInteger::new_inst(self_int + other_int));
        }
    }

    unimplemented!()
}

fn sub<'a>(args: Vec<Ref<FSRObject>>, stack: &mut CallState, vm: &FSRVM<'a>) -> Result<FSRObject<'a>, ()> {
    let self_object = &args[0];
    let other_object = &args[1];
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            return Ok(FSRInteger::new_inst(self_int - other_int));
        }
    }

    unimplemented!()
}

fn mul<'a>(args: Vec<Ref<FSRObject>>, stack: &mut CallState, vm: &FSRVM<'a>) -> Result<FSRObject<'a>, ()> {
    let self_object = &args[0];
    let other_object = &args[1];
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            return Ok(FSRInteger::new_inst(self_int * other_int));
        }
    }

    unimplemented!()
}

fn div<'a>(args: Vec<u64>, stack: &'a mut CallState, vm: &FSRVM<'a>) -> Result<FSRObject<'a>, ()> {
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

fn left_shift<'a>(args: Vec<Ref<FSRObject>>, stack: &mut CallState, vm: &FSRVM<'a>) -> Result<FSRObject<'a>, ()> {
    let self_object = &args[0];
    let other_object = &args[1];
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            return Ok(FSRInteger::new_inst(self_int << other_int));
        }
    }

    unimplemented!()
}

fn right_shift<'a>(args: Vec<Ref<FSRObject>>, stack: &mut CallState, vm: &FSRVM<'a>) -> Result<FSRObject<'a>, ()> {
    let self_object = &args[0];
    let other_object = &args[1];
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            return Ok(FSRInteger::new_inst(self_int >> other_int));
        }
    }
    unimplemented!()
}

fn greater<'a>(args: Vec<Ref<FSRObject>>, stack: &mut CallState, vm: &FSRVM<'a>) -> Result<FSRObject<'a>, ()> {
    let self_object = &args[0];
    let other_object = &args[1];
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            if self_int > other_int {
                return Ok(FSRObject {
                    obj_id: vm.get_true_id(),
                    value: FSRValue::Bool(true),
                    cls: "Bool",
                });
            } else {
                return Ok(FSRObject {
                    obj_id: vm.get_false_id(),
                    value: FSRValue::Bool(false),
                    cls: "Bool",
                });
            }
        }
    }
    unimplemented!()
}

fn less<'a>(args: Vec<Ref<FSRObject>>, stack: &mut CallState, vm: &FSRVM<'a>) -> Result<FSRObject<'a>, ()> {
    let self_object = &args[0];
    let other_object = &args[1];
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            if self_int < other_int {
                return Ok(FSRObject {
                    obj_id: vm.get_true_id(),
                    value: FSRValue::Bool(true),
                    cls: "Bool",
                });
            } else {
                return Ok(FSRObject {
                    obj_id: vm.get_false_id(),
                    value: FSRValue::Bool(false),
                    cls: "Bool",
                });
            }
        }
    }
    unimplemented!()
}

fn greater_equal<'a>(args: Vec<Ref<FSRObject>>, stack: &mut CallState, vm: &FSRVM<'a>) -> Result<FSRObject<'a>, ()> {
    let self_object = &args[0];
    let other_object = &args[1];
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            if self_int >= other_int {
                return Ok(FSRObject {
                    obj_id: vm.get_true_id(),
                    value: FSRValue::Bool(true),
                    cls: "Bool",
                });
            } else {
                return Ok(FSRObject {
                    obj_id: vm.get_false_id(),
                    value: FSRValue::Bool(false),
                    cls: "Bool",
                });
            }
        }
    }
    unimplemented!()
}

fn less_equal<'a>(args: Vec<Ref<FSRObject>>, stack: &mut CallState, vm: &FSRVM<'a>) -> Result<FSRObject<'a>, ()> {
    let self_object = &args[0];
    let other_object = &args[1];
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow();

    if let FSRValue::Integer(self_int) = self_object.value {
        if let FSRValue::Integer(other_int) = other_object.value {
            if self_int <= other_int {
                return Ok(FSRObject {
                    obj_id: vm.get_true_id(),
                    value: FSRValue::Bool(true),
                    cls: "Bool",
                });
            } else {
                return Ok(FSRObject {
                    obj_id: vm.get_false_id(),
                    value: FSRValue::Bool(false),
                    cls: "Bool",
                });
            }
        }
    }
    unimplemented!()
}

impl<'a> FSRInteger {
    pub fn get_class(vm: &mut FSRVM<'a>) -> FSRClass<'a> {
        let mut cls = FSRClass::new("Integer");
        let add_fn = FSRFn::from_rust_fn(add);
        cls.insert_attr("__add__", add_fn, vm);
        let sub_fn = FSRFn::from_rust_fn(sub);
        cls.insert_attr("__sub__", sub_fn, vm);
        let mul_fn = FSRFn::from_rust_fn(mul);
        cls.insert_attr("__mul__", mul_fn, vm);
        let gt_fn = FSRFn::from_rust_fn(greater);
        cls.insert_attr("__gt__", gt_fn, vm);
        let gte_fn = FSRFn::from_rust_fn(greater_equal);
        cls.insert_attr("__gte__", gte_fn, vm);
        let lt_fn = FSRFn::from_rust_fn(less);
        cls.insert_attr("__lt__", lt_fn, vm);
        let lte_fn = FSRFn::from_rust_fn(less_equal);
        cls.insert_attr("__lte__", lte_fn, vm);
        cls
    }

    

    pub fn new_inst(i: i64) -> FSRObject<'a> {
        let mut object = FSRObject::new();
        object.set_cls("Integer");
        object.set_value(FSRValue::Integer(i));
        return object
    }
}