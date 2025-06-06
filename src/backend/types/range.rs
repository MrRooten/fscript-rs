use std::{ops::Range, sync::atomic::AtomicUsize};

use crate::{
    backend::{
        compiler::bytecode::BinaryOffset, memory::GarbageCollector, types::base::FSRObject,
        vm::{thread::FSRThreadRuntime, virtual_machine::get_object_by_global_id},
    },
    utils::error::FSRError,
};

use super::{
    base::{AtomicObjId, FSRGlobalObjId, FSRRetValue, FSRValue, ObjId}, class::FSRClass, class_inst::FSRClassInst, ext::{filter_iter::FSRFilterIter, map_iter::FSRMapIter}, fn_def::FSRFn, iterator::{FSRInnerIterator, FSRIterator, FSRIteratorReferences}
};

#[derive(Debug, Clone)]
pub struct FSRRange {
    pub(crate) range: Range<i64>,
}

pub struct FSRRangeIterator {
    pub(crate) range_obj: ObjId,
    pub(crate) iter: Range<i64>,
}

impl FSRIteratorReferences for FSRRangeIterator {
    fn ref_objects(&self) -> Vec<ObjId> {
        vec![self.range_obj]
    }
}

impl FSRIterator for FSRRangeIterator {
    fn next(&mut self, thread: &mut FSRThreadRuntime) -> Result<Option<ObjId>, FSRError> {
        // let c = self.iter.next();
        // c.map(|x| {
        //     // let obj_id = thread
        //     //     .garbage_collect
        //     //     .new_object(FSRValue::Integer(x), get_object_by_global_id(FSRGlobalObjId::IntegerCls));
        //     let obj = thread.garbage_collect.new_object_in_place();
        //     obj.value = FSRValue::Integer(x);
        //     obj.cls = get_object_by_global_id(FSRGlobalObjId::IntegerCls);
        //     Ok(FSRObject::obj_to_id(obj))
        // })
        let c = self.iter.next();
        if let Some(x) = c {
            let obj = thread.garbage_collect.new_object_in_place();
            obj.value = FSRValue::Integer(x);
            obj.set_cls(get_object_by_global_id(FSRGlobalObjId::IntegerCls));
            Ok(Some(FSRObject::obj_to_id(obj)))
        } else {
            Ok(None)
        }
    }
}

fn iter_obj(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let self_id = args[0];
    if let FSRValue::Range(it) = &FSRObject::id_to_obj(self_id).value {
        let iterator = FSRInnerIterator {
            obj: self_id,
            iterator: Some(Box::new(FSRRangeIterator {
                range_obj: self_id,
                iter: Range {
                    start: it.range.start,
                    end: it.range.end,
                },
            })),
        };

        let inner_obj = thread.garbage_collect.new_object(
            FSRValue::Iterator(Box::new(iterator)),
            get_object_by_global_id(FSRGlobalObjId::InnerIterator),
        );
        return Ok(FSRRetValue::GlobalId(inner_obj));
    }

    panic!("not a range object")
    //Ok(FSRRetValue::Value(Box::new(FSRInnerIterator::new_inst(iterator))))
}


fn filter(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    
    let iterator = iter_obj(args, len, thread, code)?.get_id();
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let filter_iterator = FSRFilterIter {
        filter: args[1],
        prev_iterator: iterator,
        code,
    };
    let filter_iterator_id = thread.garbage_collect.new_object(
        FSRValue::Iterator(Box::new(FSRInnerIterator {
            obj: iterator,
            iterator: Some(Box::new(filter_iterator)),
        })),
        get_object_by_global_id(FSRGlobalObjId::InnerIterator),
    );


    Ok(FSRRetValue::GlobalId(filter_iterator_id))
}

fn map(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    
    let iterator = iter_obj(args, len, thread, code)?.get_id();
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let map_iterator = FSRMapIter {
        callback: args[1],
        prev_iterator: iterator,
        code,
    };
    let map_iterator_id = thread.garbage_collect.new_object(
        FSRValue::Iterator(Box::new(FSRInnerIterator {
            obj: iterator,
            iterator: Some(Box::new(map_iterator)),
        })),
        get_object_by_global_id(FSRGlobalObjId::InnerIterator),
    );


    Ok(FSRRetValue::GlobalId(map_iterator_id))
}

impl FSRRange {
    pub fn get_class() -> FSRClass<'static> {
        let mut r = FSRClass::new("Range");
        let iter = FSRFn::from_rust_fn_static(iter_obj, "range_iter");
        r.insert_attr("__iter__", iter);
        let filter = FSRFn::from_rust_fn_static(filter, "range_filter");
        r.insert_attr("filter", filter);
        let map = FSRFn::from_rust_fn_static(map, "range_map");
        r.insert_attr("map", map);
        r
    }

    pub fn new_inst() -> FSRClassInst<'static> {
        unimplemented!()
    }

    pub fn get_references(&self) -> Vec<ObjId> {
        vec![]
    }
}
