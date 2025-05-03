use std::{ops::Range, sync::atomic::AtomicUsize};

use crate::{
    backend::{
        compiler::bytecode::BinaryOffset, memory::GarbageCollector, types::base::FSRObject,
        vm::thread::FSRThreadRuntime,
    },
    utils::error::FSRError,
};

use super::{
    base::{AtomicObjId, FSRGlobalObjId, FSRRetValue, FSRValue, ObjId},
    class::FSRClass,
    class_inst::FSRClassInst,
    fn_def::FSRFn,
    iterator::{FSRInnerIterator, FSRIterator, FSRIteratorReferences},
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
    fn next(&mut self, thread: &mut FSRThreadRuntime) -> Option<Result<ObjId, FSRError>> {
        let c = self.iter.next();
        c.map(|x| {
            // let obj_id = thread
            //     .garbage_collect
            //     .new_object(FSRValue::Integer(x), FSRGlobalObjId::IntegerCls as ObjId);
            let obj = thread.garbage_collect.new_object_in_place();
            obj.value = FSRValue::Integer(x);
            obj.cls = FSRGlobalObjId::IntegerCls as ObjId;
            Ok(FSRObject::obj_to_id(obj))
        })
    }
}

fn iter_obj<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId,
) -> Result<FSRRetValue<'a>, FSRError> {
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
            FSRGlobalObjId::InnerIterator as ObjId,
        );
        return Ok(FSRRetValue::GlobalId(inner_obj));
    }

    panic!("not a range object")
    //Ok(FSRRetValue::Value(Box::new(FSRInnerIterator::new_inst(iterator))))
}

fn next_obj<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId,
) -> Result<FSRRetValue<'a>, FSRError> {
    let iterator = FSRObject::id_to_mut_obj(args[0]);
    let a = (0..3).into_iter();
    unimplemented!()
}

impl FSRRange {
    pub fn get_class() -> FSRClass<'static> {
        let mut r = FSRClass::new("Range");
        let iter = FSRFn::from_rust_fn_static(iter_obj, "range_iter");
        r.insert_attr("__iter__", iter);
        r
    }

    pub fn new_inst() -> FSRClassInst<'static> {
        unimplemented!()
    }
}
