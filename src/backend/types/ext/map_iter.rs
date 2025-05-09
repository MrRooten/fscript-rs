use std::any::Any;

use crate::{backend::{compiler::bytecode::BinaryOffset, types::{any::{AnyDebugSend, AnyType, GetReference}, base::{FSRObject, FSRValue, ObjId}, class::FSRClass, fn_def::FSRFn, iterator::{FSRIterator, FSRIteratorReferences}}, vm::thread::FSRThreadRuntime}, utils::error::FSRError};

#[derive(Debug)]
pub struct FSRMapIter {
    pub(crate) callback: ObjId,
    pub(crate) prev_iterator: ObjId
}

impl FSRIteratorReferences for FSRMapIter {
    fn ref_objects(&self) -> Vec<ObjId> {
        vec![self.callback, self.prev_iterator]
    }
}

impl FSRIterator for FSRMapIter {
    fn next(&mut self, thread: &mut FSRThreadRuntime) -> Result<Option<ObjId>, FSRError> {
        let prev_iterator = FSRObject::id_to_obj(self.prev_iterator);
        let next_method_id = prev_iterator.get_cls_offset_attr(BinaryOffset::NextObject).unwrap().load(std::sync::atomic::Ordering::Relaxed);
        let next_method = FSRObject::id_to_obj(next_method_id);
        let ret = next_method.call(&[self.prev_iterator], thread, 0, next_method_id).unwrap().get_id();
        if ret == FSRObject::none_id() {
            return Ok(None);
        }

        let callback = FSRObject::id_to_obj(self.callback);
        let map_ret = callback.call(&[ret], thread, 0, self.callback).unwrap().get_id();
        if map_ret == FSRObject::none_id() {
            return Ok(None);
        }

        Ok(Some(map_ret))
    }
}

impl GetReference for FSRMapIter {
    fn get_reference<'a>(
        &'a self,
        full: bool,
        worklist: &mut Vec<ObjId>,
        is_add: &mut bool,
    ) -> Box<dyn Iterator<Item = ObjId> + 'a> {
        Box::new(vec![self.callback, self.prev_iterator].into_iter())
    }

    fn set_undirty(&mut self) {
        
    }
}

impl AnyDebugSend for FSRMapIter {
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl FSRMapIter {
    pub fn new(callback: ObjId, prev_iterator: ObjId) -> FSRValue<'static> {
        FSRValue::Any(Box::new(AnyType {
            value: Box::new(FSRMapIter {
                callback,
                prev_iterator
            })
        }))
    }

    pub fn get_class() -> FSRClass<'static> {
        let mut cls = FSRClass::new("FSRMapIter");
        cls
    }
}