use std::{borrow::Cow, collections::HashMap};

use crate::{backend::{compiler::bytecode::BinaryOffset, types::{base::{FSRObject, FSRValue}, integer::FSRInteger, iterator::FSRInnerIterator, string::FSRString}, vm::thread::FSRThreadRuntime}, utils::error::{FSRErrCode, FSRError}};

use super::{base::{FSRGlobalObjId, FSRRetValue}, class::FSRClass, fn_def::FSRFn, module::FSRModule};

#[derive(Debug, Clone)]
pub struct FSRList {
    vs      : Vec<u64>
}

fn list_len<'a>(
    args: &[u64],
    _: &mut FSRThreadRuntime<'a>,
    _module: Option<&FSRModule>
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);

    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(

    if let FSRValue::List(self_s) = &self_object.value {
        return Ok(FSRRetValue::Value(
            Box::new(FSRInteger::new_inst(self_s.vs.len() as i64)),
        ));
    }

    unimplemented!()
}

fn list_string<'a>(
    args: &[u64],
    thread: &mut FSRThreadRuntime<'a>,
    _module: Option<&FSRModule>
) -> Result<FSRRetValue<'a>, FSRError> {
    let mut s = String::new();
    s.push('[');
    let obj_id = args[0];
    let obj = FSRObject::id_to_obj(obj_id);
    if let FSRValue::List(l) = &obj.value {
        let size = l.get_items().len();
        for (count, id) in l.get_items().iter().enumerate() {
            let obj = FSRObject::id_to_obj(*id);
            let s_obj = obj.to_string(thread);
            if let FSRValue::String(_s) = &s_obj.value {
                s.push_str(_s);
                if count < size - 1 {
                    s.push_str(", ");
                }
                
            }
        }
    }

    s.push(']');

    Ok(FSRRetValue::Value(Box::new(FSRString::new_inst(Cow::Owned(s)))))
}

fn iter<'a>(
    args: &[u64],
    _: &mut FSRThreadRuntime<'a>,
    _module: Option<&FSRModule>
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_id = args[0];
    let iterator = FSRInnerIterator {
        obj: self_id,
        index: 0,
    };

    return Ok(FSRRetValue::Value(Box::new(FSRInnerIterator::new_inst(iterator))));
}

fn get_item<'a>(
    args: &[u64],
    _: &mut FSRThreadRuntime<'a>,
    _module: Option<&FSRModule>
) -> Result<FSRRetValue<'a>, FSRError>  {
    let self_id = args[0];
    let index_id = args[1];
    let obj = FSRObject::id_to_obj(self_id);
    let index_obj = FSRObject::id_to_obj(index_id);
    if let FSRValue::List(l) = &obj.value {
        if let FSRValue::Integer(i) = &index_obj.value {
            let index = *i as usize;
            if let Some(s) = l.vs.get(index) {
                return Ok(FSRRetValue::GlobalId(*s));
            } else {
                return Err(FSRError::new("list index of range", FSRErrCode::OutOfRange));
            }
        }
    }
    unimplemented!()
}

impl FSRList {
    pub fn get_class<'a>() -> FSRClass<'a> {
        let mut cls = FSRClass {
            name: "List",
            attrs: HashMap::new(),
            offset_attrs: vec![0;30],
        };
        let len_m = FSRFn::from_rust_fn(list_len);
        cls.insert_attr("len", len_m);
        let to_string = FSRFn::from_rust_fn(list_string);
        cls.insert_attr("__str__", to_string);
        let get_iter = FSRFn::from_rust_fn(iter);
        cls.insert_attr("__iter__", get_iter);
        let get_item = FSRFn::from_rust_fn(get_item);
        cls.insert_offset_attr(BinaryOffset::GetItem, get_item);
        cls
    }

    pub fn as_string(&self) -> String {
        unimplemented!()
    }

    pub fn new_object(vs: Vec<u64>) -> FSRObject<'static> {
        let s = Self {
            vs,
        };

        let mut object = FSRObject::new();
        object.set_cls(FSRGlobalObjId::ListCls as u64);
        object.set_value(FSRValue::List(s));
        object
    }

    pub fn get_items(&self) -> &[u64] {
        &self.vs
    }
}



