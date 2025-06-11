use std::{any::Any, fmt::Debug, sync::atomic::Ordering};

use crate::{
    backend::{compiler::bytecode::BinaryOffset, memory::GarbageCollector, types::{ext::enumerate::{self, FSREnumerateIter}, list::FSRList}, vm::{thread::FSRThreadRuntime, virtual_machine::get_object_by_global_id}},
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
#[cfg_attr(feature = "more_inline", inline(always))]
pub fn next_obj(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let self_obj = FSRObject::id_to_mut_obj(args[0]).expect("msg: not a iterator");
    let mut result = None;
    if let FSRValue::Iterator(it) = &mut self_obj.value {
        let from_obj = FSRObject::id_to_obj(it.obj);
        if let FSRValue::ClassInst(inst) = &from_obj.value {
            let cls = from_obj.cls;
            // let cls = FSRObject::id_to_obj(cls);
            // let cls = cls.as_class();
            let v = cls.get_offset_attr(BinaryOffset::Index);
            if let Some(obj_id) = v {
                let obj_id = obj_id.load(Ordering::Relaxed);
                let obj = FSRObject::id_to_obj(obj_id);
                let ret = obj.call(&[it.obj], thread, code, obj_id);
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
        None => FSRRetValue::GlobalId(FSRObject::none_id()),
    })
}

pub fn map(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
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
        code
    };
    let object = thread.garbage_collect.new_object(
        FSRValue::Iterator(Box::new(FSRInnerIterator {
            obj: args[0],
            iterator: Some(Box::new(map_iterator)),
        })),
        get_object_by_global_id(FSRGlobalObjId::InnerIterator),
    );

    Ok(FSRRetValue::GlobalId(object))
}

pub fn filter(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
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
        code,
    };
    let object = thread.garbage_collect.new_object(
        FSRValue::Iterator(Box::new(FSRInnerIterator {
            obj: args[0],
            iterator: Some(Box::new(filter_iterator)),
        })),
        get_object_by_global_id(FSRGlobalObjId::InnerIterator),
    );

    Ok(FSRRetValue::GlobalId(object))
}

pub fn enumerate(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    if args.len() != 1 {
        return Err(FSRError::new(
            "msg: enumerate function requires 1 argument",
            FSRErrCode::NotValidArgs,
        ));
    }
    let self_obj = FSRObject::id_to_mut_obj(args[0]).expect("msg: not a iterator");
    let enumerate_iterator = FSREnumerateIter {
        prev_iterator: args[0],
        index: 0,
        code
    };
    let object = thread.garbage_collect.new_object(
        FSRValue::Iterator(Box::new(FSRInnerIterator {
            obj: args[0],
            iterator: Some(Box::new(enumerate_iterator)),
        })),
        get_object_by_global_id(FSRGlobalObjId::InnerIterator),
    );

    Ok(FSRRetValue::GlobalId(object))
}

pub fn any(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    if args.len() != 2 {
        return Err(FSRError::new(
            "msg: any function requires 1 argument",
            FSRErrCode::NotValidArgs,
        ));
    }
    let self_obj = FSRObject::id_to_mut_obj(args[0]).expect("msg: not a iterator");
    let any_fn_id = args[1];
    let any_fn = FSRObject::id_to_obj(any_fn_id);
    if let FSRValue::Iterator(it) = &mut self_obj.value {
        if let Some(it) = it.iterator.as_mut() {
            let mut result = false;
            while let Ok(Some(obj)) = it.next(thread) {
                let res = any_fn.call(&[obj], thread, code, any_fn_id)?;
                if res.get_id() == FSRObject::true_id() {
                    result = true;
                    break;
                }
            }
            if result {
                return Ok(FSRRetValue::GlobalId(FSRObject::true_id()));
            } else {
                return Ok(FSRRetValue::GlobalId(FSRObject::false_id()));
            }
        }
    }
    unimplemented!()
}

pub fn all(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    if args.len() != 2 {
        return Err(FSRError::new(
            "msg: all function requires 1 argument",
            FSRErrCode::NotValidArgs,
        ));
    }
    let self_obj = FSRObject::id_to_mut_obj(args[0]).expect("msg: not a iterator");
    let all_fn_id = args[1];
    let all_fn = FSRObject::id_to_obj(all_fn_id);
    if let FSRValue::Iterator(it) = &mut self_obj.value {
        if let Some(it) = it.iterator.as_mut() {
            let mut result = true;
            while let Some(obj) = it.next(thread)? {
                let res = all_fn.call(&[obj], thread, code, all_fn_id)?;
                if res.get_id() == FSRObject::false_id() {
                    result = false;
                    break;
                }
            }
            if result {
                return Ok(FSRRetValue::GlobalId(FSRObject::true_id()));
            } else {
                return Ok(FSRRetValue::GlobalId(FSRObject::false_id()));
            }
        }
    }
    unimplemented!()
}

pub fn as_list(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    if args.len() != 1 {
        return Err(FSRError::new(
            "msg: as_list function requires 1 argument",
            FSRErrCode::NotValidArgs,
        ));
    }
    let self_obj = FSRObject::id_to_mut_obj(args[0]).expect("msg: not a iterator");
    if let FSRValue::Iterator(it) = &mut self_obj.value {
        if let Some(it) = it.iterator.as_mut() {
            let mut list = vec![];
            while let Some(obj) = it.next(thread)? {
                list.push(obj);
            }

            let list = FSRList::new_value(list);
            let ret_obj = thread.garbage_collect.new_object(
                list,
                get_object_by_global_id(FSRGlobalObjId::ListCls),
            );
            return Ok(FSRRetValue::GlobalId(ret_obj));
        }
    }
    unimplemented!()
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
        let any = FSRFn::from_rust_fn_static(any, "inner_iterator_any");
        cls.insert_attr("any", any);
        let enumerate = FSRFn::from_rust_fn_static(enumerate, "inner_iterator_enumerate");
        cls.insert_attr("enumerate", enumerate);
        let as_list = FSRFn::from_rust_fn_static(as_list, "inner_iterator_as_list");
        cls.insert_attr("as_list", as_list);
        let all = FSRFn::from_rust_fn_static(all, "inner_iterator_all");
        cls.insert_attr("all", all);
        cls
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
