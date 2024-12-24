use std::{cell::RefCell, collections::VecDeque};

use crate::backend::types::{base::{FSRObject, FSRValue, ObjId}, integer::FSRInteger};

pub struct SizeAllocator<'a> {
    integer_bins    : RefCell<VecDeque<Box<FSRObject<'a>>>>
}

impl<'a> SizeAllocator<'a> {
    pub fn new() -> Self {
        Self {
            integer_bins: RefCell::new(VecDeque::new()),
        }
    }

    #[inline(always)]
    pub fn new_integer(&self, i: i64) -> Box<FSRObject<'a>> {
        if let Some(mut s) = self.integer_bins.borrow_mut().pop_front() {
            if let FSRValue::Integer(save) = &mut s.value {
                *save = i;
            }
            return s;
        }
        
        Box::new(FSRInteger::new_inst(i))
    }

    #[inline(always)]
    pub fn free(&self, obj_id: ObjId) {
        let obj = FSRObject::id_to_obj(obj_id);
        if !obj.delete_flag.get() {
            return ;
        }

        if let FSRValue::Integer(_) = &obj.value {
            let obj = FSRObject::into_object(obj_id);
            obj.leak.set(false);
            self.integer_bins.borrow_mut().push_front(obj);
            return ;
        }

        if obj.leak.get() {
            FSRObject::drop_object(obj_id);
        }
    }

    pub fn free_object(&self, obj: Box<FSRObject<'a>>) {
        if let FSRValue::Integer(_) = &obj.value {
            obj.leak.set(false);
            self.integer_bins.borrow_mut().push_front(obj);
        }

        // will drop here
    }
}