use super::types::base::{FSRObject, FSRValue, ObjId};

pub mod size_alloc;
pub mod gc;

pub trait FSRAllocator<'a> {
    fn new() -> Self;
    fn allocate(&mut self, value: FSRValue<'a>, cls: ObjId) -> Box<FSRObject<'a>>;
    fn free(&mut self, ptr: Box<FSRObject>);
}

pub trait GarbageCollector<'a> {
    fn new_object(&mut self, value: FSRValue<'a>, cls: ObjId) -> ObjId;

    fn free_object(&mut self, id: ObjId);

    fn collect(&mut self);
}