use std::{borrow::Cow, cell::RefCell, collections::VecDeque};

use crate::backend::types::{base::{FSRObject, FSRValue, ObjId}, integer::FSRInteger, string::FSRString};

use super::FSRAllocator;

#[allow(clippy::vec_box)]
pub struct FSRObjectAllocator<'a> {
    integer_bins    : RefCell<VecDeque<Box<FSRObject<'a>>>>,
    object_to_clear : RefCell<Vec<Box<FSRObject<'a>>>>
}

#[allow(clippy::new_without_default)]
impl<'a> FSRObjectAllocator<'a> {
    pub fn new() -> Self {
        Self {
            integer_bins: RefCell::new(VecDeque::new()),
            object_to_clear: RefCell::new(vec![]),
        }
    }

    pub fn add_object_to_clear_list(&self, obj_id: ObjId) {
        let object = FSRObject::into_object(obj_id);
        self.object_to_clear.borrow_mut().push(object);
    }

    pub fn new_object(&self, value: FSRValue<'a>, cls: ObjId) -> Box<FSRObject<'a>> {
        let obj = FSRObject::new_inst(value, cls);
        Box::new(obj)
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

    pub fn new_string(&self, s: Cow<'a, str>) -> Box<FSRObject<'a>> {
        Box::new(FSRString::new_inst(s))
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

        #[allow(clippy::single_match)]
        match &obj.value {
            FSRValue::ClassInst(fsrclass_inst) => fsrclass_inst.drop_obj(self),
            _ => {
                
            }
        }

        if obj.leak.get() {
            FSRObject::drop_object(obj_id);
        }
    }

    /*
    will ignore destructor of type
     */
    pub fn free_object(&self, obj: Box<FSRObject<'a>>) {
        if let FSRValue::Integer(_) = &obj.value {
            obj.leak.set(false);
            self.integer_bins.borrow_mut().push_front(obj);
        }

        // will drop here
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