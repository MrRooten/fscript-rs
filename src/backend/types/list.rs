use std::{borrow::Cow, collections::HashMap};

use ahash::AHashMap;

use crate::{backend::{compiler::bytecode::BinaryOffset, memory::{size_alloc::FSRObjectAllocator, GarbageCollector}, types::{base::{FSRObject, FSRValue}, integer::FSRInteger, iterator::FSRInnerIterator, string::FSRString}, vm::thread::FSRThreadRuntime}, utils::error::{FSRErrCode, FSRError}};

use super::{base::{DropObject, FSRGlobalObjId, FSRRetValue, ObjId}, class::FSRClass, fn_def::FSRFn, code::FSRCode};

#[derive(Debug, Clone)]
pub struct FSRList {
    vs      : Vec<ObjId>
}

fn list_len<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    _module: ObjId
) -> Result<FSRRetValue, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);

    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(

    if let FSRValue::List(self_s) = &self_object.value {
        // return Ok(FSRRetValue::Value(
        //     Box::new(FSRInteger::new_inst(self_s.vs.len() as i64)),
        // ));
        return Ok(FSRRetValue::GlobalId(
            thread.garbage_collect.new_object(
                FSRValue::Integer(self_s.get_items().len() as i64),
                FSRGlobalObjId::IntegerCls as ObjId,
            ),
        ));
    }

    unimplemented!()
}

fn list_string<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId
) -> Result<FSRRetValue, FSRError> {
    let mut s = String::new();
    s.push('[');
    let obj_id = args[0];
    let obj = FSRObject::id_to_obj(obj_id);
    if let FSRValue::List(l) = &obj.value {
        let size = l.get_items().len();
        for (count, id) in l.get_items().iter().enumerate() {
            let obj = FSRObject::id_to_obj(*id);
            let s_obj = obj.to_string(thread, module);
            if let FSRValue::String(_s) = &s_obj.value {
                s.push_str(_s);
                if count < size - 1 {
                    s.push_str(", ");
                }
                
            }
        }
    }

    s.push(']');
    let obj_id = thread.garbage_collect.new_object(
        FSRValue::String(Box::new(Cow::Owned(s))),
        FSRGlobalObjId::StringCls as ObjId,
    );
    Ok(FSRRetValue::GlobalId(obj_id))
}

fn iter<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    __module: ObjId
) -> Result<FSRRetValue, FSRError> {
    let self_id = args[0];
    let iterator = FSRInnerIterator {
        obj: self_id,
        index: 0,
    };

    let inner_obj = thread.garbage_collect.new_object(FSRValue::Iterator(Box::new(iterator)), FSRGlobalObjId::InnerIterator as ObjId);
    Ok(FSRRetValue::GlobalId(inner_obj))

    // Ok(FSRRetValue::Value(Box::new(FSRInnerIterator::new_inst(iterator))))
}

fn get_item<'a>(
    args: &[ObjId],
    _: &mut FSRThreadRuntime<'a>,
    _module: ObjId
) -> Result<FSRRetValue, FSRError>  {
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
            attrs: AHashMap::new(),
            offset_attrs: vec![],
        };
        let len_m = FSRFn::from_rust_fn_static(list_len, "list_len");
        cls.insert_attr("len", len_m);
        let to_string = FSRFn::from_rust_fn_static(list_string, "list_string");
        cls.insert_attr("__str__", to_string);
        let get_iter = FSRFn::from_rust_fn_static(iter, "list_iter");
        cls.insert_attr("__iter__", get_iter);
        let get_item = FSRFn::from_rust_fn_static(get_item, "list_get_item");
        cls.insert_offset_attr(BinaryOffset::GetItem, get_item);
        cls
    }

    pub fn as_string(&self) -> String {
        unimplemented!()
    }

    pub fn new_object(vs: Vec<ObjId>) -> FSRObject<'static> {
        let s = Self {
            vs,
        };

        let mut object = FSRObject::new();
        object.set_cls(FSRGlobalObjId::ListCls as ObjId);
        object.set_value(FSRValue::List(Box::new(s)));
        object
    }

    pub fn get_items(&self) -> &[ObjId] {
        &self.vs
    }

    pub fn new_value(vs: Vec<ObjId>) -> FSRValue<'static> {
        FSRValue::List(Box::new(Self {
            vs,
        }))
    }

    pub fn iter_values(&self) -> impl Iterator<Item = &ObjId> {
        self.vs.iter()
    }
}


impl DropObject<'_> for FSRList {
    fn drop(&self, allocator: &mut FSRObjectAllocator<'_>) {
        for id in &self.vs {
            let obj = FSRObject::id_to_obj(*id);
            // obj.ref_dec();
            // if obj.count_ref() == 1 {
            //     allocator.free(*id);
            // }
        }
    }
}