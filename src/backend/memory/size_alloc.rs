use std::{borrow::Cow, cell::RefCell, collections::VecDeque, sync::atomic::{AtomicU32, Ordering}};

use crate::backend::types::{base::{FSRObject, FSRValue, ObjId}, integer::FSRInteger, string::FSRString};

use super::FSRAllocator;

#[allow(clippy::vec_box)]
#[derive(Debug)]
pub struct FSRObjectAllocator<'a> {
    object_bins    : Vec<Box<FSRObject<'a>>>,
    value_bins     : Vec<FSRValue<'a>>,
    allocator_count: AtomicU32,
    free_count     : AtomicU32,
}

#[allow(clippy::new_without_default)]
impl<'a> FSRObjectAllocator<'a> {
    pub fn new() -> Self {
        Self {
            object_bins: vec![],
            value_bins: vec![],
            allocator_count: AtomicU32::new(0),
            free_count: AtomicU32::new(0),
        }
    }

    #[inline(always)]
    pub fn new_object(&mut self, mut value: FSRValue<'a>, cls: ObjId) -> Box<FSRObject<'a>> {
        // self.allocator_count.fetch_add(1, Ordering::Relaxed);
        if let Some(mut s) = self.object_bins.pop() {
            s.cls = cls;
            s.value = value;
            //s.ref_count.store(0, Ordering::Relaxed);
            return s;
        }
        
        Box::new(FSRObject::new_inst(value, cls))
    }

    #[inline(always)]
    pub fn free(&mut self, obj_id: ObjId) {
        let obj = FSRObject::id_to_obj(obj_id);
        
        let obj = FSRObject::into_object(obj_id);
        self.object_bins.push(obj);
    }

    #[inline(always)]
    pub fn free_object(&mut self, obj: Box<FSRObject<'a>>) {
        self.object_bins.push(obj);
        
    }

}