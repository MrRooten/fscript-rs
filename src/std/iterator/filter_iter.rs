use std::any::Any;

use crate::{
    backend::{
        compiler::bytecode::BinaryOffset,
        types::{
            any::{ExtensionTrait, FSRExtension},
            base::{FSRObject, FSRValue, ObjId},
            class::FSRClass,
            fn_def::FSRFn,
            iterator::{FSRIterator, FSRIteratorReferences},
        },
        vm::thread::FSRThreadRuntime,
    },
    utils::error::FSRError,
};

/// Support for chaining iterators
/// For example, `iter1.filter(|x| {
///    x > 0
/// })`
#[derive(Debug)]
pub struct FSRFilterIter {
    pub(crate) filter: ObjId,
    pub(crate) prev_iterator: ObjId,
    pub(crate) code: ObjId,
}

impl FSRIteratorReferences for FSRFilterIter {
    /// This is used to get the references of the iterator
    /// to be used in the garbage collector
    /// to avoid the iterator being collected
    fn ref_objects(&self) -> Vec<ObjId> {
        vec![self.filter, self.prev_iterator]
    }
}

impl FSRIterator for FSRFilterIter {
    fn next(&mut self, thread: &mut FSRThreadRuntime) -> Result<Option<ObjId>, FSRError> {
        let prev_iterator = FSRObject::id_to_obj(self.prev_iterator);
        let next_method_id = prev_iterator
            .get_cls_offset_attr(BinaryOffset::NextObject)
            .unwrap()
            .load(std::sync::atomic::Ordering::Relaxed);
        let next_method = FSRObject::id_to_obj(next_method_id);
        let mut ret = next_method
            .call(&[self.prev_iterator], thread)?
            .get_id();
        if ret == FSRObject::none_id() {
            return Ok(None);
        }

        // Get the filter method
        // and call it with the current value
        // to check if it passes the filter
        let filter = FSRObject::id_to_obj(self.filter);
        let mut filter_ret = filter
            .call(&[ret], thread)?
            .get_id();

        // keep calling the next method until we find a value that passes the filter 
        // or we reach the end of the iterator
        while filter_ret != FSRObject::true_id() {
            ret = next_method
                .call(&[self.prev_iterator], thread)?
                .get_id();
            if ret == FSRObject::none_id() {
                return Ok(None);
            }

            filter_ret = filter
                .call(&[ret], thread)?
                .get_id();
        }
        if filter_ret == FSRObject::none_id() {
            return Ok(None);
        }

        Ok(Some(ret))
    }
}

impl ExtensionTrait for FSRFilterIter {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_reference<'a>(
        &'a self,
        full: bool,
        worklist: &mut Vec<ObjId>,
        is_add: &mut bool,
    ) -> Box<dyn Iterator<Item = ObjId> + 'a> {
        Box::new(vec![self.filter, self.prev_iterator].into_iter())
    }

    fn set_undirty(&mut self) {}
}

impl FSRFilterIter {
    // pub fn new(callback: ObjId, prev_iterator: ObjId) -> FSRValue<'static> {
    //     FSRValue::Any(Box::new(AnyType {
    //         value: Box::new(FSRFilterIter {
    //             filter: callback,
    //             prev_iterator
    //         })
    //     }))
    // }

    pub fn get_class() -> FSRClass {
        let mut cls = FSRClass::new("FSRMapIter");
        cls.init_method();
        cls
    }
}
