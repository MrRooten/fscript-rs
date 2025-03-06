use super::types::base::{FSRObject, ObjId};

pub mod size_alloc;
pub mod gc;

pub trait FSRObjectAllocator<T> {
    fn allocate(&mut self, size: T) -> ObjId;
    fn free(&mut self, ptr: ObjId);
}