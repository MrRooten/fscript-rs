use std::{any::Any, fmt::Debug, sync::atomic::Ordering};

use crate::{
    backend::{compiler::bytecode::BinaryOffset, memory::GarbageCollector, vm::thread::FSRThreadRuntime},
    utils::error::{FSRErrCode, FSRError},
};

use super::{
    base::{AtomicObjId, FSRGlobalObjId, FSRObject, FSRRetValue, FSRValue, ObjId}, class::FSRClass, code::FSRCode, ext::{filter_iter::FSRFilterIter, map_iter::FSRMapIter}, fn_def::FSRFn
};

pub trait FSRIteratorReferences {
    fn ref_objects(&self) -> Vec<ObjId>;
}


pub trait FSRIterator: FSRIteratorReferences + Send {
    fn next(&mut self, thread: &mut FSRThreadRuntime) -> Result<Option<ObjId>, FSRError>;
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
pub fn next_obj(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime,
    module: ObjId,
) -> Result<FSRRetValue, FSRError> {
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
            if let Some(obj) = iter.next(thread)? {
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

pub fn map(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime,
    module: ObjId,
) -> Result<FSRRetValue, FSRError> {
    if args.len() != 2 {
        return Err(FSRError::new(
            "msg: map function requires 2 arguments",
            FSRErrCode::NotValidArgs,
        ));
    }
    let self_obj = FSRObject::id_to_mut_obj(args[0]).expect("msg: not a iterator");
    let map_fn_id = args[1];
    let map_iterator = FSRMapIter {
        callback: map_fn_id,
        prev_iterator: args[0],
        module
    };
    let object = thread.garbage_collect.new_object(
        FSRValue::Iterator(Box::new(FSRInnerIterator {
            obj: args[0],
            iterator: Some(Box::new(map_iterator)),
        })),
        FSRGlobalObjId::InnerIterator as ObjId,
    );

    Ok(FSRRetValue::GlobalId(object))
}

pub fn filter(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime,
    module: ObjId,
) -> Result<FSRRetValue, FSRError> {
    if args.len() != 2 {
        return Err(FSRError::new(
            "msg: filter function requires 2 arguments",
            FSRErrCode::NotValidArgs,
        ));
    }
    let self_obj = FSRObject::id_to_mut_obj(args[0]).expect("msg: not a iterator");
    let filter_fn_id = args[1];
    let filter_iterator = FSRFilterIter {
        filter: filter_fn_id,
        prev_iterator: args[0],
        module,
    };
    let object = thread.garbage_collect.new_object(
        FSRValue::Iterator(Box::new(FSRInnerIterator {
            obj: args[0],
            iterator: Some(Box::new(filter_iterator)),
        })),
        FSRGlobalObjId::InnerIterator as ObjId,
    );

    Ok(FSRRetValue::GlobalId(object))
}


impl FSRInnerIterator {
    pub fn get_class<'a>() -> FSRClass<'a> {
        let mut cls = FSRClass::new("InnerIterator");
        let next = FSRFn::from_rust_fn_static(next_obj, "inner_iterator_next");

        // cls.insert_attr("__next__", next);
        cls.insert_offset_attr(BinaryOffset::NextObject, next);
        let map = FSRFn::from_rust_fn_static(map, "inner_iterator_map");
        cls.insert_attr("map", map);
        let filter = FSRFn::from_rust_fn_static(filter, "inner_iterator_filter");
        cls.insert_attr("filter", filter);
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
