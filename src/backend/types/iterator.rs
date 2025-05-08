use std::{any::Any, fmt::Debug, sync::atomic::Ordering};

use crate::{
    backend::{compiler::bytecode::BinaryOffset, memory::GarbageCollector, vm::thread::FSRThreadRuntime},
    utils::error::{FSRErrCode, FSRError},
};

use super::{
    base::{AtomicObjId, FSRGlobalObjId, FSRObject, FSRRetValue, FSRValue, ObjId}, class::FSRClass, code::FSRCode, fn_def::FSRFn
};

pub trait FSRIteratorReferences {
    fn ref_objects(&self) -> Vec<ObjId>;
}


pub trait FSRIterator: FSRIteratorReferences + Send {
    fn next(&mut self, thread: &mut FSRThreadRuntime) -> Option<Result<ObjId, FSRError>>;
}

pub struct FSRInnerIterator {
    /// reference to the object that is being iterated
    /// avoid being dropped by the garbage collector
    pub(crate) obj: ObjId,
    pub(crate) iterator: Option<Box<dyn FSRIterator>>,
}

impl Debug for FSRInnerIterator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FSRInnerIterator {{ obj: {} }}", self.obj)
    }
}
#[inline(always)]
pub fn next_obj<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId,
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_obj = FSRObject::id_to_mut_obj(args[0]).expect("msg: not a iterator");
    let mut result = None;
    if let FSRValue::Iterator(it) = &mut self_obj.value {
        let from_obj = FSRObject::id_to_obj(it.obj);
        if let FSRValue::ClassInst(inst) = &from_obj.value {
            let cls = from_obj.cls;
            let cls = FSRObject::id_to_obj(cls);
            let cls = cls.as_class();
            let v = cls.get_offset_attr(BinaryOffset::Index);
            if let Some(obj_id) = v {
                let obj_id = obj_id.load(Ordering::Relaxed);
                let obj = FSRObject::id_to_obj(obj_id);
                let ret = obj.call(&[it.obj], thread, module, obj_id);
                result = Some(ret?);
            }
        } else {
            let iter = it.iterator.as_mut().unwrap();
            if let Some(obj) = iter.next(thread) {
                let obj = obj?;
                result = Some(FSRRetValue::GlobalId(obj));
            } else {
                result = None;
            }
        }
    } else {
        panic!("not a iterator");
    }

    

    Ok(match result {
        Some(s) => s,
        None => FSRRetValue::GlobalId(0),
    })
}

impl FSRInnerIterator {
    pub fn get_class<'a>() -> FSRClass<'a> {
        let mut cls = FSRClass::new("InnerIterator");
        let next = FSRFn::from_rust_fn_static(next_obj, "inner_iterator_next");

        // cls.insert_attr("__next__", next);
        cls.insert_offset_attr(BinaryOffset::NextObject, next);
        cls
    }

    pub fn new_inst<'a>(iterator: FSRInnerIterator) -> FSRObject<'a> {
        let mut object = FSRObject::new();
        object.set_cls(FSRGlobalObjId::InnerIterator as ObjId);
        object.set_value(FSRValue::Iterator(Box::new(iterator)));
        object
    }

    pub fn get_references(&self) -> Vec<ObjId> {
        let mut refs = vec![];
        if let Some(it) = &self.iterator {
            refs.extend(it.ref_objects());
        }
        refs.push(self.obj);
        refs
    }

}
