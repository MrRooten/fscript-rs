use crate::backend::types::base::FSRObject;

pub struct PoolAllocator<'a> {
    pub pools: Vec<Vec<Box<FSRObject<'a>>>>,
    pub size: usize,
}