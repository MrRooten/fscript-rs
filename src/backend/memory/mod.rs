use super::types::base::{FSRObject, FSRValue, ObjId};

pub mod size_alloc;
pub mod gc;

pub trait FSRAllocator<'a> {
    fn new() -> Self;
    fn allocate(&mut self, value: FSRValue<'a>, cls: ObjId) -> Box<FSRObject<'a>>;
    fn free(&mut self, ptr: Box<FSRObject>);
}