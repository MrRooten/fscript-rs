use std::{borrow::Cow, cell::RefCell, collections::VecDeque, sync::atomic::Ordering};

use crate::backend::types::{base::{FSRObject, FSRValue, ObjId}, integer::FSRInteger, string::FSRString};

use super::FSRAllocator;

#[allow(clippy::vec_box)]
pub struct FSRObjectAllocator<'a> {
    object_bins    : RefCell<VecDeque<Box<FSRObject<'a>>>>,
    object_to_clear : RefCell<Vec<Box<FSRObject<'a>>>>
}

#[allow(clippy::new_without_default)]
impl<'a> FSRObjectAllocator<'a> {
    pub fn new() -> Self {
        Self {
            object_bins: RefCell::new(VecDeque::new()),
            object_to_clear: RefCell::new(vec![]),
        }
    }

    pub fn new_object(&self, value: FSRValue<'a>, cls: ObjId) -> Box<FSRObject<'a>> {
        if let Some(mut s) = self.object_bins.borrow_mut().pop_front() {
            s.cls = cls;
            s.value = value;
            return s;
        }
        let obj = FSRObject::new_inst(value, cls);
        Box::new(obj)
    }

    #[inline(always)]
    pub fn free(&self, obj_id: ObjId) {
        let obj = FSRObject::id_to_obj(obj_id);
        // if !obj.delete_flag.load(Ordering::Relaxed) {
        //     return ;
        // }

        
        let obj = FSRObject::into_object(obj_id);
        obj.leak.store(false, Ordering::Relaxed);
        self.object_bins.borrow_mut().push_front(obj);
        return ;
        

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
    pub fn free_object(&self, obj: Box<FSRObject<'a>>) {
        obj.leak.store(false, Ordering::Relaxed);
        self.object_bins.borrow_mut().push_front(obj);
        return ;
        

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

    pub fn free_list(&self) {
        while let Some(s) = self.object_to_clear.borrow_mut().pop() {
            self.free_object(s);
        }
    }
}

impl<'a> FSRAllocator<'a> for FSRObjectAllocator<'a> {
    fn new() -> Self {
        Self::new()
    }
    
    fn allocate(&mut self, value: FSRValue<'a>, cls: ObjId) -> Box<FSRObject<'a>> {
        let obj = FSRObject::new_inst(value, cls);
        let b = Box::new(obj);
        b
    }
    
    fn free(&mut self, ptr: Box<FSRObject>) {
        
    }

}