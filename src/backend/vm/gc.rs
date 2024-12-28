use std::{cell::RefCell, collections::{HashMap, LinkedList}, sync::atomic::AtomicU64};

use crate::backend::types::base::{FSRObject, ObjId};

#[allow(unused)]
struct ObjectRef {
    obj_id      : u64,
    ref_count   : AtomicU64
}

#[allow(unused)]
pub struct GarbageCollect<'a> {
    objects: RefCell<LinkedList<Box<FSRObject<'a>>>>
}

impl<'a> GarbageCollect<'a> {
    pub fn new_object(&self) -> ObjId {
        unimplemented!()
    }
}