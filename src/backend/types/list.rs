use std::borrow::Cow;

use crate::{backend::{types::{base::{FSRObject, FSRValue}, integer::FSRInteger, string::FSRString}, vm::{runtime::FSRVM, thread::FSRThreadRuntime}}, utils::error::FSRError};

use super::{base::FSRRetValue, class::FSRClass, fn_def::FSRFn};

#[derive(Debug, Clone)]
pub struct FSRList {
    vs      : Vec<u64>
}

fn list_len<'a>(
    args: Vec<u64>,
    _: &mut FSRThreadRuntime<'a>
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);

    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(

    if let FSRValue::List(self_s) = &self_object.value {
        return Ok(FSRRetValue::Value(
            FSRInteger::new_inst(self_s.vs.len() as i64),
        ));
    }

    unimplemented!()
}

fn list_string<'a>(
    args: Vec<u64>,
    thread: &mut FSRThreadRuntime<'a>
) -> Result<FSRRetValue<'a>, FSRError> {
    let mut s = String::new();
    s.push('[');
    let obj_id = args[0];
    let obj = FSRObject::id_to_obj(obj_id);
    if let FSRValue::List(l) = &obj.value {
        let mut count = 0;
        let size = l.get_items().len();
        for id in l.get_items() {
            let obj = FSRObject::id_to_obj(*id);
            let s_obj = obj.to_string(thread);
            if let FSRValue::String(_s) = &s_obj.value {
                s.push_str(_s);
                if count < size - 1 {
                    s.push_str(", ");
                }
                
            }
            count += 1;
        }
    }

    s.push(']');

    Ok(FSRRetValue::Value(FSRString::new_inst(Cow::Owned(s))))
}

impl FSRList {
    pub fn get_class<'a>(vm: &mut FSRVM<'a>) -> FSRClass<'a> {
        let mut cls = FSRClass::new("List");
        let len_m = FSRFn::from_rust_fn(list_len);
        cls.insert_attr("len", len_m, vm);
        let to_string = FSRFn::from_rust_fn(list_string);
        cls.insert_attr("__str__", to_string, vm);
        cls
    }

    pub fn as_string(&self) -> String {
        unimplemented!()
    }

    pub fn new(vs: Vec<u64>) -> FSRObject<'static> {
        let s = Self {
            vs,
        };

        let mut object = FSRObject::new();
        object.set_cls("List");
        object.set_value(FSRValue::List(s));
        object
    }

    pub fn get_items(&self) -> &Vec<u64> {
        &self.vs
    }
}



