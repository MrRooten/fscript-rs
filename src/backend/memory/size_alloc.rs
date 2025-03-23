use std::{borrow::Cow, cell::RefCell, collections::VecDeque, sync::atomic::{AtomicU32, Ordering}};

use crate::backend::types::{base::{FSRObject, FSRValue, ObjId}, integer::FSRInteger, string::FSRString};

use super::FSRAllocator;

#[allow(clippy::vec_box)]
pub struct FSRObjectAllocator<'a> {
    object_bins    : Vec<Box<FSRObject<'a>>>,
    object_to_clear : RefCell<Vec<Box<FSRObject<'a>>>>,
    allocator_count: AtomicU32,
    free_count     : AtomicU32,
}

#[allow(clippy::new_without_default)]
impl<'a> FSRObjectAllocator<'a> {
    pub fn new() -> Self {
        Self {
            object_bins: Vec::new(),
            object_to_clear: RefCell::new(vec![]),
            allocator_count: AtomicU32::new(0),
            free_count: AtomicU32::new(0),
        }
    }

    #[inline(always)]
    pub fn new_object(&mut self, value: FSRValue<'a>, cls: ObjId) -> Box<FSRObject<'a>> {
        // self.allocator_count.fetch_add(1, Ordering::Relaxed);
        if let Some(mut s) = self.object_bins.pop() {
            s.cls = cls;
            s.value = value;
            s.ref_count.store(0, Ordering::Relaxed);
            return s;
        }
        
        Box::new(FSRObject::new_inst(value, cls))
    }

    #[inline(always)]
    pub fn free(&mut self, obj_id: ObjId) {
        let obj = FSRObject::id_to_obj(obj_id);
        
        let obj = FSRObject::into_object(obj_id);
        self.object_bins.push(obj);
        

        // #[allow(clippy::single_match)]
        // match &obj.value {
        //     FSRValue::ClassInst(fsrclass_inst) => fsrclass_inst.drop_obj(self),
        //     _ => {
                
        //     }
        // }

        // if obj.leak.load(Ordering::Relaxed) {
        //     FSRObject::drop_object(obj_id);
        // }
    }

    #[inline(always)]
    pub fn free_object(&mut self, obj: Box<FSRObject<'a>>) {
        self.object_bins.push(obj);
        

        // #[allow(clippy::single_match)]
        // match &obj.value {
        //     FSRValue::ClassInst(fsrclass_inst) => fsrclass_inst.drop_obj(self),
        //     _ => {
                
        //     }
        // }

        // if obj.leak.load(Ordering::Relaxed) {
        //     FSRObject::drop_object(obj_id);
        // }
    }

}

impl<'a> FSRAllocator<'a> for FSRObjectAllocator<'a> {
    fn new() -> Self {
        Self::new()
    }
    
    fn allocate(&mut self, value: FSRValue<'a>, cls: ObjId) -> Box<FSRObject<'a>> {
        let obj = FSRObject::new_inst(value, cls);
        Box::new(obj)
    }
    
    fn free(&mut self, ptr: Box<FSRObject>) {
        
    }

}