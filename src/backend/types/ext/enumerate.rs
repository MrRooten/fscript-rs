use std::any::Any;

use crate::{
    backend::{
        compiler::bytecode::BinaryOffset,
        types::{
            any::{AnyDebugSend, AnyType, GetReference},
            base::{FSRObject, FSRValue, ObjId},
            class::FSRClass,
            fn_def::FSRFn,
            iterator::{FSRIterator, FSRIteratorReferences}, list::FSRList,
        },
        vm::{thread::FSRThreadRuntime, virtual_machine::get_object_by_global_id},
    },
    utils::error::FSRError,
};

#[derive(Debug)]
pub struct FSREnumerateIter {
    //pub(crate) callback: ObjId,
    pub(crate) prev_iterator: ObjId,
    pub(crate) index: i64,
    pub(crate) code: ObjId,
}

impl FSRIteratorReferences for FSREnumerateIter {
    fn ref_objects(&self) -> Vec<ObjId> {
        vec![self.prev_iterator]
    }
}

impl FSRIterator for FSREnumerateIter {
    fn next(&mut self, thread: &mut FSRThreadRuntime) -> Result<Option<ObjId>, FSRError> {
        let prev_iterator = FSRObject::id_to_obj(self.prev_iterator);
        let next_method_id = prev_iterator
            .get_cls_offset_attr(BinaryOffset::NextObject)
            .unwrap()
            .load(std::sync::atomic::Ordering::Relaxed);
        let next_method = FSRObject::id_to_obj(next_method_id);
        let ret = next_method
            .call(&[self.prev_iterator], thread, self.code, next_method_id)
            .unwrap()
            .get_id();
        if ret == FSRObject::none_id() {
            return Ok(None);
        }

        let integer = thread.garbage_collect.new_object(
            FSRValue::Integer(self.index),
            get_object_by_global_id(crate::backend::types::base::FSRGlobalObjId::IntegerCls),
        );
        let ret = vec![integer, ret];
        self.index += 1;
        let ret_value = thread.garbage_collect.new_object(FSRList::new_value(ret), 
            get_object_by_global_id(crate::backend::types::base::FSRGlobalObjId::ListCls),
        );

        Ok(Some(ret_value))
    }
}

impl GetReference for FSREnumerateIter {
    fn get_reference<'a>(
        &'a self,
        full: bool,
        worklist: &mut Vec<ObjId>,
        is_add: &mut bool,
    ) -> Box<dyn Iterator<Item = ObjId> + 'a> {
        Box::new(vec![self.prev_iterator].into_iter())
    }

    fn set_undirty(&mut self) {}
}

impl AnyDebugSend for FSREnumerateIter {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl FSREnumerateIter {
    pub fn get_class() -> FSRClass<'static> {
        let mut cls = FSRClass::new("FSRMapIter");
        cls
    }
}
