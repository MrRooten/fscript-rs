use super::{
    types::base::{FSRObject, FSRValue, ObjId},
    vm::thread::{CallFrame, FSRThreadRuntime},
};

pub mod gc;
pub mod mempool;
pub mod size_alloc;

pub trait FSRAllocator<'a> {
    fn new() -> Self;
    fn allocate(&mut self, value: FSRValue<'a>, cls: ObjId) -> Box<FSRObject<'a>>;
    fn free(&mut self, ptr: Box<FSRObject>);
}

pub trait GarbageCollector<'a> {
    fn new_object(&mut self, value: FSRValue<'a>, cls: ObjId) -> ObjId;

    fn collect(&mut self, full: bool);

    fn will_collect(&self) -> bool;
}
