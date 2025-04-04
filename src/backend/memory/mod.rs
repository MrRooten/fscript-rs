use super::{types::base::{FSRObject, FSRValue, ObjId}, vm::thread::CallFrame};

pub mod size_alloc;
pub mod gc;

pub trait FSRAllocator<'a> {
    fn new() -> Self;
    fn allocate(&mut self, value: FSRValue<'a>, cls: ObjId) -> Box<FSRObject<'a>>;
    fn free(&mut self, ptr: Box<FSRObject>);
}

pub trait GarbageCollector<'a> {
    fn new_object(&mut self, value: FSRValue<'a>, cls: ObjId) -> ObjId;

    fn collect(&mut self, call_frames: &Vec<Box<CallFrame<'a>>>, cur_frame: &Box<CallFrame<'a>>,others: &[ObjId]);
    
    fn will_collect(&self) -> bool;
}