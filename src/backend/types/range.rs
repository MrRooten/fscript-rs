use std::ops::Range;

use crate::{backend::{compiler::bytecode::BinaryOffset, memory::GarbageCollector, vm::thread::FSRThreadRuntime}, utils::error::FSRError};

use super::{base::{FSRGlobalObjId, FSRRetValue, FSRValue, ObjId}, class::FSRClass, class_inst::FSRClassInst, fn_def::FSRFn, iterator::FSRInnerIterator};

#[derive(Debug, Clone)]
pub struct FSRRange {
    pub(crate) range: Range<i64>
}

fn iter_obj<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_id = args[0];
    let iterator = FSRInnerIterator {
        obj: self_id,
        index: 0,
    };
    let inner_obj = thread.garbage_collect.new_object(FSRValue::Iterator(Box::new(iterator)), FSRGlobalObjId::InnerIterator as ObjId);
    Ok(FSRRetValue::GlobalId(inner_obj))
    //Ok(FSRRetValue::Value(Box::new(FSRInnerIterator::new_inst(iterator))))
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